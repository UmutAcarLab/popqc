use std::collections::HashMap;
use std::collections::HashSet;
use std::f64::consts;

use circuit::CircuitSeq;
use circuit::Gate;


use super::routines::interferes;

static mut NUM_LOOPS: usize = 0;
static mut NUM_CANCELS: usize = 0;

pub unsafe fn print_h_stats() {
    println!("H Loops: {}", NUM_LOOPS);
    println!("H Cancels: {}", NUM_CANCELS);
}

pub fn h_propagation(c: &mut CircuitSeq, h_index: usize, h_q: usize) {
    let mut seen_gates: Vec<Gate> = vec![Gate::H(h_q)];
    let mut seen_idxs: Vec<usize> = vec![h_index];

    //Additional data needed for 5th reduction:
    let mut had_before_cx_idxs: HashMap<usize, usize> = HashMap::new();
    let mut cx_seen: bool = false;
    let mut invalid_qubits: HashSet<usize> = HashSet::new();
    let mut had_after_cx_idxs: HashMap<usize, usize> = HashMap::new();

    for gate_index in (h_index + 1)..(c.gates.len()) {
        unsafe {
            NUM_LOOPS += 1;
        }
        let mut first_cx = false;
        if interferes(c.gates[gate_index].clone(), h_q) {
            seen_gates.push(c.gates[gate_index].clone());
            seen_idxs.push(gate_index);
            if matches!(c.gates[gate_index], Gate::CX { q1: _, q2: _ }) {
                if cx_seen == false {
                    first_cx = true;
                }
                cx_seen = true;
            }
        } else if let Gate::H(q) = c.gates[gate_index] {
            match cx_seen {
                true => {
                    if (!invalid_qubits.contains(&q)) && (!had_after_cx_idxs.contains_key(&q)) {
                        had_after_cx_idxs.insert(q, gate_index);
                    }
                }
                false => {
                    had_before_cx_idxs.insert(q, gate_index);
                }
            }
        }

        // See if we need to remove hadamard gates from contention for the last pattern
        if (!first_cx) && (!matches!(c.gates[gate_index], Gate::H(_))) {
            if !cx_seen {
                let mut removal_qubits = vec![];
                for qubit in had_before_cx_idxs.keys() {
                    if interferes(c.gates[gate_index].clone(), *qubit) {
                        removal_qubits.push(*qubit);
                    }
                }
                for qubit in removal_qubits {
                    had_before_cx_idxs.remove(&qubit);
                }
            } else {
                for qubit in c.gates[gate_index].qubits() {
                    invalid_qubits.insert(qubit);
                }
            }
        }

        if seen_gates.len() > 5 {
            return;
        }
        match seen_gates[..] {
            [Gate::H(_), Gate::H(_)] => {
                c.gates[seen_idxs[0]] = Gate::B;
                c.gates[seen_idxs[1]] = Gate::B;
                return;
            }
            [Gate::H(_), Gate::RZ { param1: r, q1: _ }, Gate::H(_)] => {
                if r == 0.5*consts::PI {
                    c.gates[seen_idxs[0]] = Gate::RZ {
                        param1: 1.5*consts::PI,
                        q1: h_q,
                    };
                    c.gates[seen_idxs[1]] = Gate::H(h_q);
                    c.gates[seen_idxs[2]] = Gate::RZ {
                        param1: 1.5*consts::PI,
                        q1: h_q,
                    };
                }
                else if r == 1.5*consts::PI {
                    c.gates[seen_idxs[0]] = Gate::RZ {
                        param1: 0.5*consts::PI,
                        q1: h_q,
                    };
                    c.gates[seen_idxs[1]] = Gate::H(h_q);
                    c.gates[seen_idxs[2]] = Gate::RZ {
                        param1: 0.5*consts::PI,
                        q1: h_q,
                    };
                    return;
                }
            }
            // These cases are not complete becuase of target/interference
            [Gate::H(_), Gate::RZ { param1: r1, q1: _ }, Gate::CX {
                q1: control,
                q2: target,
            }, Gate::RZ { param1: r2, q1: _ }, Gate::H(_)] => {
                if target == h_q && r1 == 1.5*consts::PI && r2 == 0.5*consts::PI {
                    c.gates[seen_idxs[1]] = Gate::RZ {
                        param1: 0.5*consts::PI,
                        q1: h_q,
                    };
                    c.gates[seen_idxs[2]] = Gate::CX {
                        q1: control,
                        q2: h_q,
                    };
                    c.gates[seen_idxs[3]] = Gate::RZ {
                        param1: 1.5*consts::PI,
                        q1: h_q,
                    };
                    c.gates.remove(seen_idxs[0]);
                    c.gates.remove(seen_idxs[4] - 1);
                }
                else if target == h_q && r1 == 0.5*consts::PI && r2 == 1.5*consts::PI {
                    c.gates[seen_idxs[1]] = Gate::RZ {
                        param1: 1.5*consts::PI,
                        q1: h_q,
                    };
                    c.gates[seen_idxs[2]] = Gate::CX {
                        q1: control,
                        q2: h_q,
                    };
                    c.gates[seen_idxs[3]] = Gate::RZ {
                        param1: 0.5*consts::PI,
                        q1: h_q,
                    };
                    c.gates.remove(seen_idxs[0]);
                    c.gates.remove(seen_idxs[4] - 1);
                }
                return;
            }
            [Gate::H(_), Gate::CX {
                q1: control,
                q2: target,
            }, Gate::H(_)] => {
                if target == h_q {
                    let mut before_check = false;
                    let mut after_check = false;

                    for qubit in had_before_cx_idxs.keys() {
                        if *qubit == control {
                            before_check = true;
                        }
                    }
                    for qubit in had_after_cx_idxs.keys() {
                        if *qubit == control {
                            after_check = true;
                        }
                    }

                    if !(before_check == true && after_check == true) {
                        continue;
                    }

                    c.gates[seen_idxs[1]] = Gate::CX {
                        q1: h_q,
                        q2: control,
                    };

                    c.gates[seen_idxs[0]] = Gate::B;
                    c.gates[seen_idxs[2]] = Gate::B;
                    c.gates[had_before_cx_idxs[&control]] = Gate::B;
                    c.gates[had_after_cx_idxs[&control]] = Gate::B;

                    return;
                };
            }
            _ => {}
        }
    }
}

// TODO: See if this is actually neccesary
pub fn h_cancellation(c: &CircuitSeq) -> CircuitSeq {
    let mut result = CircuitSeq {
        gates: vec![],
        num_qubits: c.num_qubits,
    };
    let mut end_with_h_gate = vec![false; c.num_qubits]; // H gates at the end
    for g in &c.gates {
        // take ownership of the elements
        if let Gate::H(ref index) = g {
            // if H gate, flip the state at the end
            end_with_h_gate[*index] = !end_with_h_gate[*index];
        } else {
            // if not H gate, push all H gates sharing a qubit with this gate before pushing this gate
            for i in g.qubits() {
                if end_with_h_gate[i] {
                    result.gates.push(Gate::H(i));
                    end_with_h_gate[i] = false;
                }
            }
            result.gates.push(g.clone());
        }
    }
    for i in 0..c.num_qubits {
        if end_with_h_gate[i] {
            result.gates.push(Gate::H(i));
        }
    }

    unsafe {
        NUM_CANCELS += c.gates.len() - result.gates.len();
    }

    result
}
