use super::routines::interferes;
use circuit::CircuitSeq;
use circuit::Gate;

static mut NUM_LOOPS: usize = 0;
static mut NUM_CANCELS: usize = 0;

pub unsafe fn print_rz_stats() {
    println!("RZ Loops: {}", NUM_LOOPS);
    println!("RZ Cancels: {}", NUM_CANCELS);
}

enum PropagateStatus {
    PROPAGATE,
    CANCELLATION,
    REVERT,
}

struct PropagationResult {
    status: PropagateStatus,
    start_index: usize,
    shift_instr: (usize, usize),
}

pub fn single_qubit_prop(c: &mut CircuitSeq, g_index: usize, g_q: usize) -> bool {
    let mut gate_index: usize = g_index;
    let mut continue_propagation: bool = true;
    let mut reversal_instructions: Vec<(usize, usize)> = vec![];
    while continue_propagation {
        let result: PropagationResult = propagate(c, gate_index, g_q);
        match result.status {
            PropagateStatus::PROPAGATE => {
                gate_index = result.start_index;
                reversal_instructions.push(result.shift_instr);
            }
            PropagateStatus::CANCELLATION => {
                return true;
            }
            PropagateStatus::REVERT => {
                for instr in reversal_instructions.clone().into_iter().rev() {
                    c.shift_left(instr.0, instr.1);
                }
                continue_propagation = false;
            }
        }
    }
    false
}

fn propagate(c: &mut CircuitSeq, g_index: usize, g_q: usize) -> PropagationResult {
    let mut seen_gates: Vec<Gate> = vec![c.gates[g_index].clone()];
    let mut seen_idxs: Vec<usize> = vec![g_index];

    //Additional data needed for 2nd propagation:
    let mut cx_check_needed: bool = false;
    let mut skip_check: bool = true;
    let mut cx_control = 0;

    let mut length_2_check = false;
    let mut length_3_check = false;

    for gate_index in (g_index + 1)..(c.gates.len()) {
        if seen_gates.len() == 4 {
            return PropagationResult {
                status: PropagateStatus::REVERT,
                start_index: g_index,
                shift_instr: (0, 0),
            };
        }
        unsafe {
            NUM_LOOPS += 1;
        }
        // If the gate interferes with current qubit, add it to our pattern
        if interferes(c.gates[gate_index].clone(), g_q) {
            seen_gates.push(c.gates[gate_index].clone());
            seen_idxs.push(gate_index);

            // Check if we have seen a second RZ gate
            // This is later used to see which half of pattern 2 we are on
            if let Gate::CX {
                q1: control,
                q2: target,
            } = c.gates[gate_index]
            {
                if target == g_q {
                    cx_check_needed = true;
                    cx_control = control;
                }
            }
        }

        if !skip_check {
            if interferes(c.gates[gate_index].clone(), cx_control) {
                match c.gates[gate_index] {
                    Gate::CX { q1: _, q2: target } => {
                        if target == g_q && seen_gates.len() == 4 {
                            if let Gate::RZ { q1: _, param1: _ } = seen_gates[2] {
                            } else {
                                return PropagationResult {
                                    status: PropagateStatus::REVERT,
                                    start_index: g_index,
                                    shift_instr: (0, 0),
                                };
                            }
                        } else {
                            return PropagationResult {
                                status: PropagateStatus::REVERT,
                                start_index: g_index,
                                shift_instr: (0, 0),
                            };
                        }
                    }
                    _ => {
                        return PropagationResult {
                            status: PropagateStatus::REVERT,
                            start_index: g_index,
                            shift_instr: (0, 0),
                        };
                    }
                }
            }
        }
        if cx_check_needed {
            skip_check = false
        }

        // ==== Step 1: Check for cancellation rules ====

        // Cancelation rules are limited to two gates
        // TODO: This should be a helper function for use in 2 qubit prop, also check indicies
        if seen_gates.len() == 2 {
            match seen_gates[..] {
                [Gate::RZ { q1: _, param1: k1 }, Gate::RZ { q1: _, param1: k2 }] => {
                    unsafe {
                        NUM_CANCELS += 1;
                    }
                    c.gates[seen_idxs[0]] = Gate::RZ {
                        q1: g_q,
                        param1: k1 + k2,
                    };
                    c.gates[seen_idxs[1]] = Gate::B;
                    return PropagationResult {
                        status: PropagateStatus::CANCELLATION,
                        start_index: g_index,
                        shift_instr: (0, 0),
                    };
                }
                _ => {}
            }
        }

        // Longest propagation rule is length 4
        if !length_2_check && (seen_gates.len() == 2) {
            if early_termination_legnth_2(&seen_gates) {
                return PropagationResult {
                    status: PropagateStatus::REVERT,
                    start_index: g_index,
                    shift_instr: (0, 0),
                };
            } else {
                length_2_check = true;
            }
        }
        if !length_3_check && (seen_gates.len() == 3) {
            if early_termination_legnth_3(&seen_gates) {
                return PropagationResult {
                    status: PropagateStatus::REVERT,
                    start_index: g_index,
                    shift_instr: (0, 0),
                };
            } else {
                length_3_check = true;
            }
        }

        // ==== Step 2: Check for propagation rules ====
        match seen_gates[..] {
            [Gate::RZ { q1: _, param1: _ }, Gate::H(_), Gate::CX { q1: _, q2: target }, Gate::H(_)] => {
                if target == g_q {
                    c.shift_right(g_index, seen_idxs[3]);
                    return PropagationResult {
                        status: PropagateStatus::PROPAGATE,
                        start_index: seen_idxs[3],
                        shift_instr: (g_index, seen_idxs[3]),
                    };
                }
            }
            [Gate::RZ { q1: _, param1: _ }, Gate::CX {
                q1: control_1,
                q2: target_1,
            }, Gate::RZ { q1: _, param1: _ }, Gate::CX {
                q1: control_2,
                q2: target_2,
            }] => {
                // Some of these indicies may get confusing, but you need to account for the
                // shifted indicies from removing and inserting
                if (control_1 == control_2) && (target_1 == target_2) && (target_2 == g_q) {
                    c.shift_right(g_index, seen_idxs[3]);
                    return PropagationResult {
                        status: PropagateStatus::PROPAGATE,
                        start_index: seen_idxs[3],
                        shift_instr: (g_index, seen_idxs[3]),
                    };
                }
            }
            [Gate::RZ { q1: _, param1: _ }, Gate::CX { q1: control, q2: _ }] => {
                if control == g_q {
                    c.shift_right(g_index, seen_idxs[1]);
                    return PropagationResult {
                        status: PropagateStatus::PROPAGATE,
                        start_index: seen_idxs[1],
                        shift_instr: (g_index, seen_idxs[1]),
                    };
                }
            }
            _ => {}
        }
    }

    return PropagationResult {
        status: PropagateStatus::REVERT,
        start_index: g_index,
        shift_instr: (0, 0),
    };
}

fn early_termination_legnth_2(gates: &Vec<Gate>) -> bool {
    match gates[..] {
        [Gate::RZ { q1: _, param1: _ }, Gate::H(_)] => false,
        [Gate::RZ { q1: _, param1: _ }, Gate::CX { q1: _, q2: _ }] => false,
        _ => true,
    }
}

fn early_termination_legnth_3(gates: &Vec<Gate>) -> bool {
    match gates[..] {
        [Gate::RZ { q1: _, param1: _ }, Gate::H(_), Gate::CX { q1: _, q2: _ }] => false,
        [Gate::RZ { q1: _, param1: _ }, Gate::CX { q1: _, q2: _ }, Gate::RZ { q1: _, param1: _ }] => {
            false
        }
        _ => true,
    }
}
