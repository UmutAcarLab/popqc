use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use circuit::config::SingleConfig;
#[derive(Serialize, Deserialize, Debug)]
pub struct SingleResult {
    pub original_depth: usize,
    pub optimized_depth: usize,
    pub original_gates: usize,
    pub optimized_gates: usize,
    pub n_rounds: usize,
    pub time: f32,
    pub oracle_time: f32,
    pub n_seams_total: usize,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigResult {
    pub config: SingleConfig,
    pub result: SingleResult,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MultipleResults {
    pub results: Vec<ConfigResult>,
}
pub fn write_results(config_path: &str, results: &MultipleResults) {
    //dump the config first, then some properties of the optimized circuit
    let result_path = config_path.replace("configs", "results");
    let path = Path::new(&result_path);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent directory");
    }

    let mut file = File::create(path).expect("failed to create file");

    let _ = file.write_all(toml::to_string(results).unwrap().as_bytes());
}
