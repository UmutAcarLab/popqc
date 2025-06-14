import numpy as np
from qiskit import QuantumCircuit
from qiskit.quantum_info import Operator, Statevector, state_fidelity


def check_circuit_equivalence(
    circuit1: QuantumCircuit, circuit2: QuantumCircuit
) -> bool:
    if circuit1.num_qubits != circuit2.num_qubits:
        raise ValueError(
            f"Circuits have different number of qubits: {circuit1.num_qubits} vs {circuit2.num_qubits}"
        )

    matrix1 = Operator(circuit1).data
    matrix2 = Operator(circuit2).data
    mat = matrix1 @ matrix2.conj().T
    mat = mat / np.trace(mat) * mat.shape[0]

    is_equal = np.allclose(mat, np.eye(mat.shape[0]), rtol=1e-10, atol=1e-10)

    return is_equal


def check_circuit_equivalence_monte_carlo(
    circuit1: QuantumCircuit,
    circuit2: QuantumCircuit,
    num_trials: int = 5,
    threshold: float = 0.999,
) -> bool:
    if circuit1.num_qubits != circuit2.num_qubits:
        raise ValueError(
            f"Circuits have different number of qubits: {circuit1.num_qubits} vs {circuit2.num_qubits}"
        )

    num_qubits = circuit1.num_qubits

    for _ in range(num_trials):
        random_qc = QuantumCircuit(num_qubits)
        for i in range(num_qubits):
            random_qc.rx(np.random.random() * 2 * np.pi, i)

        qc1 = random_qc.compose(circuit1)
        qc2 = random_qc.compose(circuit2)

        sv1 = Statevector.from_instruction(qc1)
        sv2 = Statevector.from_instruction(qc2)

        fid = state_fidelity(sv1, sv2)
        if fid < threshold:
            return False

    return True


if __name__ == "__main__":
    qc1 = QuantumCircuit.from_qasm_file(
        "nwq_binary_welded_tree_n17.qasm.preprocessed.new.roqc"
    )
    qc2 = QuantumCircuit.from_qasm_file(
        "nwq_binary_welded_tree_n17.qasm.preprocessed.new"
    )

    # Use Monte Carlo method which is more efficient for large circuits
    if check_circuit_equivalence_monte_carlo(qc1, qc2):
        print("Circuit 1 and 2 are equivalent (Monte Carlo method)")
    else:
        print("Circuit 1 and 2 are not equivalent (Monte Carlo method)")

    # You can still use the original method for comparison
    if check_circuit_equivalence(qc1, qc2):
        print("Circuit 1 and 2 are equivalent (Unitary method)")
    else:
        print("Circuit 1 and 2 are not equivalent (Unitary method)")
