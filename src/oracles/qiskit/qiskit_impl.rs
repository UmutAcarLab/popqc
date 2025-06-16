use std::process::Command;

use circuit::config::QiskitConfig;
use circuit::CircuitSeq;
#[allow(dead_code)]
pub struct Qiskit {
    config: QiskitConfig,
}

impl Qiskit {
    pub fn new(config: QiskitConfig) -> Self {
        Qiskit { config }
    }
    pub fn run_single(&self, circ: CircuitSeq, task_id: usize) -> CircuitSeq {
        let circ_str = circ.dump();
        std::fs::write(format!("temp_{}.qasm", task_id), circ_str).expect("Unable to write file");
        let _ = Command::new("/home/cc/miniconda3/envs/qiskit/bin/python")
            .arg("/home/cc/quicr/soam/resources/qiskit/run_qiskit.py")
            // let _ = Command::new("/home/cc/quicr/soam/resources/qiskit/run_qiskit.bin")
            .args(["-f", &format!("temp_{}.qasm", task_id)])
            .args(["-o", &format!("temp_out_{}.qasm", task_id)])
            .output()
            .expect("Failed to execute command");
        let output_str =
            std::fs::read(format!("temp_out_{}.qasm", task_id)).expect("Unable to read file");
        let optimized_circ = CircuitSeq::new_from_source(&String::from_utf8_lossy(&output_str));
        std::fs::remove_file(format!("temp_{}.qasm", task_id)).expect("Unable to remove file");
        std::fs::remove_file(format!("temp_out_{}.qasm", task_id)).expect("Unable to remove file");
        optimized_circ
    }
}
