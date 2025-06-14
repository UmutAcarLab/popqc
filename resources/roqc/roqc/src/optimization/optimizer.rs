use crate::optimization::routines::{
    opt_routine_0, opt_routine_1, opt_routine_2, opt_routine_3, opt_routine_4, print_statistics,
};

use circuit::CircuitSeq;
use std::time::SystemTime;

#[allow(dead_code)]
pub enum OptimizationType {
    Light,
    Heavy,
    Voqc,
}

pub fn optimize_light(c: &mut CircuitSeq) -> isize {
    // let order = [1,3,2,3,1,2,4,3,2];
    let order = [0, 2, 1, 2, 0, 1, 3, 2, 1];
    optimize(c, &order)
}

pub fn optimize_heavy(c: &mut CircuitSeq) -> isize {
    // let order = [1,3,2,3,1,2,5];
    let order = [0, 2, 1, 2, 0, 1, 4];
    optimize(c, &order)
}

pub fn optimize_voqc(c: &mut CircuitSeq) -> isize {
    // let order = [1,3,2,3,1,2,5];
    let order = [0, 1, 3, 2, 3, 2, 1, 4, 3, 2];
    optimize(c, &order)
}

pub fn optimize(c: &mut CircuitSeq, order: &[u8]) -> isize {
    // Convert all Z gates into Rz
    c.replace_z_gates();
    let initial_len: isize = c.gates.len().try_into().unwrap();
    loop {
        for routine in order {
            println!("--- Running Routine #{routine}");
            let now = SystemTime::now();
            match routine {
                0 => opt_routine_0(c),
                1 => opt_routine_1(c),
                2 => opt_routine_2(c),
                3 => opt_routine_3(c),
                4 => opt_routine_4(c), // TODO: Find Error!!
                _ => continue,
            }
            c.remove_identities();
            println!("took {} seconds", now.elapsed().unwrap().as_secs_f32());
        }
        unsafe {
            print_statistics();
        }
        c.print_gate_counts();
        /// println!("Circuit: {:?}", c.gates);
        // c.reduce_angles();
        return initial_len - <usize as TryInto<isize>>::try_into(c.gates.len()).unwrap();
    }
}

pub fn optimize_circuit(c: &mut CircuitSeq) -> isize {
    let opt_type = OptimizationType::Voqc;

    match opt_type {
        OptimizationType::Light => optimize_light(c),
        OptimizationType::Heavy => optimize_heavy(c),
        OptimizationType::Voqc => optimize_voqc(c),
    }
}

#[test]
pub fn test_end_to_end_x_prop() {
    use circuit::Gate;
    let mut c = CircuitSeq {
        gates: vec![
            Gate::X(0),
            Gate::CX { q1: 0, q2: 1 },
            Gate::H(0),
            Gate::X(1),
        ],
        num_qubits: 2,
    };
    let benefit = optimize_light(&mut c);
    assert_eq!(benefit, 1);
}
