use circuit::{config::RoqcConfig, CircuitSeq};
use rayon::prelude::*;
use roqc::optimize_light;
pub struct Roqc {
    config: RoqcConfig,
}

impl Roqc {
    pub fn new(config: RoqcConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, circs: Vec<CircuitSeq>) -> Vec<CircuitSeq> {
        let optimized_circs = circs
            .par_iter()
            .map(|circ| {
                let mut circ = circ.clone();
                optimize_light(&mut circ);
                circ
            })
            .collect();
        optimized_circs
    }
    pub fn run_single(&self, circ: CircuitSeq) -> CircuitSeq {
        let mut circ = circ.clone();
        optimize_light(&mut circ);
        circ
    }

    pub fn shutdown(&self) {}
}
