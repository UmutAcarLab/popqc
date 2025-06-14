import numpy as np
import qiskit.quantum_info as qi
from qiskit import Aer, QuantumCircuit, execute
from qiskit.circuit.random import random_circuit
from qiskit.providers.aer import QasmSimulator
from qiskit.quantum_info import random_statevector


def load_circuit_from_qasm(file_path):
    with open(file_path, "r") as file:
        qasm = file.read()
    circuit = QuantumCircuit.from_qasm_str(qasm)
    return circuit


def compare_circuits(file_path_1, file_path_2, trials=2):
    circuit_1 = load_circuit_from_qasm(file_path_1)
    circuit_2 = load_circuit_from_qasm(file_path_2)

    simulator = Aer.get_backend("aer_simulator")

    for _ in range(trials):
        rand_circuit = random_circuit(
            circuit_1.num_qubits, 5, max_operands=3, measure=False
        )
        c1 = rand_circuit.compose(circuit_1)
        c2 = rand_circuit.compose(circuit_2)
        sv1 = qi.Statevector.from_instruction(c1)
        sv2 = qi.Statevector.from_instruction(c2)

        if sv1 != sv2:
            return False
    return True


file_path_1 = "benchmarks/large_circ/qft_n29/qft_n29.qasm.transpiled.strip.soam"
file_path_2 = "benchmarks/large_circ/qft_n29/qft_n29.qasm.transpiled.strip"
are_circuits_equal = compare_circuits(file_path_1, file_path_2)
print(
    "The circuits are equivalent:"
    if are_circuits_equal
    else "The circuits are not equivalent."
)
