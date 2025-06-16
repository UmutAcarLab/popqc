use super::single_quartz::SingleQuartz;
use circuit::config::QuartzConfig;
use circuit::config::TimeOut;
use circuit::CircuitSeq;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::Notify;

// This struct provides a multi-threaded async interface to the different optimizer.

pub struct Quartz {
    servers: Vec<Arc<SingleQuartz>>,
    server_status: Arc<Mutex<Vec<bool>>>,
    notify: Arc<Notify>,
    rt: tokio::runtime::Runtime,
}

impl Quartz {
    pub fn new(config: QuartzConfig, port: u16) -> Self {
        let runners = (0..config.n_threads)
            .map(|_| Arc::new(SingleQuartz::new()))
            .collect();
        let status = vec![false; config.n_threads];
        let rt = tokio::runtime::Runtime::new().unwrap();
        let this = Quartz {
            servers: runners,
            server_status: Arc::new(Mutex::new(status)),
            notify: Arc::new(Notify::new()),
            rt,
        };
        this.rt.block_on(async {
            this.initialize(
                port,
                config.gateset.to_string(),
                config.ecc_path.to_string(),
                config.cost.to_string(),
                config.timeout.clone(),
            )
            .await
        });
        this
    }

    pub async fn initialize(
        &self,
        starting_port: u16,
        gate_set: String,
        ecc_file: String,
        cost_func: String,
        timeout: TimeOut,
    ) {
        let mut futures = Vec::new();

        for (i, runner) in self.servers.iter().enumerate() {
            let port = starting_port + i as u16;
            futures.push(runner.initialize(
                port,
                gate_set.clone(),
                ecc_file.clone(),
                cost_func.clone(),
                timeout.clone(),
            ));
        }
        let _results = futures::future::join_all(futures).await;
    }

    pub async fn optimize_single_async(&self, circuit: String, function_name: String) -> String {
        loop {
            // Try to find an available runner
            let runner_index = {
                let mut status = self.server_status.lock().await;
                let available = status.iter().position(|&busy| !busy);
                if let Some(idx) = available {
                    status[idx] = true; // Mark as busy
                    Some(idx)
                } else {
                    None
                }
            };

            if let Some(idx) = runner_index {
                // We found an available runner
                let runner = &self.servers[idx];
                let result = runner.optimize(circuit, function_name).await;

                // Mark the runner as available again
                let mut status = self.server_status.lock().await;
                status[idx] = false;

                // Notify any waiting tasks that a runner is now available
                self.notify.notify_one();

                return result;
            }

            // If no runner is available, wait for notification
            self.notify.notified().await;
        }
    }
    pub fn run_single(&self, circ: CircuitSeq, function_name: String) -> CircuitSeq {
        let circ_string = circ.dump();
        let res = self
            .rt
            .block_on(async { self.optimize_single_async(circ_string, function_name).await });
        CircuitSeq::new_from_source(&res)
    }
    pub async fn optimize_all(&self, circuits: Vec<String>, function_name: String) -> Vec<String> {
        //A more advanced and efficient way to do job balancing
        let num_circuits = circuits.len();
        let mut results = vec![None; num_circuits];

        let queue: Arc<Mutex<VecDeque<_>>> =
            Arc::new(Mutex::new(circuits.into_iter().enumerate().collect()));
        let (tx, mut rx) = mpsc::channel(num_circuits);

        let servers = Arc::new(self.servers.clone());

        for runner in servers.iter() {
            let tx = tx.clone();
            let queue = Arc::clone(&queue);
            let runner = Arc::clone(runner);
            let function_name = function_name.clone();
            tokio::spawn(async move {
                loop {
                    let task = {
                        let mut guard = queue.lock().await;
                        guard.pop_front()
                    };

                    if let Some((index, circuit)) = task {
                        let optimized_circuit =
                            runner.optimize(circuit, function_name.clone()).await;
                        tx.send((index, optimized_circuit)).await.unwrap();
                    } else {
                        break;
                    }
                }
            });
        }

        drop(tx);

        while let Some((index, result)) = rx.recv().await {
            results[index] = Some(result);
        }

        results.into_iter().flatten().collect()
    }

    pub fn shutdown(&self) {
        self.rt.block_on(async {
            for runner in self.servers.iter() {
                runner.shutdown().await;
            }
        });
        //make sure all the runners are shutdown
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use circuit::CircuitSeq;
//     use std::path;

//     #[test]
//     fn test_manager() {
//         let circ = CircuitSeq::new_from_file(path::Path::new("benchmarks/test.qasm"));

//         let manager = MultipleQuartz::new(3);
//         let rt = tokio::runtime::Runtime::new().unwrap();
//         rt.block_on(async {
//             manager
//                 .initialize(
//                     12385,
//                     "Nam_B".to_string(),
//                     "resources/Nam_4_3_complete_ECC_set.json".to_string(),
//                     "Gate".to_string(),
//                     TimeOut::PerSegment(100),
//                 )
//                 .await
//         });
//         println!("Initialized");
//         let result = rt.block_on(async {
//             manager
//                 .optimize_all(vec![circ.dump(); 10], "optimize".to_string())
//                 .await
//         });
//         println!("result: {:?}", result);

//         rt.block_on(async { manager.shutdown().await });
//     }
// }
