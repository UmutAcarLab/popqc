use std::process::Command;

use circuit::config::VoqcConfig;
use circuit::CircuitSeq;
#[allow(dead_code)]
pub struct Voqc {
    config: VoqcConfig,
}

impl Voqc {
    pub fn new(config: VoqcConfig) -> Self {
        Voqc { config }
    }
    // pub fn run(&self, circs: Vec<CircuitSeq>) -> Vec<CircuitSeq> {
    //     let mut optimized_circs = vec![];
    //     for circ in circs {
    //         let circ_str = circ.dump();
    //         std::fs::write("temp.qasm", circ_str).expect("Unable to write file");
    //         let _ = Command::new("./resources/voqc/voqc_exec")
    //             .args(["-f", "./temp.qasm"])
    //             .args(["-o", "./temp_out.qasm"])
    //             .output()
    //             .expect("Failed to execute command");
    //         let output_str = std::fs::read("temp_out.qasm").expect("Unable to read file");
    //         let optimized_circ = CircuitSeq::new_from_source(&String::from_utf8_lossy(&output_str));
    //         optimized_circs.push(optimized_circ);
    //     }
    //     optimized_circs
    // }
    pub fn run_single(&self, circ: CircuitSeq, task_id: usize) -> CircuitSeq {
        let circ_str = circ.dump();
        std::fs::write(format!("temp_{}.qasm", task_id), circ_str).expect("Unable to write file");
        // check platform
        let platform = std::env::consts::OS;
        if platform == "linux" {
            let _ = Command::new("./resources/voqc/voqc_exec_linux")
                .args(["-f", &format!("temp_{}.qasm", task_id)])
                .args(["-o", &format!("temp_out_{}.qasm", task_id)])
                .output()
                .expect("Failed to execute command");
        } else if platform == "macos" {
            let _ = Command::new("./resources/voqc/voqc_exec_mac")
                .args(["-f", &format!("temp_{}.qasm", task_id)])
                .args(["-o", &format!("temp_out_{}.qasm", task_id)])
                .output()
                .expect("Failed to execute command");
        } else {
            panic!("Unsupported platform: {}", platform);
        }
        let output_str =
            std::fs::read(format!("temp_out_{}.qasm", task_id)).expect("Unable to read file");
        let optimized_circ = CircuitSeq::new_from_source(&String::from_utf8_lossy(&output_str));
        std::fs::remove_file(format!("temp_{}.qasm", task_id)).expect("Unable to remove file");
        std::fs::remove_file(format!("temp_out_{}.qasm", task_id)).expect("Unable to remove file");
        optimized_circ
    }
}
