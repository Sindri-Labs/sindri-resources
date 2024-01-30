import json
import os
import subprocess
from sindri_labs.sindri import Sindri  # pip install sindri-labs

# Initialize an instance of the Sindri API SDK
API_KEY = os.getenv("SINDRI_API_KEY", "")
sindri = Sindri(API_KEY)

proof_input_file_path = "../multiplier2/input.json"
with open(proof_input_file_path, "r") as f:
    proof_id = sindri.prove_circuit("260f8728-2d72-4763-8514-b3251ffcf505", f.read())

proof_details = sindri.get_proof(proof_id)

# Retrieve output from the proof
public_input = proof_details["public"]
proof = proof_details["proof"]

result = subprocess.run(
    [
        "./build/verify-linux",
        "--proof",
        json.dumps(proof, separators=(",", ":")),
        "--pub",
        json.dumps(public_input, separators=(",", ":")),
    ],
    capture_output=True,
    text=True,
)
with open("calldata.txt", "w") as file:
    file.write(result.stdout)

result = subprocess.run(
    [
        "./build/insertAndTest-linux", 
    ],
    capture_output=True,
    text=True,
)
print(result.stdout, result.stderr)