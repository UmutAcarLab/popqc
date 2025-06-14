#[cfg(test)]
mod tests {

    use circuit::{CircuitSeq, Gate};

    use roqc::optimization::routines::{
        opt_routine_0,
        opt_routine_1,
        opt_routine_2,
        opt_routine_3,
        // opt_routine_4,
    };

    use roqc::optimization::x_prop_and_cancel::x_cancellation;
    use std::f64::consts;

    #[test]
    pub fn test_x_cancellation() {
        let c = CircuitSeq {
            gates: vec![Gate::X(0), Gate::X(1), Gate::H(1), Gate::X(0)],
            num_qubits: 2,
        };
        let c = x_cancellation(&c);
        assert_eq!(c.gates.len(), 2);
    }

    #[test]
    pub fn x_propagation_1() {
        let mut c = CircuitSeq {
            gates: vec![Gate::X(0), Gate::H(0)],
            num_qubits: 1,
        };
        opt_routine_0(&mut c);
        assert_eq!(c.gates.len(), 2);
        assert_eq!(Gate::H(0), c.gates[0]);
        assert_eq!(Gate::RZ {param1: consts::PI, q1: 0}, c.gates[1]);
    }

    #[test]
    pub fn x_propagation_2() {
        let mut c = CircuitSeq {
            gates: vec![Gate::X(0), Gate::RZ { param1: 1.0, q1: 0 }],
            num_qubits: 1,
        };
        opt_routine_0(&mut c);
        assert_eq!(c.gates.len(), 2);
        assert_eq!(
            Gate::RZ {
                param1: 2.0 * consts::PI - 1.0,
                q1: 0
            },
            c.gates[0]
        );
        assert_eq!(Gate::X(0), c.gates[1]);
    }

    #[test]
    pub fn x_propagation_3() {
        let mut c = CircuitSeq {
            gates: vec![Gate::X(0), Gate::CX { q1: 0, q2: 1 }],
            num_qubits: 2,
        };
        opt_routine_0(&mut c);
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[0]);
        assert_eq!(Gate::X(0), c.gates[1]);
        assert_eq!(Gate::X(1), c.gates[2]);
    }

    #[test]
    pub fn x_propagation_4() {
        let mut c = CircuitSeq {
            gates: vec![Gate::X(1), Gate::CX { q1: 0, q2: 1 }],
            num_qubits: 2,
        };
        opt_routine_0(&mut c);
        assert_eq!(c.gates.len(), 2);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[0]);
        assert_eq!(Gate::X(1), c.gates[1]);
    }

    #[test]
    pub fn x_propagation_non_interference() {
        let mut c = CircuitSeq {
            gates: vec![Gate::X(0), Gate::RZ {param1: consts::PI, q1: 1}, Gate::H(0)],
            num_qubits: 2,
        };
        opt_routine_0(&mut c);
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::H(0), c.gates[0]);
        assert_eq!(Gate::RZ {param1: consts::PI, q1: 1}, c.gates[1]);
        assert_eq!(Gate::RZ {param1: consts::PI, q1: 0}, c.gates[2]);
    }

    #[test]
    pub fn x_propagation_5() {
        let mut c = CircuitSeq {
            gates: vec![Gate::X(0), Gate::RZ {param1: consts::PI, q1: 0}, Gate::H(0)],
            num_qubits: 1,
        };
        opt_routine_0(&mut c);
        println!("Gates: {:?}", c.gates);
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::RZ {param1: consts::PI, q1: 0}, c.gates[0]);
        assert_eq!(Gate::H(0), c.gates[1]);
        assert_eq!(Gate::RZ {param1: consts::PI, q1: 0}, c.gates[2]);
    }

    #[test]
    pub fn h_cancellation_test_1() {
        let mut c = CircuitSeq {
            gates: vec![Gate::H(0), Gate::RZ { param1: 0.5*consts::PI, q1: 0 }, Gate::H(0)],
            num_qubits: 1,
        };
        opt_routine_1(&mut c);
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::RZ { param1: 1.5*consts::PI, q1: 0 }, c.gates[0]);
        assert_eq!(Gate::H(0), c.gates[1]);
        assert_eq!(Gate::RZ { param1: 1.5*consts::PI, q1: 0 }, c.gates[2]);
    }

    #[test]
    pub fn h_cancellation_test_2() {
        let mut c = CircuitSeq {
            gates: vec![Gate::H(0), Gate::RZ { param1: 1.5*consts::PI, q1: 0 }, Gate::H(0)],
            num_qubits: 1,
        };
        opt_routine_1(&mut c);
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::RZ { param1: 0.5*consts::PI, q1: 0 }, c.gates[0]);
        assert_eq!(Gate::H(0), c.gates[1]);
        assert_eq!(Gate::RZ { param1: 0.5*consts::PI, q1: 0 }, c.gates[2]);
    }

    #[test]
    pub fn h_cancellation_test_3() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::H(0),
                Gate::RZ { param1: 1.5*consts::PI, q1: 0 },
                Gate::CX { q1: 1, q2: 0 },
                Gate::RZ { param1: 0.5*consts::PI, q1: 0 },
                Gate::H(0),
            ],
            num_qubits: 2,
        };
        opt_routine_1(&mut c);
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::RZ { param1: 0.5*consts::PI, q1: 0 }, c.gates[0]);
        assert_eq!(Gate::CX { q1: 1, q2: 0 }, c.gates[1]);
        assert_eq!(Gate::RZ { param1: 1.5*consts::PI, q1: 0 }, c.gates[2]);
    }

    #[test]
    pub fn h_cancellation_test_4() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::H(0),
                Gate::RZ { param1: 0.5*consts::PI, q1: 0 },
                Gate::CX { q1: 1, q2: 0 },
                Gate::RZ { param1: 1.5*consts::PI, q1: 0 },
                Gate::H(0),
            ],
            num_qubits: 2,
        };
        opt_routine_1(&mut c);
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::RZ { param1: 1.5*consts::PI, q1: 0 }, c.gates[0]);
        assert_eq!(Gate::CX { q1: 1, q2: 0 }, c.gates[1]);
        assert_eq!(Gate::RZ { param1: 0.5*consts::PI, q1: 0 }, c.gates[2]);
    }

    #[test]
    pub fn h_cancellation_test_5() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::H(0),
                Gate::H(1),
                Gate::CX { q1: 1, q2: 0 },
                Gate::H(0),
                Gate::H(1),
            ],
            num_qubits: 2,
        };
        opt_routine_1(&mut c);
        c.remove_identities();
        assert_eq!(c.gates.len(), 1);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[0]);
    }

    #[test]
    pub fn h_cancellation_test_6() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::H(0),
                Gate::H(2),
                Gate::H(1),
                Gate::CX { q1: 1, q2: 0 },
                Gate::H(0),
                Gate::H(1),
            ],
            num_qubits: 3,
        };
        opt_routine_1(&mut c);
        c.remove_identities();
        assert_eq!(c.gates.len(), 2);
        assert_eq!(Gate::H(2), c.gates[0]);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[1]);
    }

    #[test]
    pub fn rz_prop_cancel_1() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::RZ { q1: 1, param1: 1.0 },
                Gate::H(1),
                Gate::CX { q1: 0, q2: 1 },
                Gate::H(1),
                Gate::RZ { q1: 1, param1: 2.0 },
            ],
            num_qubits: 2,
        };
        opt_routine_2(&mut c);
        c.remove_identities();
        println!("Gates: {:?}", c.gates);
        assert_eq!(c.gates.len(), 4);
        assert_eq!(Gate::H(1), c.gates[0]);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[1]);
        assert_eq!(Gate::H(1), c.gates[2]);
        assert_eq!(Gate::RZ { q1: 1, param1: 3.0 }, c.gates[3]);
    }

    #[test]
    pub fn rz_prop_cancel_2() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::RZ { q1: 1, param1: 1.0 },
                Gate::CX { q1: 0, q2: 1 },
                Gate::RZ { q1: 1, param1: 1.5 },
                Gate::CX { q1: 0, q2: 1 },
                Gate::RZ { q1: 1, param1: 2.0 },
            ],
            num_qubits: 2,
        };
        opt_routine_2(&mut c);
        c.remove_identities();
        assert_eq!(c.gates.len(), 4);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[0]);
        assert_eq!(Gate::RZ { q1: 1, param1: 1.5 }, c.gates[1]);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[2]);
        assert_eq!(Gate::RZ { q1: 1, param1: 3.0 }, c.gates[3]);
    }

    #[test]
    pub fn rz_prop_cancel_3() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::RZ { q1: 0, param1: 1.0 },
                Gate::CX { q1: 0, q2: 1 },
                Gate::RZ { q1: 0, param1: 2.0 },
            ],
            num_qubits: 2,
        };
        opt_routine_2(&mut c);
        c.remove_identities();
        assert_eq!(c.gates.len(), 2);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[0]);
        assert_eq!(Gate::RZ { q1: 0, param1: 3.0 }, c.gates[1]);
    }

    #[test]
    pub fn rz_prop_revert() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::RZ { q1: 1, param1: 1.0 },
                Gate::H(1),
                Gate::CX { q1: 0, q2: 1 },
                Gate::H(1),
            ],
            num_qubits: 2,
        };
        opt_routine_2(&mut c);
        assert_eq!(c.gates.len(), 4);
        assert_eq!(Gate::RZ { q1: 1, param1: 1.0 }, c.gates[0]);
        assert_eq!(Gate::H(1), c.gates[1]);
        assert_eq!(Gate::CX { q1: 0, q2: 1 }, c.gates[2]);
        assert_eq!(Gate::H(1), c.gates[3]);
    }

    #[test]
    pub fn cx_prop_1() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::CX { q1: 1, q2: 0 },
                Gate::CX { q1: 2, q2: 0 },
                Gate::CX { q1: 1, q2: 0 },
            ],
            num_qubits: 3,
        };
        opt_routine_3(&mut c);
        c.remove_identities();
        assert_eq!(c.gates.len(), 1);
        assert_eq!(Gate::CX { q1: 2, q2: 0 }, c.gates[0]);
    }

    #[test]
    pub fn cx_prop_2() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::CX { q1: 2, q2: 0 },
                Gate::CX { q1: 2, q2: 1 },
                Gate::CX { q1: 2, q2: 0 },
            ],
            num_qubits: 3,
        };
        opt_routine_3(&mut c);
        c.remove_identities();
        assert_eq!(c.gates.len(), 1);
        assert_eq!(Gate::CX { q1: 2, q2: 1 }, c.gates[0]);
    }

    #[test]
    pub fn cx_prop_3() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::CX { q1: 0, q2: 1 },
                Gate::H(1),
                Gate::CX { q1: 1, q2: 2 },
                Gate::H(1),
                Gate::CX { q1: 0, q2: 1 },
            ],
            num_qubits: 3,
        };
        opt_routine_3(&mut c);
        c.remove_identities();
        assert_eq!(c.gates.len(), 3);
        assert_eq!(Gate::H(1), c.gates[0]);
        assert_eq!(Gate::CX { q1: 1, q2: 2 }, c.gates[1]);
        assert_eq!(Gate::H(1), c.gates[2]);
    }

    #[test]
    pub fn cx_prop_4() {
        let mut c = CircuitSeq {
            gates: vec![
                Gate::CX { q1: 1, q2: 2 },
                Gate::H(2),
                Gate::CX { q1: 1, q2: 0 },
                Gate::CX { q1: 2, q2: 3 },
                Gate::H(2),
                Gate::CX { q1: 1, q2: 2 },
            ],
            num_qubits: 4,
        };
        opt_routine_3(&mut c);
        c.remove_identities();
        println!("got: {:?}", c.gates);
        assert_eq!(c.gates.len(), 4);
        assert_eq!(Gate::CX { q1: 1, q2: 0 }, c.gates[0]);
        assert_eq!(Gate::H(2), c.gates[1]);
        assert_eq!(Gate::CX { q1: 2, q2: 3 }, c.gates[2]);
        assert_eq!(Gate::H(2), c.gates[3]);
    }
}
