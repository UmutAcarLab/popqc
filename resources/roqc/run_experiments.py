import test_correctness
import os
import matplotlib.pyplot as plt
import time
import sys

excluded_circuits = [
    # Small Circuits
    "hhl_n10_transpiled.qasm", #REMOVED MEASURE still kinda long tho
    # "hhl_n7_transpiled.qasm",
    "bb84_n8_transpiled.qasm",
    "ipea_n2_transpiled.qasm",
    "qec_sm_n5_transpiled.qasm", # barrier
    "simon_n6_transpiled.qasm",
    # "vqe_uccsd_n4_transpiled.qasm", #q REMOVED MEASURE
    "qpe_n9_transpiled.qasm", #barrier
    "inverseqft_n4_transpiled.qasm", #barrier
    # "vqe_uccsd_n8_transpiled.qasm", #q REMOVED MEASURE
    # "adder_n10_transpiled.qasm", #duplicate gates
    # "vqe_uccsd_n6_transpiled.qasm", #q REMOVED MEASRUE
    "shor_n5_transpiled.qasm", #reset
    "qft_n4_transpiled.qasm", #barrier
    # ====Medium Circuits====
    "vqe_n24_transpiled.qasm", # too long
    "cc_n12_transpiled.qasm", # if gate
    "qf21_n15_transpiled.qasm", # barrrier
    "multiply_n13_transpiled.qasm", # barrier
    "square_root_n18_transpiled.qasm", # reset
    # "sat_n11_transpiled.qasm",
    # "qec9xz_n17_transpiled.qasm",
    "seca_n11_transpiled.qasm", # barrier
    "bv_n19_transpiled.qasm", # barrier
    "bv_n14_transpiled.qasm", # barrier
    # "qram_n20_transpiled.qasm",
    # "bigadder_n18_transpiled.qasm",
    # ====Large Circuits====
    "bv_n70_transpiled.qasm",
    "bv_n30_transpiled.qasm",
    "bv_n140_transpiled.qasm",
    "cc_n151_transpiled.qasm",
    "cc_n32_transpiled.qasm",
    "cc_n301_transpiled.qasm",
    "bv_n280_transpiled.qasm",
    "cc_n64_transpiled.qasm",
    "vqe_uccsd_n28_transpiled.qasm", # c not declared 
]

long_tests = [
    #Medium Circuits:
    "bwt_n21_transpiled.qasm", # too long
    #Large Circuits:
    "multiplier_n350_transpiled.qasm", # Took too long
    "bwt_n97_transpiled.qasm", # Too long
    "bwt_n57_transpiled.qasm", # Too long
    "qft_n320_transpiled.qasm", # Too long
    "multiplier_n400_transpiled.qasm", # Too long
    "square_root_n45_transpiled.qasm", # Too long
    "square_root_n60_transpiled.qasm", # Too long
    "bwt_n37_transpiled.qasm", # I think this was too long
]

    

def run_all_tests(full):
    circuit_sizes = ["large"]
    original_lengths = []
    roqc_lengths = []
    voqc_lengths = []
    roqc_times = []
    voqc_times = []
    for test_circuit_size in circuit_sizes:
        for circuit_dir in os.walk(os.path.join("./roqc/test_circuits/QASMBench/", test_circuit_size)):
            circuit_files = circuit_dir[2]
            circuit_name = ""
            for filename in circuit_files:
                if (filename[-15:] == "transpiled.qasm" and
                    (((not full) and (not filename in excluded_circuits) and (not filename in long_tests))
                    or (full and (not filename in excluded_circuits)))):

                    print("Testing with:", filename)


                    full_path = os.path.join(circuit_dir[0], filename)
                    if test_circuit_size == "small":
                        result = test_correctness.test_correctness(full_path, suppressed=True, verify=True)
                    if test_circuit_size == "medium" or test_circuit_size == "large":
                        result = test_correctness.test_correctness(full_path, suppressed=True, verify=False)
                    original_lengths.append(result[1])
                    roqc_lengths.append(result[2])
                    voqc_lengths.append(result[3])
                    roqc_times.append(result[4])
                    voqc_times.append(result[5])



    roqc_result = [(original_lengths[i] - roqc_lengths[i]) / original_lengths[i] for i in range(len(original_lengths))]
    voqc_result = [(original_lengths[i] - voqc_lengths[i]) / original_lengths[i] for i in range(len(original_lengths))]

    final_length_results = list(zip(original_lengths, roqc_result, voqc_result, roqc_times, voqc_times))

    sort_func = lambda results: results[0]

    final_length_results.sort(key=sort_func)

    o_len, r_len, v_len, r_time, v_time = zip(*final_length_results)

    print(final_length_results)
    print(o_len)
    print(r_len)
    print(v_len)




    plt.plot(o_len, r_len, label="ROQC optimization percentage")
    plt.plot(o_len, v_len, label="VOQC optimization percentage")

    plt.legend()
    plt.show()

    plt.plot(o_len, r_time, label="ROQC optimization time")
    plt.plot(o_len, v_time, label="VOQC optimization time")

    plt.legend()
    plt.show()

if __name__ == "__main__":
    if len(sys.argv) > 1:
        if sys.argv[1] != "full":
            print("Invalid parameter")
        else:
            run_all_tests(full=True)
    else:
        run_all_tests(full=False)
