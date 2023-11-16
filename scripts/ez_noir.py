#!/usr/bin/python3

from sindri_sdk import SindriSdk  # type: ignore

circuit_name = "Noir NEQ Circuit"
circuit_upload_path = "../noir/neq_v0_10_5/neq_v0_10_5.tar.gz"
proof_input = ""
proof_input_file_path = "../noir/neq_v0_10_5/Prover.toml"
with open(proof_input_file_path, "r") as f:
    proof_input = f.read()

sindri_sdk = SindriSdk(verbose_level=2)
circuit_id = sindri_sdk.create_circuit(circuit_name, circuit_upload_path)
proof_id = sindri_sdk.prove_circuit(circuit_id, proof_input)

print("Done!")
print("Using Sindri Labs' API is EZ!\n")
