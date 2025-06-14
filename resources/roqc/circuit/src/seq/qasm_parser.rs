use super::seq_impl::CircuitSeq;
use crate::Gate;
use std::fs::File;
use std::io::prelude::*;

fn extract_qubit_index(qubit_str: &str) -> usize {
    let start_idx = qubit_str.find('[').expect("Failed to find opening bracket") + 1;
    let end_idx = qubit_str.find(']').expect("Failed to find closing bracket");
    qubit_str[start_idx..end_idx]
        .parse::<usize>()
        .expect("Failed to parse qubit index")
}

fn extract_register_name(qubit_str: &str) -> String {
    let end_idx = qubit_str.find('[').expect("Failed to find opening bracket");
    qubit_str[0..end_idx].to_string()

}

fn calculate_qubit_index(qubit_str: &str, qubit_regs: &Vec<(String, usize)>) -> usize {
    let register_name = extract_register_name(qubit_str);
    let original_qubit_index = extract_qubit_index(qubit_str);
    for reg_name in qubit_regs {
        if reg_name.0 == register_name {
            return original_qubit_index + reg_name.1;
        }
    }
    0
}

#[allow(dead_code)]
fn float_to_int_with_precision(value: f64, precision: f64) -> usize {
    let rounded_value = ((value / precision).round() * precision) as usize;

    if (value - rounded_value as f64).abs() >= precision {
        panic!("Failed to round value to integer with precision");
    }

    rounded_value
}

fn extract_and_parse_parameter(param_str: &str) -> f64 {
    let start_idx = param_str
        .find('(')
        .expect("Failed to find opening parenthesis")
        + 1;
    // find the last ')'
    let end_idx = param_str
        .rfind(')')
        .expect("Failed to find closing parenthesis");
    let param_str = &param_str[start_idx..end_idx];

    let pi = std::f64::consts::PI;
    let param_str = param_str.replace("PI", &pi.to_string());
    let param_str = param_str.replace('π', &pi.to_string());

    let mut param: f64 =
        meval::eval_str(param_str).expect("Failed to evaluate parameter expression");
    if param < 0.0 {
        param = (2.0 * pi) + param;
    }

    param
}

pub fn parse_program(program: &str) -> CircuitSeq {
    let mut instructions = Vec::new();
    let mut n_qubits = 0;

    let mut qubit_regs : Vec<(String, usize)> = vec![];

    for line in program.lines() {
        let tokens: Vec<&str> = line
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|&token| !token.is_empty())
            .collect();

        if tokens.is_empty() {
            continue;
        }
        let op = tokens[0].split('(').collect::<Vec<&str>>()[0];
        match op.to_uppercase().as_str() {
            "OPENQASM" => {}
            "INCLUDE" => {}
            "QREG" => {
                qubit_regs.push((extract_register_name(tokens[1]), n_qubits));
                n_qubits += extract_qubit_index(tokens[1]);
            }
            "CCX" => {
                let qubit_str1 = tokens[1];
                let qubit_str2 = tokens[2];
                let qubit_str3 = tokens[3];
                let qubit_idx1 = calculate_qubit_index(qubit_str1, &qubit_regs);
                let qubit_idx2 = calculate_qubit_index(qubit_str2, &qubit_regs);
                let qubit_idx3 = calculate_qubit_index(qubit_str3, &qubit_regs);
                instructions.push(Gate::CCX {
                    q1: qubit_idx1,
                    q2: qubit_idx2,
                    q3: qubit_idx3,
                });
            }
            "CCZ" => {
                let qubit_str1 = tokens[1];
                let qubit_str2 = tokens[2];
                let qubit_str3 = tokens[3];
                let qubit_idx1 = calculate_qubit_index(qubit_str1, &qubit_regs);
                let qubit_idx2 = calculate_qubit_index(qubit_str2, &qubit_regs);
                let qubit_idx3 = calculate_qubit_index(qubit_str3, &qubit_regs);
                instructions.push(Gate::CCZ {
                    q1: qubit_idx1,
                    q2: qubit_idx2,
                    q3: qubit_idx3,
                });
            }
            "CX" => {
                let qubit_str1 = tokens[1];
                let qubit_str2 = tokens[2];
                let qubit_idx1 = calculate_qubit_index(qubit_str1, &qubit_regs);
                let qubit_idx2 = calculate_qubit_index(qubit_str2, &qubit_regs);
                instructions.push(Gate::CX {
                    q1: qubit_idx1,
                    q2: qubit_idx2,
                });
            }
            "CZ" => {
                let qubit_str1 = tokens[1];
                let qubit_str2 = tokens[2];
                let qubit_idx1 = calculate_qubit_index(qubit_str1, &qubit_regs);
                let qubit_idx2 = calculate_qubit_index(qubit_str2, &qubit_regs);
                instructions.push(Gate::CZ {
                    q1: qubit_idx1,
                    q2: qubit_idx2,
                });
            }
            "H" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::H(qubit_idx));
            }
            "X" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::X(qubit_idx));
            }
            "Y" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::Y(qubit_idx));
            }
            "Z" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::Z(qubit_idx));
            }
            "RX" => {
                let qubit_str = tokens[1];
                let param = extract_and_parse_parameter(tokens[0]);
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::RX {
                    param1: param,
                    q1: qubit_idx,
                });
            }
            "RY" => {
                let qubit_str = tokens[1];
                let param = extract_and_parse_parameter(tokens[0]);
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::RY {
                    param1: param,
                    q1: qubit_idx,
                });
            }
            "RZ" => {
                let qubit_str = tokens[1];
                let param = extract_and_parse_parameter(tokens[0]);
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::RZ {
                    param1: param,
                    q1: qubit_idx,
                });
            }
            "S" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::S(qubit_idx));
            }
            "SDG" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::Sdg(qubit_idx));
            }
            "SQRTX" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::SqrtX(qubit_idx));
            }
            "SQRTXDG" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::SqrtXdg(qubit_idx));
            }
            "SWAP" => {
                let qubit_str1 = tokens[1];
                let qubit_str2 = tokens[2];
                let qubit_idx1 = calculate_qubit_index(qubit_str1, &qubit_regs);
                let qubit_idx2 = calculate_qubit_index(qubit_str2, &qubit_regs);
                instructions.push(Gate::Swap {
                    q1: qubit_idx1,
                    q2: qubit_idx2,
                });
            }
            "T" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::T(qubit_idx));
            }
            "TDG" => {
                let qubit_str = tokens[1];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                instructions.push(Gate::Tdg(qubit_idx));
            }
            "U" => {
                let qubit_str = tokens[1];
                let theta_str = tokens[2];
                let phi_str = tokens[3];
                let lambda_str = tokens[4];
                let qubit_idx = calculate_qubit_index(qubit_str, &qubit_regs);
                let theta = extract_and_parse_parameter(theta_str);
                let phi = extract_and_parse_parameter(phi_str);
                let lambda = extract_and_parse_parameter(lambda_str);
                instructions.push(Gate::U {
                    q1: qubit_idx,
                    theta,
                    phi,
                    lambda,
                });
            }
            "CREG" => {}
            "ID" => {}
            "MEASURE" => {}
            "//" => {}
            _ => panic!("Unknown gate: {}", tokens[0]),
        }
    }

    CircuitSeq::new(instructions, n_qubits)
}

pub fn write_program(
    gates: &Vec<Gate>,
    num_qubits: usize,
    filename: String,
) -> std::io::Result<()> {
    let mut result: String = "OPENQASM 2.0;\ninclude \"qelib1.inc\";\n".to_string();
    result += "qreg q[";
    result += &num_qubits.to_string();
    result += "];\n";
    for gate in gates {
        match gate {
            Gate::X(q) => {
                result += "x q[";
                result += &q.to_string();
                result += "];\n";
            }
            Gate::H(q) => {
                result += "h q[";
                result += &q.to_string();
                result += "];\n";
            }
            Gate::Z(q) => {
                result += "z q[";
                result += &q.to_string();
                result += "];\n";
            }
            Gate::RZ { param1, q1 } => {
                result += "rz(";
                result += &param1.to_string();
                result += ") q[";
                result += &q1.to_string();
                result += "];\n";
            }
            Gate::CX {
                q1: control,
                q2: target,
            } => {
                result += "cx q[";
                result += &control.to_string();
                result += "], q[";
                result += &target.to_string();
                result += "];\n";
            }
            _ => {}
        }
    }
    let mut file = File::create(filename)?;
    file.write_all(result.as_bytes())?;
    Ok(())
}
