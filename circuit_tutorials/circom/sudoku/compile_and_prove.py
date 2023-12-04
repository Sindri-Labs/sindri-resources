#! /usr/bin/env python
import json
import os
import sys
import time
import tarfile
import io
import pprint

import requests  # pip install requests

# NOTE: Provide your API Key here
API_KEY = os.getenv("SINDRI_API_KEY", "")
API_URL = os.getenv("SINDRI_API_URL", "https://forge.sindri.app/api/")

API_VERSION = "v1"
API_URL = os.path.join(API_URL, API_VERSION, "")

#Define various headers
headers_json = {
    "Accept": "application/json",
    "Authorization": f"Bearer {API_KEY}"
}


# Create a tar archive of the circuit and upload via byte stream
circuit_upload_path = "circuit"
fh = io.BytesIO()
with tarfile.open(fileobj=fh, mode='w:gz') as tar:
    tar.add(circuit_upload_path, arcname="upload.tar.gz")
files = {"files": fh.getvalue()}

# Create new circuit
print("1. Creating circuit...")
response = requests.post(
    API_URL + "circuit/create",
    headers=headers_json,
    files=files,
)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."
circuit_id = response.json().get("circuit_id")  # Obtain circuit_id


# Poll circuit detail unitl it has a status of Ready or Failed
TIMEOUT = 600  # timeout after 10 minutes
for i in range(TIMEOUT):
    response = requests.get(
        API_URL + f"circuit/{circuit_id}/detail" ,
        headers=headers_json,
        params={"include_verification_key": False},
    )
    assert (
        response.status_code == 200
    ), f"Expected status code 200, received {response.status_code}."
    status = response.json()["status"]
    if status in ["Ready", "Failed"]:
        print(f"Circuit poll exited after {i} seconds with status: {status}")
        break
    time.sleep(1)
else:
    sys.exit("Circuit compile polling timed out")

# Check for compilation issues
if status == "Failed":
    sys.exit("Circuit compilation failed")

pprint.pprint(response.json(), depth=2, indent=2, width=40)


# Initiate proof generation
with open("example_solution.json", "r") as proof_file:
    proof_input = json.dumps(json.load(proof_file))
proof_response = requests.post(
    API_URL + f"circuit/{circuit_id}/prove",
    headers=headers_json,
    data={"proof_input": proof_input},
)
assert proof_response.status_code == 201
proof_id = proof_response.json()["proof_id"]
print(f"Proof ID: {proof_id}")

# Poll proof status
TIMEOUT = 1200 #timeout after 20 minutes
action_complete = False
for i in range(TIMEOUT):
    poll_response = requests.get(
        API_URL + f"proof/{proof_id}/detail",
        headers=headers_json,
        params={
            "include_proof_input": False,
            "include_public": True,
            "include_verification_key": True,
            "include_proof": True,
        }
    )
    status = poll_response.json()["status"]
    if status in ["Ready", "Failed"]:
        print(f"Proof poll exited after {i} seconds with status: {status}")
        action_complete = True
        break
    time.sleep(1)

# Check for proving issues
if not action_complete:
    sys.exit("Proof polling timed out")
elif status == "Failed":
    sys.exit("Proving failed")
else:
    proof_detail = poll_response.json()

# Save Artifacts for Verification
with open("verification_key.json","w") as outfile:
    json.dump(proof_detail["verification_key"], outfile, indent=4)
with open("public.json","w") as outfile:
    json.dump(proof_detail["public"], outfile, indent=4)
with open("proof.json","w") as outfile:
    json.dump(proof_detail["proof"], outfile, indent=4)

# Retrieve output from the proof
pprint.pprint(proof_detail, depth=1, indent=2, width=40)
print(proof_detail["public"])