import os
import json
import subprocess

from sindri.sindri import Sindri  # pip install sindri


# Initialize an instance of the Sindri API SDK
API_KEY = os.getenv("SINDRI_API_KEY", "")
sindri = Sindri(API_KEY)

# Create and upload a circuit
CIRCUIT_UPLOAD_PATH = os.path.abspath(os.path.join("..","..","..","..", "circuit_database", "circom", "multiplier2")) # Adjust the path to your circuit directory
circuit_id = sindri.create_circuit(CIRCUIT_UPLOAD_PATH)
print(circuit_id)

circuit_details = sindri.get_circuit(circuit_id)  # Retrieve a circuit

verification_key = circuit_details.get("verification_key")
if verification_key:
    result = subprocess.run(
        [
            "./build/solidity-gen-linux", #os depencdency here! 
            "--string",
            json.dumps(verification_key, separators=(",", ":")),
        ],
        capture_output=True,
        text=True,
    )
    #print(result.stdout, result.stderr)

