use crate::oracles::quartz::quartz_impl::Quartz;
use crate::{
    oracles::qiskit::Qiskit, oracles::roqc_oracle::Roqc, oracles::tket::tket_impl::Tket,
    oracles::voqc::Voqc,
};
use circuit::CircuitSeq;

pub enum OracleEnum {
    SyncQuartz(Quartz),
    Voqc(Voqc),
    Roqc(Roqc),
    Qiskit(Qiskit),
    Tket(Tket),
}

impl OracleEnum {
    // pub fn run(&self, circs: Vec<CircuitSeq>, function_name: String) -> Vec<CircuitSeq> {
    //     match self {
    //         OracleEnum::SyncQuartz(oracle) => oracle.run(circs, function_name),
    //         OracleEnum::Voqc(oracle) => oracle.run(circs),
    //         OracleEnum::Roqc(oracle) => oracle.run(circs),
    //     }
    // }
    pub fn run_single(&self, circ: CircuitSeq, task_id: usize) -> CircuitSeq {
        match self {
            OracleEnum::SyncQuartz(oracle) => oracle.run_single(circ, "optimize".to_string()),
            OracleEnum::Voqc(oracle) => oracle.run_single(circ, task_id),
            OracleEnum::Roqc(oracle) => oracle.run_single(circ),
            OracleEnum::Qiskit(oracle) => oracle.run_single(circ, task_id),
            OracleEnum::Tket(oracle) => oracle.run_single(circ, task_id),
        }
    }
    pub fn shutdown(&self) {
        match self {
            OracleEnum::SyncQuartz(oracle) => oracle.shutdown(),
            OracleEnum::Voqc(_) => {}
            OracleEnum::Roqc(oracle) => oracle.shutdown(),
            OracleEnum::Qiskit(_) => {}
            OracleEnum::Tket(_) => {}
        }
    }

    // pub fn oracle_running_time(&self) -> u64 {
    //     match self {
    //         OracleEnum::SyncQuartz(_) => {
    //             panic!("SyncQuartz does not have oracle_running_time")
    //         }
    //         OracleEnum::Voqc(oracle) => oracle.oracle_running_time,
    //         OracleEnum::Roqc(oracle) => oracle.oracle_running_time,
    //     }
    // }
}
#[cfg(test)]
mod tests {

    use std::path;

    use super::*;
    use circuit::config::{MultipleConfigs, OracleName};
    use circuit::CircuitSeq;
    #[test]
    fn test_oracle_utils() {
        let circ = CircuitSeq::new_from_file(path::Path::new("benchmarks/testsoam2.qasm"));

        let config = MultipleConfigs::read_config(&"configs/test_quartz.toml".to_string())
            .to_single_configs()[0]
            .clone();
        let oracle_runner = match config.oracle_name {
            OracleName::Quartz(ref quartz_config) => {
                let oracle_runner = Quartz::new(quartz_config.clone(), 12345);
                OracleEnum::SyncQuartz(oracle_runner)
            }
            OracleName::Voqc(ref voqc_config) => {
                let oracle_runner = Voqc::new(voqc_config.clone());
                OracleEnum::Voqc(oracle_runner)
            }
            OracleName::Roqc(ref roqc_config) => {
                let oracle_runner = Roqc::new(roqc_config.clone());
                OracleEnum::Roqc(oracle_runner)
            }
            OracleName::Qiskit(ref qiskit_config) => {
                let oracle_runner = Qiskit::new(qiskit_config.clone());
                OracleEnum::Qiskit(oracle_runner)
            }
            OracleName::Tket(ref tket_config) => {
                let oracle_runner = Tket::new(tket_config.clone());
                OracleEnum::Tket(oracle_runner)
            }
        };
        for _ in 0..10 {
            let res = oracle_runner.run_single(circ.clone(), 0);
            println!("res: {:?}", res.len());
            println!("res: {:?}", res);
        }
        oracle_runner.shutdown();
    }
}
