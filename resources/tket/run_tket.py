from pytket.passes import (
    SequencePass,
    RemoveRedundancies,
    CommuteThroughMultis,
    FullPeepholeOptimise,
)
from pytket.passes import RebaseCustom
import pytket as tk
import argparse
from pytket.qasm import circuit_from_qasm, circuit_to_qasm
from pytket.circuit import OpType
from qiskit import QuantumCircuit
from pytket.extensions.qiskit import tk_to_qiskit
from qiskit import transpile
from qiskit.qasm2 import dump


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("-f", type=str, required=True)
    parser.add_argument("-o", type=str, required=True)
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    input_file = args.f
    output_file = args.o

    tket_circuit = circuit_from_qasm(input_file)
    hw_independent_passes = SequencePass(
        [
            CommuteThroughMultis(),
            RemoveRedundancies(),
            FullPeepholeOptimise(),
        ]
    )

    success = hw_independent_passes.apply(tket_circuit)
    qiskit_circuit = tk_to_qiskit(tket_circuit)
    circuit = transpile(
        qiskit_circuit, basis_gates=["cx", "rz", "h", "x"], optimization_level=0
    )
    # 合并连续的 Rz 门
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
                # 更新前一个 Rz 门的参数
                new_data[last_idx].operation.params[0] = last_angle + angle
                last_rz[q_idx] = (last_idx, last_angle + angle)  # 更新缓存的总角度
                # 不需要添加新的指令，继续下一个
                continue
            else:
                # 这是这个 qubit 上的第一个 Rz 门（或非连续的）
                new_data.append(instruction)
                last_rz[q_idx] = (len(new_data) - 1, angle)
        else:
            # 如果当前门不是 Rz，或者作用在多个 qubit 上，则清除涉及 qubit 的 last_rz 缓存
            for q in qubits:
                if q in last_rz:
                    del last_rz[q]
            new_data.append(instruction)

    # 用合并后的指令创建一个新的线路
    merged_circuit = QuantumCircuit(*circuit.qregs, *circuit.cregs, name=circuit.name)
    merged_circuit.data = new_data
    circuit = merged_circuit  # 替换原线路

    dump(circuit, output_file)
