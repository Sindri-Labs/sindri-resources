import argparse
import json
import os
import subprocess
from sindri_labs.sindri import Sindri  # pip install sindri-labs

parser = argparse.ArgumentParser(description="Process circuit_id argument.")
parser.add_argument("--circuit_id", type=str, help="The circuit ID to use")
args = parser.parse_args()


# Initialize an instance of the Sindri API SDK
API_KEY = os.getenv("SINDRI_API_KEY", "")
sindri = Sindri(API_KEY)

PROOF_INPUT_FILE_PATH = os.path.abspath(os.path.join("..","..","..","..", "circuit_database", "circom", "multiplier2","input.json")) 
with open(PROOF_INPUT_FILE_PATH, "r") as f:
    proof_id = sindri.prove_circuit(args.circuit_id, f.read())

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