#! /usr/bin/env python
import os

from sindri_labs.sindri import Sindri  # pip install sindri-labs

# NOTE: Provide your API Key and API Url
API_KEY = os.getenv("SINDRI_API_KEY", "")
API_URL = os.getenv("SINDRI_API_URL", "https://sindri.app/api/")

# Initialize Sindri API SDK
sindri = Sindri(API_KEY, api_url=API_URL, verbose_level=1)

# Create the circuit
circuit_upload_path = "circuit/"
circuit_id = sindri.create_circuit(circuit_upload_path)

# Prove the circuit
proof_input_file_path = "input.json"
with open(proof_input_file_path, "r") as f:
    proof_id = sindri.prove_circuit(circuit_id, f.read())

print("Done!\nUsing Sindri Labs' API is EZ!\n")
