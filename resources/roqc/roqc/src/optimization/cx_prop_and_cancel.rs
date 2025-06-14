use circuit::CircuitSeq;
use circuit::Gate;

use super::routines::interferes;
use ::std::collections::HashSet;

static mut NUM_LOOPS: usize = 0;
static mut NUM_CANCELS: usize = 0;

pub unsafe fn print_cx_stats() {
    println!("CX Loops: {}", NUM_LOOPS);
    println!("CX Cancels: {}", NUM_CANCELS);
}

enum PropagateStatus {
    PROPAGATE,
    CANCELLATION,
    REVERT,
}

struct PropagationResult {
    status: PropagateStatus,
    start_index: usize,
    shift_instr: (Vec<usize>, usize),
}

pub fn two_qubit_prop(c: &mut CircuitSeq, g_index: usize, control: usize, target: usize) -> bool {
    let mut gate_index: usize = g_index;
    let mut continue_propagation: bool = true;
    let mut reversal_instructions: Vec<(Vec<usize>, usize)> = vec![];
    while continue_propagation {
        let result: PropagationResult = propagate(c, gate_index);
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
                    revert_chain(c, &instr.0, instr.1);
                }
                continue_propagation = false;
            }
        }
    }
    false
}

fn propagate(
    c: &mut CircuitSeq,
    g_index: usize
) -> PropagationResult {

    let mut control: usize = 0;
    let mut target: usize = 0;

    if let Gate::CX {q1: c, q2: t} = c.gates[g_index] {
        control = c;
        target = t;
    } else { panic!("CX propagation on non CX gate"); }

    let mut seen_gates_trgt: Vec<Gate> = vec![c.gates[g_index].clone()];
    let mut seen_idxs_trgt: Vec<usize> = vec![g_index];
    let mut seen_gates_ctrl: Vec<Gate> = vec![c.gates[g_index].clone()];
    let mut seen_idxs_ctrl: Vec<usize> = vec![g_index];

    let mut cx_cancel_viable = true;

    for gate_index in (g_index + 1)..(c.gates.len()) {
        unsafe {
            NUM_LOOPS += 1;
        }
        // If the gate interferes with current qubit, add it to our pattern
        if interferes(c.gates[gate_index].clone(), target) {
            seen_gates_trgt.push(c.gates[gate_index].clone());
            seen_idxs_trgt.push(gate_index);
        }
        if interferes(c.gates[gate_index].clone(), control) {
            seen_gates_ctrl.push(c.gates[gate_index].clone());
            seen_idxs_ctrl.push(gate_index);
        }

        if interferes(c.gates[gate_index].clone(), control) {
            if let Gate::CX { q1, q2 } = c.gates[gate_index] {
                if (q1 != control) || (q2 != target) {
                    cx_cancel_viable = false;
                }
            } else {
                cx_cancel_viable = false;
            }
        }

        // Longest propagation rule is length 4
        if seen_gates_trgt.len() > 4 {
            return PropagationResult {
                status: PropagateStatus::REVERT,
                start_index: g_index,
                shift_instr: (vec![], 0),
            };
        }

        // ==== Step 1: Check for cancellation rules ====

        // Cancelation rules are limited to two gates
        // TODO: This should be a helper function for use in 2 qubit prop
        if seen_gates_trgt.len() == 2 {
            match seen_gates_trgt[..] {
                [Gate::CX { q1: _, q2: _ }, Gate::CX {
                    q1: control_2,
                    q2: target_2,
                }] => {
                    if (seen_idxs_ctrl.len() == 2) &&(control_2 == control) && (target == target_2) && cx_cancel_viable {
                        unsafe {
                            NUM_CANCELS += 1;
                        }
                        c.gates[seen_idxs_trgt[0]] = Gate::B;
                        c.gates[seen_idxs_trgt[1]] = Gate::B;
                        return PropagationResult {
                            status: PropagateStatus::CANCELLATION,
                            start_index: g_index,
                            shift_instr: (vec![], 0),
                        };
                    }
                }
                _ => {}
            }
        }

        // ==== Step 2: Check for propagation rules ====
        //


        match seen_gates_trgt[..] {
            [Gate::CX { q1: _, q2: _ }, Gate::CX {
                q1: control_2,
                q2: target_2,
            }] => {
                if target_2 == target {
                    let interference_chain = create_interference_chain(
                        &c,
                        g_index,
                        seen_idxs_trgt[1],
                        control,
                        target,
                        control_2,
                    );

                    if !interference_chain.is_empty() {
                        move_chain_to_back(c, &interference_chain, seen_idxs_trgt[1]);
                        // println!("new gates: {:?}", c.gates);
                        return PropagationResult {
                            status: PropagateStatus::PROPAGATE,
                            start_index: seen_idxs_trgt[1] - (interference_chain.len() - 1),
                            shift_instr: (interference_chain.clone(), seen_idxs_trgt[1]),
                        };
                    }
                }
            }
            [Gate::CX { q1: _, q2: _ }, Gate::H(_), Gate::CX {
                q1: control_2,
                q2: target_2,
            }, Gate::H(_)] => {
                if target == control_2 && control != target_2 {
                    let interference_chain: Vec<usize> = create_interference_chain(
                        &c,
                        seen_idxs_trgt[0],
                        seen_idxs_trgt[3],
                        control,
                        target,
                        target_2,
                    );
                    if !interference_chain.is_empty() {
                        move_chain_to_back(c, &interference_chain, seen_idxs_trgt[3]);
                        return PropagationResult {
                            status: PropagateStatus::PROPAGATE,
                            start_index: seen_idxs_trgt[3] - (interference_chain.len() - 1),
                            shift_instr: (interference_chain.clone(), seen_idxs_trgt[3]),
                        };
                    }
                }
            }
            _ => {}
        }

        match seen_gates_ctrl[..] {
            [Gate::CX { q1: _, q2: _ }, Gate::CX {
                q1: control_2,
                q2: target_2,
            }] => {
                if control_2 == control {
                    let interference_chain = create_interference_chain(
                        &c,
                        g_index,
                        seen_idxs_ctrl[1],
                        target,
                        control,
                        target_2,
                    );

                    if !interference_chain.is_empty() {
                        move_chain_to_back(c, &interference_chain, seen_idxs_ctrl[1]);
                        return PropagationResult {
                            status: PropagateStatus::PROPAGATE,
                            start_index: seen_idxs_ctrl[1] - (interference_chain.len() - 1),
                            shift_instr: (interference_chain.clone(), seen_idxs_ctrl[1]),
                        };
                    }
                }
            }
            _ => {}
        }
    }

    return PropagationResult {
        status: PropagateStatus::REVERT,
        start_index: g_index,
        shift_instr: (vec![], 0),
    };
}

fn move_chain_to_back(c: &mut CircuitSeq, chain: &Vec<usize>, end_index: usize) {
    for (i, inter_index) in chain[..].iter().enumerate() {
        c.shift_right(inter_index - i, end_index);
    }
}

fn revert_chain(c: &mut CircuitSeq, chain: &Vec<usize>, end_index: usize) {
    for (i, inter_index) in chain[..].iter().enumerate() {
        c.shift_left(*inter_index, (end_index - chain.len()) + 1 + i);
    }
}

fn create_interference_chain(
    c: &CircuitSeq,
    start: usize,
    end: usize,
    start_qubit: usize,
    pattern_qubit: usize,
    exclusion_qubit: usize,
) -> Vec<usize> {
    let mut chain: Vec<usize> = vec![start];
    let mut interfering_qubits: HashSet<usize> = HashSet::from([start_qubit]);

    for (index, gate) in c.gates[(start + 1)..end].iter().enumerate() {
        // If it intereferes with pattern, it would already be included in pattern match
        if interferes(gate.clone(), pattern_qubit) {
            continue;
        }

        if let Gate::CX {
            q1: control,
            q2: target,
        } = gate
        {
            if interfering_qubits.contains(control) || interfering_qubits.contains(target) {
                interfering_qubits.insert(*control);
                interfering_qubits.insert(*target);
            }
        }

        let qubit_clone = interfering_qubits.clone();
        for q in qubit_clone.iter() {
            if interferes(gate.clone(), *q) {
                chain.push(index + start + 1);
                for gate_qubit in gate.qubits() {
                    if (gate_qubit) == pattern_qubit || (gate_qubit == exclusion_qubit) {
                        return vec![];
                    }
                }
                break;
            }
        }
    }
    return chain;
}
