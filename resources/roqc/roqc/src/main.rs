use std::env;
use std::fs;
use std::time::SystemTime;

mod optimization;
use circuit::{parse_program, write_program};

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    let circuit_from_file = fs::read_to_string(filename).expect("Error reading circuit file");

    let mut c = parse_program(&circuit_from_file);

    println!("Original length: {}", c.gates.len());

    println!("Starting Optimization");
    let now = SystemTime::now();

    optimization::optimizer::optimize_circuit(&mut c);
    match now.elapsed() {
        Ok(elapsed) => println!("Optimization took {} seconds", elapsed.as_secs_f32()),
        Err(e) => println!("Error: {e:?}"),
    }
    println!("Optimized length: {}", c.gates.len());

    let output_filename = format!("{}.roqc", filename);
    println!("Writing optimized circuit to {}", output_filename);

    write_program(&c.gates, c.num_qubits, output_filename)
        .expect("Unable to write optimized circuit to file");
}
