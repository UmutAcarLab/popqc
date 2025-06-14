use super::qasm_parser::parse_program;
use crate::config::Cost;
use crate::Gate;
use rayon::prelude::*;
use shellexpand;
use std::f64::consts;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CircuitSeq {
    pub gates: Vec<Gate>,
    pub num_qubits: usize,
}

impl CircuitSeq {
    pub fn new(gates: Vec<Gate>, num_qubits: usize) -> Self {
        Self { gates, num_qubits }
    }
    pub fn len(&self) -> usize {
        self.gates.len()
    }
    pub fn get(&self, start: usize, end: usize) -> Self {
        Self {
            gates: self.gates[start..end]
                .to_vec()
                .iter()
                .filter(|gate| !matches!(gate, Gate::B))
                .cloned()
                .collect(),
            num_qubits: self.num_qubits,
        }
    }
    pub fn to_seq(&self) -> Self {
        Self {
            gates: self
                .gates
                .iter()
                .filter(|gate| !matches!(gate, Gate::B))
                .cloned()
                .collect(),
            num_qubits: self.num_qubits,
        }
    }
    pub fn cost(&self, cost: &Cost) -> usize {
        match cost {
            Cost::Gate => self
                .gates
                .iter()
                .filter(|gate| !matches!(gate, Gate::B))
                .count(),
            Cost::Depth => panic!("Depth cost is not supported for circuit seq"),
            Cost::Mixed => panic!("Mixed cost is not supported for circuit seq"),
        }
    }
    pub fn is_empty(&self, id: usize) -> bool {
        matches!(self.gates[id], Gate::B)
    }
    pub fn get_one(&self, id: usize) -> Vec<Gate> {
        vec![self.gates[id].clone()]
    }
    pub fn par_set(&mut self, index: Vec<(usize, Vec<Gate>)>) {
        let ptr_as_usize = self.gates.as_mut_ptr() as usize;
        index.par_iter().for_each(|(i, v)| {
            let ptr = ptr_as_usize as *mut Gate;
            unsafe {
                *ptr.add(*i) = v[0].clone();
            }
        });
    }
    pub fn remove_identities(&mut self) {
        let mut clean_gates: Vec<Gate> = vec![];
        for gate in &self.gates {
            match gate {
                Gate::B => {}
                Gate::RZ { q1: _, param1 } => {
                    if *param1 % (2.0 * consts::PI) != 0.0 {
                        clean_gates.push(gate.clone());
                    }
                }
                _ => clean_gates.push(gate.clone()),
            }
        }
        self.gates = clean_gates;
    }
    
    pub fn reduce_angles(&mut self) {
        println!("reducing angles");
        for gate_index in 0..self.gates.len() {
            match self.gates[gate_index] {
                Gate::RZ{q1: intial_q, param1: initial_param} => {
                    self.gates[gate_index] = Gate::RZ{q1: intial_q, param1: initial_param % (2.0*consts::PI)};
                }
                _ => {},
            }
        }
    }

    pub fn replace_z_gates(&mut self) {
        for gate_index in 0..self.gates.len() {
            match self.gates[gate_index] {
                Gate::Z(q) => {
                    self.gates[gate_index] = Gate::RZ {
                        param1: consts::PI,
                        q1: q,
                    }
                }
                _ => {}
            }
        }
    }

    pub fn print_gate_counts(&mut self) {
        let mut x_gate_count: usize = 0;
        let mut h_gate_count: usize = 0;
        let mut rz_gate_count: usize = 0;
        let mut cx_gate_count: usize = 0;
        for gate_index in 0..self.gates.len() {
            match self.gates[gate_index] {
                Gate::X(_) => {
                    x_gate_count += 1;
                }
                Gate::H(_) => {
                    h_gate_count += 1;
                }
                Gate::RZ { param1: _, q1: _ } => {
                    rz_gate_count += 1;
                }
                Gate::CX { q1: _, q2: _ } => {
                    cx_gate_count += 1;
                }
                Gate::Z(_) => {
                    panic!("Z gate found in final result");
                }
                _ => {}
            }
        }
        println!("X gates: {}", x_gate_count);
        println!("H gates: {}", h_gate_count);
        println!("RZ gates: {}", rz_gate_count);
        println!("CX gates: {}", cx_gate_count);
    }

    pub fn shift_right(&mut self, source: usize, dest: usize) {
        for i in source..dest {
            self.gates.swap(i, i + 1);
        }
    }

    pub fn shift_left(&mut self, source: usize, dest: usize) {
        for i in 0..(dest - source) {
            self.gates.swap(dest - i, dest - (i + 1));
        }
    }
    pub fn new_from_source(source: &str) -> Self {
        parse_program(source)
    }
    pub fn new_from_file(path: &Path) -> Self {
        //print more details
        let expanded_path = shellexpand::env(&path.to_string_lossy())
            .expect("failed to expand path")
            .into_owned();
        // println!("expanded_path: {:?}", expanded_path);
        let source = std::fs::read_to_string(expanded_path).expect("failed to read file");
        Self::new_from_source(&source)
    }

    fn dump_header(&self, writer: &mut impl Write) {
        writeln!(writer, "OPENQASM 2.0;").unwrap();
        writeln!(writer, "include \"qelib1.inc\";").unwrap();
        writeln!(writer, "qreg q[{}];", self.num_qubits).unwrap();
    }
    pub fn dump(&self) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        {
            self.dump_header(&mut buffer);
            for gate in self.gates.iter() {
                writeln!(buffer, "{};", gate).unwrap();
            }
        }
        String::from_utf8(buffer).unwrap()
    }
}
