use crate::results::MultipleResults;
use circuit::config::MultipleConfigs;

use serde::Serialize;
#[derive(Serialize)]
struct CsvRecord {
    config: String,
    gates_before_optimization: usize,
    depth_before_optimization: usize,
    gates_after_optimization: usize,
    depth_after_optimization: usize,
    n_rounds: usize,
    time: f32,
    oracle_time: f32,
    n_seams_total: usize,
}
pub fn analyze(config_path: &str) {
    let result_path = config_path.replace("configs", "results");
    let results_string = std::fs::read_to_string(&result_path).expect("failed to read result file");
    let configs_string = std::fs::read_to_string(config_path).expect("failed to read config file");
    let results: MultipleResults =
        toml::from_str(&results_string).expect("failed to parse config file");
    let config: MultipleConfigs =
        toml::from_str(&configs_string).expect("failed to parse config file");
    let unique_elements = config.unique_config_elements();
    //print unique elements first
    println!("********************************* Common Configs *********************************");
    config.print_unique_elements();
    let mut records_csv = Vec::new();
    //print non-unique elements for each result
    for config_result in results.results.iter() {
        println!("------------------------");
        config_result
            .config
            .print_non_unique_elements(&unique_elements);
        println!("{:?}", config_result.result);
        records_csv.push(CsvRecord {
            config: config_result.config.non_unique_elements(&unique_elements),
            gates_before_optimization: config_result.result.original_gates,
            depth_before_optimization: config_result.result.original_depth,
            gates_after_optimization: config_result.result.optimized_gates,
            depth_after_optimization: config_result.result.optimized_depth,
            n_rounds: config_result.result.n_rounds,
            time: config_result.result.time,
            oracle_time: config_result.result.oracle_time,
            n_seams_total: config_result.result.n_seams_total,
        });
    }
    let mut wtr = csv::Writer::from_path(result_path.clone().replace("toml", "csv"))
        .expect("failed to create csv file");
    for record in records_csv {
        wtr.serialize(record).expect("failed to serialize record");
    }
}
