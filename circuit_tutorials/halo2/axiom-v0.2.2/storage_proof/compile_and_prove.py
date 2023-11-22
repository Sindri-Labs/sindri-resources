#!/usr/bin/python3
import sys
sys.path.append("../../../../reference_code")
from sdk import SindriSdk
import os

# NOTE: Provide your API Key here
API_KEY = os.getenv("SINDRI_API_KEY", "")
API_URL = os.getenv("SINDRI_API_URL", "https://forge.sindri.app/api/")

circuit_upload_path = "circuit"
proof_input = ""
proof_input_file_path = "input.json"
with open(proof_input_file_path, "r") as f:
    proof_input = f.read()

sindri_sdk = SindriSdk(verbose_level=1, api_key=API_KEY, api_url=API_URL)
circuit_id = sindri_sdk.create_circuit(circuit_upload_path)
proof_id = sindri_sdk.prove_circuit(circuit_id, proof_input)

print("Done!")
print("Using Sindri Labs' API is EZ!\n")
