use crate::types::{QubitIndex, Real};
use std::fmt::Display;
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Gate {
    CCX {
        q1: QubitIndex,
        q2: QubitIndex,
        q3: QubitIndex,
    },
    CCZ {
        q1: QubitIndex,
        q2: QubitIndex,
        q3: QubitIndex,
    },
    CX {
        q1: QubitIndex,
        q2: QubitIndex,
    },
    CZ {
        q1: QubitIndex,
        q2: QubitIndex,
    },
    H(QubitIndex),
    X(QubitIndex),
    Y(QubitIndex),
    Z(QubitIndex),
    RX {
        param1: Real,
        q1: QubitIndex,
    },
    RY {
        param1: Real,
        q1: QubitIndex,
    },
    RZ {
        param1: Real,
        q1: QubitIndex,
    },
    S(QubitIndex),
    Sdg(QubitIndex),
    SqrtX(QubitIndex),
    SqrtXdg(QubitIndex),
    Swap {
        q1: QubitIndex,
        q2: QubitIndex,
    },
    T(QubitIndex),
    Tdg(QubitIndex),
    U {
        q1: QubitIndex,
        theta: Real,
        phi: Real,
        lambda: Real,
    },
    B,
}

impl Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gate::CCX { q1, q2, q3 } => {
                write!(f, "ccx q[{q1}], q[{q2}], q[{q3}]")
            }
            Gate::CCZ { q1, q2, q3 } => {
                write!(f, "ccz q[{q1}], q[{q2}], q[{q3}]")
            }
            Gate::CX { q1, q2 } => {
                write!(f, "cx q[{q1}], q[{q2}]")
            }
            Gate::CZ { q1, q2 } => {
                write!(f, "cz q[{q1}], q[{q2}]")
            }
            Gate::H(q1) => {
                write!(f, "h q[{q1}]")
            }
            Gate::X(q1) => {
                write!(f, "x q[{q1}]")
            }
            Gate::Y(q1) => {
                write!(f, "y q[{q1}]")
            }
            Gate::Z(q1) => {
                write!(f, "z q[{q1}]")
            }
            Gate::RX { param1, q1 } => {
                write!(f, "rx({param1}) q[{q1}]")
            }
            Gate::RY { param1, q1 } => {
                write!(f, "ry({param1}) q[{q1}]")
            }
            Gate::RZ { param1, q1 } => {
                write!(f, "rz({param1}) q[{q1}]")
            }
            Gate::S(q1) => {
                write!(f, "s q[{q1}]")
            }
            Gate::Sdg(q1) => {
                write!(f, "sdg q[{q1}]")
            }
            Gate::SqrtX(q1) => {
                write!(f, "sx q[{q1}]")
            }
            Gate::SqrtXdg(q1) => {
                write!(f, "sxdg q[{q1}]")
            }
            Gate::Swap { q1, q2 } => {
                write!(f, "swap q[{q1}], q[{q2}]")
            }
            Gate::T(q1) => {
                write!(f, "t q[{q1}]")
            }
            Gate::Tdg(q1) => {
                write!(f, "tdg q[{q1}]")
            }
            Gate::U {
                q1,
                theta,
                phi,
                lambda,
            } => {
                write!(f, "u({theta}, {phi}, {lambda}) q[{q1}]")
            }
            Gate::B => {
                panic!("B gate is not supported in the display");
            }
        }
    }
}

impl Gate {
    pub fn qubits(&self) -> Vec<QubitIndex> {
        match &self {
            Gate::CCX { q1, q2, q3 } => {
                vec![*q1, *q2, *q3]
            }
            Gate::CCZ { q1, q2, q3 } => {
                vec![*q1, *q2, *q3]
            }
            Gate::CX { q1, q2 } => {
                vec![*q1, *q2]
            }
            Gate::CZ { q1, q2 } => {
                vec![*q1, *q2]
            }
            Gate::H(q1) => {
                vec![*q1]
            }
            Gate::X(q1) => {
                vec![*q1]
            }
            Gate::Y(q1) => {
                vec![*q1]
            }
            Gate::Z(q1) => {
                vec![*q1]
            }
            Gate::RX { q1, .. } => {
                vec![*q1]
            }
            Gate::RY { q1, .. } => {
                vec![*q1]
            }
            Gate::RZ { q1, .. } => {
                vec![*q1]
            }
            Gate::S(q1) => {
                vec![*q1]
            }
            Gate::Sdg(q1) => {
                vec![*q1]
            }
            Gate::SqrtX(q1) => {
                vec![*q1]
            }
            Gate::SqrtXdg(q1) => {
                vec![*q1]
            }
            Gate::Swap { q1, q2 } => {
                vec![*q1, *q2]
            }
            Gate::T(q1) => {
                vec![*q1]
            }
            Gate::Tdg(q1) => {
                vec![*q1]
            }
            Gate::U { q1, .. } => {
                vec![*q1]
            }
            Gate::B => {
                vec![]
            }
        }
    }
}
