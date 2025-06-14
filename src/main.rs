use circuit::config::Cost;
use circuit::layer::Layout;
use circuit::{config::MultipleConfigs, config::SingleConfig, CircuitLayer, CircuitSeq};
use rayon::ThreadPoolBuilder;
use shellexpand;
use soam::optimizer::SoamRunner;
use soam::results::{ConfigResult, MultipleResults, SingleResult};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
const CORRECTNESS_CHECK: bool = false;
const DUMP: bool = false;

fn main() {
    // env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let config_path = &args[1];
    run(config_path);
    soam::analyze::analyze(config_path);
}
fn single_run(config: &SingleConfig) -> SingleResult {
    let layout = config.layout.clone();
    let circuit_file = PathBuf::from(config.circuit_path.clone());
    let circuit = CircuitLayer::from_seq(
        CircuitSeq::new_from_file(circuit_file.as_path()),
        layout.clone(),
    );
    // let circuit = CircuitLayer::from_seq(
    //     CircuitLayer::from_seq(
    //         CircuitSeq::new_from_file(circuit_file.as_path()),
    //         Layout::Dense,
    //     )
    //     .left_layout()
    //     .to_seq(),
    //     layout.clone(),
    // );

    let original_depth = circuit.cost(&Cost::Depth);
    let original_gates = circuit.cost(&Cost::Gate);
    let config = config.clone();
    let _ = ThreadPoolBuilder::new()
        .num_threads(config.n_threads)
        .build_global();
    let mut soam_runner = SoamRunner::new(config.to_owned(), 12345, circuit, layout.clone());
    let now = std::time::Instant::now();
    soam_runner.soam();
    let optimization_time = now.elapsed().as_secs_f32();
    soam_runner.oracle_runner.shutdown();
    let new_circuit = soam_runner.circ.clone();
    if DUMP {
        let str = new_circuit.to_seq().dump();
        let expanded_path = shellexpand::env(&config.circuit_path)
            .expect("failed to expand path")
            .into_owned();
        let file_name = expanded_path + ".eval";
        let mut file = File::create(file_name).unwrap();
        file.write_all(str.as_bytes()).unwrap();
    }
    if CORRECTNESS_CHECK {
        soam_runner.correctness_check(&new_circuit);
    }
    SingleResult {
        original_depth,
        optimized_depth: new_circuit.cost(&Cost::Depth),
        original_gates,
        optimized_gates: new_circuit.cost(&Cost::Gate),
        n_rounds: soam_runner.n_rounds,
        time: optimization_time,
        oracle_time: soam_runner.time_oracle,
        n_seams_total: soam_runner.n_seams_total,
    }
}

// fn single_test_range(config: &SingleConfig) -> SingleResult {
//     let circuit_file = PathBuf::from(config.circuit_path.clone());
//     let circuit = CircuitSeq::new_from_file(circuit_file.as_path());
//     let n_qubits = circuit.num_qubits;
//     let n_gates = circuit.gates.len();
//     let mut results: Vec<usize> = vec![0; n_gates];
//     for (i, gate) in circuit.gates.iter().enumerate() {
//         let qubits = gate.qubits();
//         let mut max = n_gates;
//         let mut min = 0;
//         for j in (i + 1)..n_gates {
//             if circuit.gates[j].qubits().iter().any(|q| qubits.contains(q)) {
//                 max = j;
//                 break;
//             }
//         }
//         for j in (0..i).rev() {
//             if circuit.gates[j].qubits().iter().any(|q| qubits.contains(q)) {
//                 min = j;
//                 break;
//             }
//         }
//         // println!("{:?}", qubits);
//         // println!("{} {}", min, max);
//         results[i] = max - min;
//     }
//     // println!("{:?}", results);
//     let expanded_path = shellexpand::env(&config.circuit_path)
//         .expect("failed to expand path")
//         .into_owned();
//     let file_name = expanded_path + ".txt";
//     let mut file = File::create(file_name).unwrap();
//     // add commas between elements
//     for result in results {
//         file.write_all(result.to_string().as_bytes()).unwrap();
//         file.write_all(b",").unwrap();
//     }
//     SingleResult {
//         original_depth: 0,
//         optimized_depth: 0,
//         original_gates: 0,
//         optimized_gates: 0,
//         n_rounds: 0,
//         time: 0.0,
//         oracle_time: 0.0,
//         n_seams_total: 0,
//     }
// }

fn run(config_path: &String) {
    let config = MultipleConfigs::read_config(config_path);
    let single_configs = config.to_single_configs();

    let results: Vec<ConfigResult> = single_configs
        .iter()
        .map(|single_config| ConfigResult {
            config: single_config.clone(),
            result: single_run(single_config),
        })
        .collect();
    let results = MultipleResults { results };
    soam::results::write_results(config_path, &results);
}
