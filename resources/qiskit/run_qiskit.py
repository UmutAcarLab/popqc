import argparse
from qiskit import QuantumCircuit, transpile
from qiskit.qasm2 import dump
from qiskit.transpiler.passes import Optimize1qGatesDecomposition
from qiskit.transpiler import PassManager


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("-f", type=str, required=True)
    parser.add_argument("-o", type=str, required=True)
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    input_file = args.f
    output_file = args.o

    circuit = QuantumCircuit.from_qasm_file(input_file)
    circuit = transpile(
        circuit, optimization_level=3, basis_gates=["rz", "x", "h", "cx"]
    )

    new_data = []
    last_rz = (
        {}
    )  # key: qubit index, value: (instruction index in new_data, total angle)

    for instruction in circuit.data:
        gate_name = instruction.operation.name
        qubits = tuple(circuit.find_bit(q).index for q in instruction.qubits)

        if gate_name == "rz" and len(qubits) == 1:
            q_idx = qubits[0]
            angle = float(instruction.operation.params[0])

            if q_idx in last_rz:
                last_idx, last_angle = last_rz[q_idx]
                new_data[last_idx].operation.params[0] = last_angle + angle
                last_rz[q_idx] = (last_idx, last_angle + angle)
                continue
            else:
                new_data.append(instruction)
                last_rz[q_idx] = (len(new_data) - 1, angle)
        else:
            for q in qubits:
                if q in last_rz:
                    del last_rz[q]
            new_data.append(instruction)

    merged_circuit = QuantumCircuit(*circuit.qregs, *circuit.cregs, name=circuit.name)
    merged_circuit.data = new_data
    circuit = merged_circuit

    dump(circuit, output_file)
