#!/usr/bin/python3
from sdk import SindriSdk
import os

# NOTE: Provide your API Key here
API_KEY = os.getenv("SINDRI_API_KEY", "")
API_URL = os.getenv("SINDRI_API_URL", "https://forge.sindri.app/api/")

circuit_name = "Circom multiplier2"
circuit_upload_path = "../circuit_database/circom/multiplier2"
proof_input = ""
proof_input_file_path = "../circuit_database/circom/multiplier2/input.json"
with open(proof_input_file_path, "r") as f:
    proof_input = f.read()

sindri_sdk = SindriSdk(verbose_level=2, api_key=API_KEY, api_url=API_URL)
circuit_id = sindri_sdk.create_circuit(circuit_name, circuit_upload_path)
proof_id = sindri_sdk.prove_circuit(circuit_id, proof_input)

print("Done!")
print("Using Sindri Labs' API is EZ!\n")
