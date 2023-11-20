#!/usr/bin/python3
from sindri_sdk import SindriSdk  # type: ignore

circuit_name = "Gnark v0.9.0 cubic"
circuit_upload_path = "../gnark/v9_cubic"
proof_input = ""
proof_input_file_path = "../gnark/v9_cubic/input.json"
with open(proof_input_file_path, "r") as f:
    proof_input = f.read()

sindri_sdk = SindriSdk(verbose_level=1)
circuit_id = sindri_sdk.create_circuit(circuit_name, circuit_upload_path)
proof_id = sindri_sdk.prove_circuit(circuit_id, proof_input)

print("Done!")
print("Using Sindri Labs' API is EZ!\n")
