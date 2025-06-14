import equiv_verification
import qiskit.qasm2
import subprocess
import sys
import random
import math
from tqdm import tqdm
import multiprocessing
import os
import time

NUM_QUBITS = 5

FAST_FAIL = False

def test_correctness(original_filename, suppressed, verify):

    original_circuit = qiskit.qasm2.load(original_filename, 
                                         include_path=qiskit.qasm2.LEGACY_INCLUDE_PATH,
                                         custom_instructions=qiskit.qasm2.LEGACY_CUSTOM_INSTRUCTIONS,
                                         custom_classical=qiskit.qasm2.LEGACY_CUSTOM_CLASSICAL
                                         )
    original_circuit.remove_final_measurements()
    original_circuit_transpiled = qiskit.transpile(original_circuit, basis_gates=["cx", "rz", "h", "x"])
    
    qiskit.qasm2.dump(original_circuit_transpiled, original_filename + ".transpiled")

    time_start = time.perf_counter_ns()
    result_roqc = subprocess.run(["./target/release/roqc", original_filename + ".transpiled"], capture_output=True, text=True)
    time_end = time.perf_counter_ns()

    roqc_time = time_end - time_start

    print("Finished roqc optimization")

    time_start = time.perf_counter_ns()
    result_voqc = subprocess.run(["./voqc_exec/voqc_full", "-f", original_filename + ".transpiled"], capture_output=True, text=True)
    time_end = time.perf_counter_ns()

    voqc_time = time_end - time_start

    print("Finished voqc optimization")

    if not suppressed:
        print(result_roqc.stdout)
        print(result_voqc.stdout)

    lines_roqc = result_roqc.stdout.splitlines()
    lines_voqc = result_voqc.stdout.splitlines()

    original_len = int(lines_roqc[0].split(": ")[1])
    roqc_len = int(lines_roqc[-2].split(": ")[1])
    voqc_len = int(lines_voqc[2].split(" ")[5])

    print("original length =", original_len)
    print("roqc length =", roqc_len)
    print("voqc length =", voqc_len)

    roqc_circuit = qiskit.qasm2.load(f'{original_filename}.transpiled.roqc')

    original_circuit_transpiled.remove_final_measurements()
    roqc_circuit.remove_final_measurements()

    if verify:
        success = equiv_verification.check_circuit_equivalence(roqc_circuit, original_circuit_transpiled)
    else:
        success = True
    if FAST_FAIL:
        assert(success)

    if not success:
        print("=======Test Failed!========")

    return (success, original_len, roqc_len, voqc_len, roqc_time, voqc_time)

def test_single_random(circuit_params):
    qubit_count = circuit_params[0]
    circuit_length = circuit_params[1]
    round = circuit_params[2]
    current_circuit = gen_random_circuit(qubit_count, circuit_length)
    circuit_file = f"random_{qubit_count}_{circuit_length}_{round}.qasm"
    with open(circuit_file, 'w') as f:
        f.write(current_circuit)

    if not test_correctness(circuit_file, f"{circuit_file}.roqc", True):
        os.rename(f"{circuit_file}", f"failed_tests/{circuit_file}")
        os.rename(f"{circuit_file}.roqc", f"failed_tests/{circuit_file}.roqc")
        return False
    else:
        os.remove(f"{circuit_file}")
        os.remove(f"{circuit_file}.roqc")
        return True

def test_random(qubit_count):
    num_failed = 0
    for circuit_length in range(4, 21):
        arguments = [(qubit_count, circuit_length, r) for r in range(10000)]
        with multiprocessing.Pool(processes=4) as pool, tqdm(total=10000, desc=f"Testing random circuits of length {circuit_length}, with {qubit_count} qubits") as pbar:
            for success in pool.imap(test_single_random, arguments):
                if not success:
                    num_failed += 1
                pbar.update()

    print("Total failed tests = ", num_failed)
    print("Check failed_tests to see which circuits caused failures")

#Define our own random circuit generator for more control than qiskit generator
def gen_random_circuit(num_qubits, circuit_length):
    gate_options = ["h", "cx", "rz", "x"]
    circuit = "OPENQASM 2.0;\ninclude \"qelib1.inc\";\nqreg q[" + str(num_qubits) + "];"
    for _ in range(circuit_length):
        gate = random.choice(gate_options)
        circuit += "\n" + gate
        match gate:
            case "x":
                circuit += " q[" + str(random.randint(0,num_qubits - 1)) + "];"
            case "h":
                circuit += " q[" + str(random.randint(0,num_qubits - 1)) + "];"
            case "rz":
                circuit += "(" + str(random.random() * math.pi) + ")"
                circuit += " q[" + str(random.randint(0,num_qubits - 1)) + "];"
            case "cx":
                q0 = random.randint(0,num_qubits - 1)
                q1 = random.randint(0,num_qubits - 1)
                while q1 == q0:
                    q1 = random.randint(0,num_qubits - 1)
                circuit += " q[" + str(q0) + "],"
                circuit += "q[" + str(q1) + "];"

    return circuit

if __name__ == '__main__':
    filename = sys.argv[1]
    

    if "fast_fail" in sys.argv:
        FAST_FAIL = True
    if filename == "random":
        for num_qubits in range(6, 7):
            test_random(num_qubits)
    else:
        test_correctness(filename, False, False)
