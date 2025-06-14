use crate::config::{Cost, Gateset};
use crate::CircuitSeq;
use crate::Gate;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug)]
pub struct Layer {
    pub gates: Vec<Gate>,
}
impl Layer {
    pub fn new(gates: Vec<Gate>) -> Self {
        Self { gates }
    }
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]

pub enum Layout {
    Dense,
    One,
}
#[derive(Clone, Debug)]
pub struct CircuitLayer {
    pub num_qubits: usize,
    pub layers: Vec<Layer>,
    pub layout: Layout,
}

impl CircuitLayer {
    pub fn new(gates: Vec<Gate>, num_qubits: usize, layout: Layout) -> Self {
        match layout {
            Layout::Dense => {
                let mut layer_idx = vec![0; num_qubits];
                let mut layers = Vec::<Layer>::new();
                for gate in gates {
                    let qubits = gate.qubits();
                    let max_layer_idx = qubits.iter().map(|qubit| layer_idx[*qubit]).max().unwrap();
                    if max_layer_idx >= layers.len() {
                        layers.push(Layer { gates: Vec::new() });
                    }
                    for qubit in qubits {
                        layer_idx[qubit] = max_layer_idx + 1;
                    }
                    layers[max_layer_idx].gates.push(gate);
                }
                Self {
                    layers,
                    num_qubits,
                    layout: Layout::Dense,
                }
            }
            Layout::One => {
                let mut layers = Vec::<Layer>::new();
                for gate in gates {
                    layers.push(Layer { gates: vec![gate] });
                }
                Self {
                    layers,
                    num_qubits,
                    layout: Layout::One,
                }
            }
        }
    }
    pub fn len(&self) -> usize {
        // including empty layers
        self.layers.len()
    }
    pub fn get(&self, start: usize, end: usize) -> Self {
        Self {
            num_qubits: self.num_qubits,
            layers: self.layers[start..end].to_vec(),
            layout: self.layout.clone(),
        }
    }
    pub fn par_set(&mut self, index: Vec<(usize, Vec<Gate>)>) {
        let ptr_as_usize = self.layers.as_mut_ptr() as usize;
        index.par_iter().for_each(|(i, v)| {
            let ptr = ptr_as_usize as *mut Layer;
            unsafe {
                *ptr.add(*i) = Layer { gates: v.clone() };
            }
        });
    }
    pub fn cost(&self, cost: &Cost) -> usize {
        match self.layout {
            Layout::Dense => match cost {
                Cost::Depth => self.depth(),
                Cost::Gate => self.gate_count(),
                Cost::Mixed => 10 * self.depth() + self.gate_count(),
            },
            Layout::One => match cost {
                Cost::Depth => 0,
                Cost::Gate => self.gate_count(),
                Cost::Mixed => 0,
            },
        }
    }
    pub fn to_seq(&self) -> CircuitSeq {
        CircuitSeq::new(
            self.layers
                .iter()
                .flat_map(|layer| layer.gates.clone())
                .collect(),
            self.num_qubits,
        )
    }
    pub fn from_seq(seq: CircuitSeq, layout: Layout) -> Self {
        Self::new(seq.gates, seq.num_qubits, layout)
    }
    pub fn is_empty(&self, id: usize) -> bool {
        self.layers[id].is_empty()
    }
    pub fn get_one(&self, id: usize) -> Vec<Gate> {
        self.layers[id].gates.clone()
    }
    pub fn gate_count(&self) -> usize {
        self.layers.iter().map(|layer| layer.gates.len()).sum()
    }
    pub fn depth(&self) -> usize {
        //excluding empty layers
        self.layers.iter().filter(|layer| !layer.is_empty()).count()
    }
    pub fn gate_count_rz(&self) -> usize {
        self.layers
            .iter()
            .map(|layer| {
                layer
                    .gates
                    .iter()
                    .map(|gate| match gate {
                        Gate::CCZ { .. } => 13,
                        _ => 1,
                    })
                    .sum::<usize>() // Add type annotation here
            })
            .sum()
    }
    // pub fn depth_count_rz(&self) -> usize {
    //     let new_gates: Vec<Gate> = self
    //         .layers
    //         .iter()
    //         .flat_map(|layer| {
    //             layer
    //                 .gates
    //                 .iter()
    //                 .flat_map(|gate| match gate {
    //                     Gate::CCZ { q1, q2, q3 } => {
    //                         vec![
    //                             Gate::CX { q1: *q1, q2: *q2 },
    //                             Gate::Tdg(*q3),
    //                             Gate::CX { q1: *q1, q2: *q3 },
    //                             Gate::T(*q3),
    //                             Gate::CX { q1: *q2, q2: *q3 },
    //                             Gate::Tdg(*q3),
    //                             Gate::CX { q1: *q1, q2: *q3 },
    //                             Gate::T(*q3),
    //                             Gate::T(*q2),
    //                             Gate::CX { q1: *q1, q2: *q2 },
    //                             Gate::T(*q1),
    //                             Gate::Tdg(*q2),
    //                             Gate::CX { q1: *q1, q2: *q2 },
    //                         ]
    //                     }
    //                     _ => vec![gate.clone()],
    //                 })
    //                 .collect::<Vec<Gate>>()
    //         })
    //         .collect();

    //     let new_circuit = CircuitLayer::new(new_gates, self.num_qubits);
    //     new_circuit.len()
    // }

    // #[allow(dead_code)]
    // pub fn left_layout(&self) -> CircuitLayer {
    //     let mut layer_idx = vec![0; self.num_qubits];
    //     let mut layers = Vec::<Layer>::new();
    //     for old_layer in &self.layers {
    //         for gate in old_layer.gates.iter().cloned() {
    //             let qubits = gate.qubits();
    //             let max_layer_idx = qubits.iter().map(|qubit| layer_idx[*qubit]).max().unwrap();
    //             if max_layer_idx >= layers.len() {
    //                 layers.push(Layer { gates: Vec::new() });
    //             }
    //             for qubit in qubits {
    //                 layer_idx[qubit] = max_layer_idx + 1;
    //             }
    //             layers[max_layer_idx].gates.push(gate);
    //         }
    //     }
    //     CircuitLayer {
    //         num_qubits: self.num_qubits,
    //         layers,
    //     }
    // }
    // #[allow(dead_code)]
    // pub fn right_layout(&self) -> CircuitLayer {
    //     let mut layer_idx = vec![0; self.num_qubits];
    //     let mut layers = Vec::<Layer>::new();
    //     for old_layer in self.layers.iter().rev() {
    //         for gate in old_layer.gates.iter().cloned() {
    //             let qubits = gate.qubits();
    //             let max_layer_idx = qubits.iter().map(|qubit| layer_idx[*qubit]).max().unwrap();
    //             if max_layer_idx >= layers.len() {
    //                 layers.push(Layer { gates: Vec::new() });
    //             }
    //             for qubit in qubits {
    //                 layer_idx[qubit] = max_layer_idx + 1;
    //             }
    //             layers[max_layer_idx].gates.push(gate);
    //         }
    //     }
    //     CircuitLayer {
    //         num_qubits: self.num_qubits,
    //         layers: layers.into_iter().rev().collect_vec(),
    //     }
    // }
    pub fn get_gateset(&self) -> Gateset {
        let mut gateset = Gateset::Nam;
        for layer in &self.layers {
            for gate in &layer.gates {
                match &gate {
                    Gate::CCZ { .. } | Gate::CCX { .. } => {
                        gateset = Gateset::CliffordT;
                        break;
                    }
                    _ => {}
                }
            }
        }
        gateset
    }
}
