use circuit::CircuitSeq;
use circuit::Gate;

use super::{
    cx_prop_and_cancel::{print_cx_stats, two_qubit_prop},
    hadamard_prop_and_cancel::{h_cancellation, h_propagation, print_h_stats},
    rotation_merge::merge_rotations,
    x_prop_and_cancel::{print_x_stats, x_cancellation, x_propagation},
    z_prop_and_cancel::{print_rz_stats, single_qubit_prop},
};

// Not Propagation
pub fn opt_routine_0(c: &mut CircuitSeq) {
    let mut circuit_len = c.gates.len();
    let mut gate_index = 0;
    while gate_index < circuit_len {
        if let Gate::X(index) = c.gates[gate_index] {
            if !x_propagation(c, gate_index, index) {
                gate_index += 1;
            }
        }
        else {
            gate_index += 1;
        }
        circuit_len = c.gates.len();
    }
    // *c = x_cancellation(c);
}

//Hadamard Reduction
pub fn opt_routine_1(c: &mut CircuitSeq) {
    let mut circuit_len = c.gates.len();
    let mut gate_index = 0;
    while gate_index < circuit_len {
        if let Gate::H(index) = c.gates[gate_index] {
            h_propagation(c, gate_index, index);
            circuit_len = c.gates.len();
        }
        gate_index += 1;
    }
    // *c = h_cancellation(c);
}

// Single-Qubit Gate Cancellation
pub fn opt_routine_2(c: &mut CircuitSeq) {
    let mut circuit_len = c.gates.len();
    let mut gate_index = 0;
    while gate_index < circuit_len {
        if let Gate::RZ {
            q1: index,
            param1: _,
        } = c.gates[gate_index]
        {
            if single_qubit_prop(c, gate_index, index) {
                circuit_len = c.gates.len();
            }
        }
        gate_index += 1;
    }
}

// Twp-Qubit Gate Cancellation
pub fn opt_routine_3(c: &mut CircuitSeq) {
    let mut circuit_len = c.gates.len();
    let mut gate_index = 0;
    while gate_index < circuit_len {
        if let Gate::CX { q1, q2 } = c.gates[gate_index] {
            if two_qubit_prop(c, gate_index, q1, q2) {
                circuit_len = c.gates.len();
            }
        }
        gate_index += 1;
    }
}

// Rotation Merging
pub fn opt_routine_4(c: &mut CircuitSeq) {
    merge_rotations(c);
}

pub unsafe fn print_statistics() {
    print_x_stats();
    print_h_stats();
    print_rz_stats();
    print_cx_stats();
}

pub fn interferes(gate: Gate, q: usize) -> bool {
    match gate {
        Gate::CCX { q1, q2, q3 } => q1 == q || q2 == q || q3 == q,
        Gate::CCZ { q1, q2, q3 } => q1 == q || q2 == q || q3 == q,
        Gate::CX { q1, q2 } => q1 == q || q2 == q,
        Gate::CZ { q1, q2 } => q1 == q || q2 == q,
        Gate::H(index) => index == q,
        Gate::X(index) => index == q,
        Gate::Y(index) => index == q,
        Gate::Z(index) => index == q,
        Gate::RX { q1, .. } => q1 == q,
        Gate::RY { q1, .. } => q1 == q,
        Gate::RZ { q1, .. } => q1 == q,
        Gate::S(index) => index == q,
        Gate::Sdg(index) => index == q,
        Gate::SqrtX(index) => index == q,
        Gate::SqrtXdg(index) => index == q,
        Gate::Swap { q1, q2 } => q1 == q || q2 == q,
        Gate::T(index) => index == q,
        Gate::Tdg(index) => index == q,
        Gate::B => false,
        _ => false,
    }
}
