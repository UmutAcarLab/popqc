use crate::optimization::routines::interferes;
use circuit::CircuitSeq;
use circuit::Gate;

// TODO: Copy optimized version into smaller copy and then write to actual circuit
// Okay because its small enough even if not in place
//
// TODO: Only check each subcircuit region once

use std::collections::HashMap;
use std::hash::RandomState;

#[derive(Debug)]
struct SubcircuitSection {
    gates: Vec<usize>,
    anchor_point: usize,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct WireState {
    state: Vec<u8>
}

impl WireState {

    // Create new state for the start of a subcircuit
    fn new_from_wire(wire: usize, num_qubits: usize) -> Self {
        let mut state = vec![0; num_qubits];
        state[wire / 8] = 1 << (wire % 8);
        Self{ state }
    }

    // Create new state for CX action on qubit
    fn xor(first: &WireState, second: &WireState) -> Self {
        let state_len = first.state.len();
        let mut state = vec![0; state_len];
        for state_segment_index in 0..state_len {
            state[state_segment_index] = first.state[state_segment_index] ^ second.state[state_segment_index];
        }
        Self{ state }
    }

    // Create new state for X action on qubit
    fn flip_all(current_state: &WireState) -> Self {
        let state_len = current_state.state.len();
        let mut state = vec![0; state_len];
        for state_segment_index in 0..state_len - 1 {
            state[state_segment_index] = current_state.state[state_segment_index] ^ u8::MAX;
        }
        state[state_len - 1] = current_state.state[state_len - 1] ^ u8::MAX >> (8 - state_len % 8);
        Self{ state }
    }

}
            

pub fn merge_rotations(c: &mut CircuitSeq) {
    let mut c_len = c.gates.len();
    let mut gate_index = 0;
    // Maps qubits to (left termination point, right termination point) tuples
    let mut termination_points: HashMap<usize, (usize, usize), RandomState> = HashMap::new();
    // Vector of gates included in subcircuit
    let mut subcircuit: Vec<usize>;

    // TODO: This length will need to adapt to updates
    while gate_index < c_len {
        if let Gate::CX { q1, q2 } = c.gates[gate_index] {
            if (termination_points.contains_key(&q1) && termination_points.contains_key(&q2))
                && (termination_points[&q1].1 > gate_index
                    || termination_points[&q2].1 > gate_index)
            {
                gate_index += 1;
                continue;
            }
            // println!("CREATING SUBCIRCUIT AT INDEX {}", gate_index);
            (termination_points, subcircuit) = create_subcircuit(&c, gate_index);
            merge(c, &mut subcircuit, &termination_points);
            c_len = c.gates.len();
        }
        gate_index += 1;
    }
}

pub fn create_subcircuit(
    c: &CircuitSeq,
    start_index: usize,
) -> (HashMap<usize, (usize, usize), RandomState>, Vec<usize>) {
    // Subcircuit is a vector of the gate indicies of gates included in the subcircuit
    // This procedure follows from Nam's paper:
    // "Automated optimization of large quantum circuits with continuous parameters"

    // Maps qubits to included gates on that qubit
    let mut subcircuit: HashMap<usize, SubcircuitSection> = HashMap::new();

    let mut qubit_exploration_order: Vec<usize> = vec![];

    // Map qubits to the boundries of their section of the subcircuit
    let mut termination_points = HashMap::new();
    
    if let Gate::CX{q1, q2} = c.gates[start_index] {
        let mut anchor_points_queue = vec![(start_index, q1), (start_index, q2)];

        // Anchor points acts as a queue for other wires to explore
        while !anchor_points_queue.is_empty() {
            let (gate_index, qubit) = anchor_points_queue.remove(0);

            if subcircuit.contains_key(&qubit) {
                continue;
            } else {
                subcircuit.insert(qubit, SubcircuitSection{ gates: vec![gate_index], anchor_point: gate_index });
                qubit_exploration_order.push(qubit);
            }

            // Buidling in the forward direction
            let end_termination: usize = expand_subcircuit(c, &mut subcircuit, 
                gate_index, qubit, false, &mut anchor_points_queue);

            // Building in the reverse direction
            let start_termination: usize = expand_subcircuit(c, &mut subcircuit,
                gate_index, qubit, true, &mut anchor_points_queue);

            termination_points.insert(qubit, (start_termination, end_termination));

            // TODO: Check if we need this
            // anchor_points.sort_by(|a, b| {(a.0.abs_diff(start_index)).cmp(&b.0.abs_diff(start_index))});
        }


        // let subcircuit_vec: Vec<(usize, Vec<usize>)> = subcircuit.into_iter().collect();

        for section in subcircuit.values_mut() {
            section.gates.sort();
        }

        // Pruning needed to fix termination boundries
        // let pruning_start_index = subcircuit_vec.binary_search(&(start_index, c.gates[start_index].qubits()));
        // println!("Circuit: {:?}", c.gates);

        // println!("termination points before pruning: {:?}", termination_points);
        // println!("Subcircuit before pruning: {:?}", subcircuit);
        
        // By pruning in the same order that we explored, we can remove sections if their
        // "parent" section has been pruned out
        let mut repeat = true;
        let mut qubits_pruned_out = vec![];
        while repeat {
            repeat = false;
            for qubit in &qubit_exploration_order[..] {
                if qubits_pruned_out.contains(qubit) {
                    continue; 
                }
                if !subcircuit.contains_key(qubit) { continue; }
                if let Gate::CX{q1: control, q2: target} = c.gates[subcircuit[qubit].anchor_point] {
                    // Check if anchor point has been pruned out
                    let mut prune_entire_section = false;
                    // These two cases are to distinguish if the target or control are on the parent
                    // qubit
                    if control == *qubit {
                        if !subcircuit.contains_key(&target) {
                            prune_entire_section = true;
                        }
                        else if !subcircuit[&target].gates.contains(&subcircuit[qubit].anchor_point) {
                            prune_entire_section = true;
                        }
                    }
                    else if target == *qubit {
                        if !subcircuit.contains_key(&control) {
                            prune_entire_section = true;
                        }
                        else if !subcircuit[&control].gates.contains(&subcircuit[qubit].anchor_point) {
                            prune_entire_section = true;
                        }
                    }
                    if prune_entire_section {
                        subcircuit.remove(&qubit);
                        // termination_points.remove(&qubit);
                        // Nothing should be in range anymore
                        termination_points.insert(*qubit, (c.gates.len(), c.gates.len()));
                        repeat = true;
                        qubits_pruned_out.push(*qubit);
                        // println!("Pruning entire qubit {}", qubit);
                        continue;
                    }
                    if adjust_termination_points(&c, subcircuit.get_mut(&qubit).unwrap(), &mut termination_points) { repeat = true; }
                    prune_gates(*qubit, subcircuit.get_mut(&qubit).unwrap(), &termination_points);
                    // println!("Final section for qubit {}: {:?}", qubit, subcircuit[qubit].gates);
                }
                else {
                    panic!("Anchor point is not a CX gate");
                }
            }
        }
    }


    let mut subcircuit_vec: Vec<usize> = vec![];

    for section in subcircuit.values_mut() {
        subcircuit_vec.append(&mut section.gates);
    }

    subcircuit_vec.sort();
    subcircuit_vec.dedup();

    // println!("termination points: {:?}, subcircuit: {:?}", termination_points, subcircuit_vec);

    (termination_points, subcircuit_vec)
}

fn expand_subcircuit(c: &CircuitSeq, subcircuit: &mut HashMap<usize, SubcircuitSection>, start_index: usize, start_qubit: usize, reverse: bool, anchor_points: &mut Vec<(usize, usize)>) -> usize {
    let mut current_termination_point: usize = start_index;
    let section: &mut SubcircuitSection = subcircuit.get_mut(&start_qubit).unwrap();
    if !reverse {
        let index_range = start_index + 1..c.gates.len();
        for gate_index in index_range {
            if interferes(c.gates[gate_index].clone(), start_qubit) {
                if !matches!(c.gates[gate_index], Gate::H(_)) {
                    current_termination_point = gate_index
                }
                match c.gates[gate_index] {
                    Gate::CX { q1, q2 } => {
                        if q1 == start_qubit {
                            anchor_points.push((gate_index, q2));
                        } else {
                            anchor_points.push((gate_index, q1))
                        }
                        section.gates.push(gate_index);

                    },
                    Gate::H(_) => {
                        return current_termination_point;
                    },
                    _ => {
                        section.gates.push(gate_index);
                    }
                }
            }
        }
    }
    else {
        //TODO: Can we do start_index - 1 here, or is it exclusive?
        let index_range = (0..start_index).rev();

        for gate_index in index_range {
            if interferes(c.gates[gate_index].clone(), start_qubit) {
                if !matches!(c.gates[gate_index], Gate::H(_)) {
                    current_termination_point = gate_index
                }
                match c.gates[gate_index] {
                    Gate::CX { q1, q2 } => {
                        if q1 == start_qubit {
                            anchor_points.push((gate_index, q2));
                        } else {
                            anchor_points.push((gate_index, q1))
                        }
                        section.gates.push(gate_index);

                    },
                    Gate::H(_) => {
                        return current_termination_point;
                    },
                    _ => {
                        section.gates.push(gate_index);
                    }
                }
            }
        }
    }

    if reverse {
        0
    } else {
        c.gates.len() - 1
    }
}

fn adjust_termination_points(c: &CircuitSeq, section: &mut SubcircuitSection, termination_points: &mut HashMap<usize, (usize, usize), RandomState>) -> bool {
    let mut section_index = 0;

    // CXs to remove where target is outside termination points
    let mut to_remove: Vec<usize> = vec![];

    let mut repeat = false;

    while section_index < section.gates.len() {
        let gate_index = section.gates[section_index];
        match c.gates[gate_index] {
            Gate::CX{q1: control, q2: target} => {
                // Do nothing in this case
                if !(gate_index < termination_points[&control].0 || gate_index > termination_points[&control].1)
                    && !(gate_index < termination_points[&target].0 || gate_index > termination_points[&target].1) {
                }

                // Control is outside termination points boundry
                else if gate_index < termination_points[&control].0 || gate_index > termination_points[&control].1{
                    if gate_index < section.anchor_point {
                        termination_points.insert(target, (gate_index + 1, termination_points[&target].1));
                        repeat = true;
                    }else {
                        termination_points.insert(target, (termination_points[&target].0, gate_index - 1));
                        repeat = true;
                        break
                    }
                } 

                // Target is outside termination points boundry: Also do nothing here
                else if gate_index < termination_points[&target].0 || gate_index > termination_points[&target].1 {
                    to_remove.push(section_index);
                }
            },
            _ => {}
        }
        section_index += 1;
    }

    for removal_index in to_remove.into_iter().rev() {
        section.gates.remove(removal_index);
    }
    repeat
}

fn prune_gates(qubit: usize, section: &mut SubcircuitSection, termination_points: &HashMap<usize, (usize, usize), RandomState>) {
    let mut section_index = 0;

    while section_index < section.gates.len() {
        let gate_index = section.gates[section_index];
        if gate_index < termination_points[&qubit].0 || gate_index > termination_points[&qubit].1 {
            section.gates.remove(section_index);
        }
        else {
            section_index += 1;
        }
    }


}

pub fn merge(c: &mut CircuitSeq, subcircuit: &mut Vec<usize>, termination_points: &HashMap<usize, (usize, usize), RandomState>) {

    //Map binary states to (location, rotation, wire)
    let mut rotation_merge_dict: HashMap<WireState, (usize, f64, usize), RandomState> = HashMap::new();

    //Map wire to state
    let mut state_dict: HashMap<usize, WireState, RandomState> = HashMap::new();

    let mut subcircuit_start = c.gates.len();
    let mut subcircuit_end = 0;
    for point in termination_points.values() {
        if point.0 != c.gates.len() && point.0 < subcircuit_start { subcircuit_start = point.0; }
        if point.1 != c.gates.len() && point.1 > subcircuit_end { subcircuit_end = point.1; }
    }

    for wire in termination_points.keys() {
        //check if qubit has been pruned out
        if termination_points[wire].0 != c.gates.len() {
            let state: WireState = WireState::new_from_wire(*wire, c.num_qubits);
            state_dict.insert(*wire, state.clone());
            rotation_merge_dict.insert(state.clone(), (termination_points[wire].0, 0.0, *wire));
        }
    }

    let mut subcircuit_copy = CircuitSeq {
        num_qubits: c.num_qubits,
        gates: c.gates[subcircuit_start..subcircuit_end + 1].to_vec(),
    };

    for gate_index in subcircuit {
        // println!("Gate index: {}", gate_index);
        // Why do we need to dereference here?
        match c.gates[*gate_index] {
            // CX gates alter the state on the wire
            Gate::CX{q1: control, q2: target} => {
                state_dict.insert(target, WireState::xor(&state_dict[&target], &state_dict[&control]));
                if !rotation_merge_dict.contains_key(&state_dict[&target]) {
                    rotation_merge_dict.insert(state_dict[&target].clone(), (*gate_index + 1, 0.0, target));
                }
            }

            // RZ gates contribute to the rotation on the current state of the wire
            Gate::RZ{param1: k, q1: q} => {
                rotation_merge_dict.insert(state_dict[&q].clone(), (rotation_merge_dict[&state_dict[&q]].0, rotation_merge_dict[&state_dict[&q]].1 + k, rotation_merge_dict[&state_dict[&q]].2));
                subcircuit_copy.gates[*gate_index - subcircuit_start] = Gate::B;
            }

            // X gate flips that bit of the state
            Gate::X(q) => {
                state_dict.insert(q, WireState::flip_all(&state_dict[&q]));
                if !rotation_merge_dict.contains_key(&state_dict[&q]) {
                    rotation_merge_dict.insert(state_dict[&q].clone(), (*gate_index + 1, 0.0, q));
                }
            }
            _ => {
                // println!("Gate: {:?}", c.gates[*gate_index]);
            }
        }
    }
    // println!("dict: {:?}", rotation_merge_dict);

    let mut rotations_to_add: Vec<(usize, f64, usize)> =
        rotation_merge_dict.values().cloned().collect();

    // sort by reversed order in gates
    rotations_to_add.sort_by(|a, b| (b.0).cmp(&a.0));

    for rotation in rotations_to_add {
        if rotation.1 != 0.0 {
            subcircuit_copy.gates.insert(
                rotation.0 - subcircuit_start,
                Gate::RZ {
                    param1: rotation.1,
                    q1: rotation.2,
                },
            );
        }
    }

    subcircuit_copy.remove_identities();

    for gate_index in subcircuit_start..subcircuit_end + 1 {
        if gate_index - subcircuit_start >= subcircuit_copy.gates.len() {
            c.gates[gate_index] = Gate::B;
        } else {
            c.gates[gate_index] = subcircuit_copy.gates[gate_index - subcircuit_start].clone();
        }
    }
}

// #[test]
pub fn create_subcircuit_1() {
    let mut c = CircuitSeq {
        gates: vec![
            Gate::H(0),
            Gate::H(1),
            Gate::H(2),
            Gate::RZ { param1: 1.0, q1: 1 },
            Gate::RZ { param1: 1.0, q1: 2 },
            Gate::CX { q1: 1, q2: 0 },
            Gate::RZ { param1: 1.0, q1: 0 },
            Gate::CX { q1: 1, q2: 2 },
            Gate::CX { q1: 0, q2: 1 },
            Gate::H(2),
            Gate::CX { q1: 1, q2: 2 },
            Gate::CX { q1: 0, q2: 1 },
            Gate::RZ { param1: 1.0, q1: 1 },
            Gate::H(0),
            Gate::H(1),
        ],
        num_qubits: 3,
    };

    let (termination_points, mut subcircuit) = create_subcircuit(&c, 5);

    assert_eq!(subcircuit, vec![3, 4, 5, 6, 7, 8, 11, 12]);

    merge(&mut c, &mut subcircuit, &termination_points);
    c.remove_identities();
    println!("Gates: {:?}", c.gates);

    assert_eq!(c.gates.len(), 14);

    assert_eq!(
        c.gates,
        vec![
            Gate::H(0),
            Gate::H(1),
            Gate::RZ { param1: 2.0, q1: 1 },
            Gate::H(2),
            Gate::RZ { param1: 1.0, q1: 2 },
            Gate::CX { q1: 1, q2: 0 },
            Gate::RZ { param1: 1.0, q1: 0 },
            Gate::CX { q1: 1, q2: 2 },
            Gate::CX { q1: 0, q2: 1 },
            Gate::H(2),
            Gate::CX { q1: 1, q2: 2 },
            Gate::CX { q1: 0, q2: 1 },
            Gate::H(0),
            Gate::H(1),
        ]
    );
}
