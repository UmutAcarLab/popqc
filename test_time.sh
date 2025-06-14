PYTHON_SCRIPT="./resources/qiskit/run_qiskit.py" # <-- Replace this with the path to your Python script

# Check if the Python script exists
if [ ! -f "$PYTHON_SCRIPT" ]; then
    echo "Error: Python script '$PYTHON_SCRIPT' not found."
    exit 1
fi

echo "Timing Python script: $PYTHON_SCRIPT"
echo "-------------------------------------"

# Use the 'time' command to measure execution time
time python "$PYTHON_SCRIPT" -f temp_2.qasm -o temp_out_2.qasm

echo "-------------------------------------"
echo "Timing complete."

