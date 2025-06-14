use circuit::CircuitSeq;
use circuit::Gate;
use std::f64::consts;

use super::routines::interferes;

static mut NUM_LOOPS: usize = 0;
static mut NUM_CANCELS: isize = 0;

pub unsafe fn print_x_stats() {
    println!("X Loops: {}", NUM_LOOPS);
    println!("X Cancels: {}", NUM_CANCELS);
}

pub fn x_propagation(c: &mut CircuitSeq, x_index: usize, x_q: usize) -> bool {
    println!("x_q is {}, x_index is {}", x_q, x_index);
    for gate_index in (x_index + 1)..(c.gates.len()) {
        unsafe {
            NUM_LOOPS += 1;
        }
        match c.gates[gate_index] {
            Gate::X(q) => {
                if q == x_q {
                    c.gates[gate_index] = Gate::B;
                    c.gates[x_index] = Gate::B;
                    return true;
                }
            }
            Gate::H(q) => {
                println!("Reached this with x_index: {} and gate index: {}", x_index, gate_index);
                if q == x_q {
                    println!("Made it through check");
                    c.gates[gate_index] = Gate::RZ {
                        param1: consts::PI,
                        q1: q,
                    };
                    c.gates[x_index] = Gate::H(x_q);
                    return true;
                }
            }
            Gate::RZ { param1: k, q1: q } => {
                if q == x_q {
                    c.gates[x_index] = Gate::RZ {
                        param1: (2.0 * consts::PI) - k,
                        q1: q,
                    };
                    c.gates[gate_index] = Gate::X(q);
                    return true;
                }
            }
            Gate::CX { q1, q2 } => {
                if q1 == x_q {
                    c.gates.insert(gate_index + 1, Gate::X(q1));
                    c.gates.insert(gate_index + 2, Gate::X(q2));
                    c.gates.remove(x_index);
                    return true;
                }
                if q2 == x_q {
                    c.gates.insert(gate_index + 1, Gate::X(q2));
                    c.gates.remove(x_index);
                    return true;
                }
            }
            _ => {
                if interferes(c.gates[gate_index].clone(), x_q) {
                    return false;
                }
            }
        };
    }
    return false;
}

// TODO: See if this is actually neccesary
pub fn x_cancellation(c: &CircuitSeq) -> CircuitSeq {
    let mut result = CircuitSeq {
        gates: vec![],
        num_qubits: c.num_qubits,
    };
    let mut end_with_x_gate = vec![false; c.num_qubits]; // X gates at the end
    for g in &c.gates {
        // take ownership of the elements
        if let Gate::X(ref index) = g {
            // if X gate, flip the state at the end
            end_with_x_gate[*index] = !end_with_x_gate[*index];
        } else {
            // if not X gate, push all X gates sharing a qubit with this gate before pushing this gate
            for i in g.qubits() {
                if end_with_x_gate[i] {
                    result.gates.push(Gate::X(i));
                    end_with_x_gate[i] = false;
                }
            }
            result.gates.push(g.clone());
        }
    }
    for i in 0..c.num_qubits {
        if end_with_x_gate[i] {
            result.gates.push(Gate::X(i));
        }
    }

    unsafe {
        NUM_CANCELS +=
            isize::try_from(c.gates.len()).unwrap() - isize::try_from(result.gates.len()).unwrap();
    }

    result
}
