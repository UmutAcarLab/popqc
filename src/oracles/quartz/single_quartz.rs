use core::panic;
use msgpack_rpc::Client;
use std::env::consts;
use std::net::SocketAddr;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_util::compat::TokioAsyncReadCompatExt;

use circuit::config::TimeOut;

// This struct provides a single-threaded async interface to the Quartz optimizer.
pub struct SingleQuartz {
    server: Arc<Mutex<Option<(u16, Child)>>>,
    client: Arc<Mutex<Option<Client>>>,
}

impl Default for SingleQuartz {
    fn default() -> Self {
        Self::new()
    }
}

impl SingleQuartz {
    pub fn new() -> Self {
        SingleQuartz {
            server: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn initialize(
        &self,
        port: u16,
        gate_set: String,
        ecc_file: String,
        cost_func: String,
        timeout: TimeOut,
    ) {
        let timeout_type = match timeout {
            TimeOut::PerSegment(_) => "PerSegment",
            TimeOut::PerGate(_) => "PerGate",
        };
        let timeout_value = match timeout {
            TimeOut::PerSegment(t) => t,
            TimeOut::PerGate(t) => t,
        };
        let quartz_command = match consts::OS {
            "windows" => "./resources/quartz/build/Release/wrapper_rpc.exe",
            _ => "./resources/quartz/build/wrapper_rpc",
        };
        let child = Command::new(quartz_command)
            .arg(port.to_string())
            .arg(gate_set)
            .arg(ecc_file)
            .arg(cost_func)
            .arg(timeout_type)
            .arg(timeout_value.to_string())
            .spawn()
            .expect("Failed to start optimizer process");
        let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();
        let mut attempts = 0;
        let my_socket: Option<TcpStream>;
        loop {
            attempts += 1;
            match TcpStream::connect(&addr).await {
                Ok(socket) => {
                    my_socket = Some(socket);
                    break;
                }
                Err(_) => {
                    if attempts >= 20 {
                        println!("Reached maximum number of connection attempts. Exiting.");
                        panic!("Failed to connect to optimizer process");
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        let client = Client::new(my_socket.unwrap().compat());

        let mut server_lock = self.server.lock().await;
        *server_lock = Some((port, child));

        let mut client_lock = self.client.lock().await;
        *client_lock = Some(client);
    }

    pub async fn optimize(&self, circuit: String, function_name: String) -> String {
        let client_lock = self.client.lock().await;
        let client = client_lock.as_ref().expect("Client not initialized");

        client
            .request(function_name.as_str(), &[circuit.into()])
            .await
            .map(|resp| resp.as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "An error occurred".to_string())
    }

    pub async fn shutdown(&self) {
        let mut server_lock = self.server.lock().await;
        if let Some((_port, mut child)) = server_lock.take() {
            child.kill().expect("Failed to kill optimizer process");
        }

        let mut client_lock = self.client.lock().await;
        *client_lock = None;
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path};

    use super::*;
    use circuit::CircuitSeq;
    #[test]
    fn test_quartz() {
        let circ: CircuitSeq = CircuitSeq::new_from_file(path::Path::new("benchmarks/test.qasm"));
        let quartz = SingleQuartz::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            quartz
                .initialize(
                    12375,
                    "Nam_B".to_string(),
                    "resources/Nam_4_3_complete_ECC_set.json".to_string(),
                    "Gate".to_string(),
                    TimeOut::PerSegment(100.0),
                )
                .await
        });
        println!("Initialized");
        let result =
            rt.block_on(async { quartz.optimize(circ.dump(), "optimize".to_string()).await });
        println!("result: {:?}", result);
        rt.block_on(async { quartz.shutdown().await });
    }
    #[test]
    fn run_preprocess() {
        for i in vec![18, 22, 26, 30] {
            let circ: CircuitSeq = CircuitSeq::new_from_file(path::Path::new(
                format!(
                    "benchmarks/vqe_new/vqe_n{}_from_python.qasm.preprocessed",
                    i
                )
                .as_str(),
            ));
            let quartz = SingleQuartz::new();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                quartz
                    .initialize(
                        12376,
                        "Nam_B".to_string(),
                        "resources/Nam_5_3_complete_ECC_set.json".to_string(),
                        "Gate".to_string(),
                        TimeOut::PerSegment(0.1),
                    )
                    .await
            });
            println!("Initialized");
            let result = rt.block_on(async {
                quartz
                    .optimize(circ.dump(), "rotation_merging".to_string())
                    .await
            });
            let mut file = File::create(format!(
                "benchmarks/vqe_new/vqe_n{}_from_python.qasm.preprocessed.new",
                i
            ))
            .unwrap();
            file.write_all(result.as_bytes()).unwrap();
            rt.block_on(async { quartz.shutdown().await });
        }
    }
}
