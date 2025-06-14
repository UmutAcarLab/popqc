import multiprocessing
import subprocess
import time
import csv


def run_command(cmd):
    now = time.time()
    process = None
    try:
        process = subprocess.Popen(cmd, shell=True)
        process.wait(timeout=86400)
        return process.returncode, time.time() - now
    except subprocess.TimeoutExpired:
        print(f"Command timed out: {cmd}")
        if process:
            process.kill()
        return 1, time.time() - now


def run_parallel(commands):
    pool = multiprocessing.Pool()
    results = pool.map(run_command, commands)
    pool.close()
    pool.join()
    return results


if __name__ == "__main__":
    names = [
        # "nwq_boolean_satisfaction_n28.qasm",
        # "nwq_boolean_satisfaction_n30.qasm",
        # "nwq_boolean_satisfaction_n32.qasm",
        "nwq_boolean_satisfaction_n34.qasm",
        # "nwq_binary_welded_tree_n17.qasm",
        # "nwq_binary_welded_tree_n21.qasm",
        # "nwq_binary_welded_tree_n25.qasm",
        "nwq_binary_welded_tree_n29.qasm",
        # "grover_n9_from_python.qasm",
        # "grover_n11_from_python.qasm",
        # "grover_n13_from_python.qasm",
        "grover_n15_from_python.qasm",
        # "hhl_n7_from_python.qasm",
        # "hhl_n9_from_python.qasm",
        # "hhl_n11_from_python.qasm",
        # "hhl_n13_from_python.qasm",
        # "shor_7_mod_15_n10_from_python.qasm",
        # "shor_7_mod_15_n12_from_python.qasm",
        # "shor_7_mod_15_n14_from_python.qasm",
        "shor_7_mod_15_n16_from_python.qasm",
        # "nwq_square_root_n42.qasm",
        # "nwq_square_root_n48.qasm",
        # "nwq_square_root_n54.qasm",
        "nwq_square_root_n60.qasm",
        # "nwq_statevector_n5.qasm",
        # "nwq_statevector_n6.qasm",
        # "nwq_statevector_n7.qasm",
        "nwq_statevector_n8.qasm",
        # "vqe_n18_from_python.qasm",
        # "vqe_n22_from_python.qasm",
        # "vqe_n26_from_python.qasm",
        "vqe_n30_from_python.qasm",
    ]
    commands = []
    for name in names:
        # commands.append(f"./resources/voqc/voqc_exec_linux -f /home/cc/benchmarks/{name} -o /home/cc/benchmarks/{name}.optimized")
        commands.append(
            f"python ./resources/qiskit/run_qiskit.py -f /home/cc/benchmarks/{name} -o /home/cc/benchmarks/{name}.optimized"
        )
    times = run_parallel(commands)
    # times = [[0, 0] for _ in range(len(names))]
    # read the results
    results = []
    for name, time in zip(names, times):
        try:
            with open(f"/home/cc/benchmarks/{name}.optimized", "r") as f:
                # number of lines, removing empty lines
                num_lines = 0
                lines = f.readlines()
                for line in lines:
                    if line.strip():
                        num_lines += 1

                results.append([name, num_lines - 3, time[1]])
        except FileNotFoundError:
            results.append([name, "NA", time[1]])
    print(results)
    # write to csv
    with open("qiskit_results.csv", "w") as f:
        writer = csv.writer(f)
        writer.writerow(["name", "gates", "time"])
        writer.writerows(results)
