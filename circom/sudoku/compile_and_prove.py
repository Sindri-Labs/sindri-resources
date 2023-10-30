import json
import os
import sys
import time
import tarfile
import io
import pprint

import requests  # pip install requests

# You should pass your key in as an environment variable
API_KEY = os.getenv("SINDRI_API_KEY", "")

# Use V1 of Sindri API
API_VERSION = "v1"
API_URL = f"https://forge.sindri.app/api/{API_VERSION}/"

headers_json = {
    "Accept": "application/json",
    "Authorization": f"Bearer {API_KEY}"
}
headers_multipart = {
    "Accept": "multipart/form-data",
    "Authorization": f"Bearer {API_KEY}"
}
headers_urlencode = {
    "Accept": "application/json",
    "Content-Type": "application/x-www-form-urlencoded",
    "Authorization": f"Bearer {API_KEY}"
}

# Create new circuit
creation_response = requests.post(
    API_URL + "circuit/create",
    headers=headers_json,
    data={
        "circuit_name": "sudoku",
        "circuit_type": "Circom"
        },
)
assert creation_response.status_code == 201
circuit_id = creation_response.json().get("circuit_id")
print(f"Circuit ID: {circuit_id}")

# Create a tar archive and upload via byte stream
fh = io.BytesIO()
with tarfile.open(fileobj=fh, mode='w:gz') as tar:
    tar.add("circuit/")
files = {"files": fh.getvalue()}

# Upload the circuit file
upload_response = requests.post(
    API_URL + f"circuit/{circuit_id}/uploadfiles",
    headers=headers_multipart,
    files=files
)
assert upload_response.status_code == 201


# Initiate compilation
compile_response = requests.post(
    API_URL + f"circuit/{circuit_id}/compile",
    headers=headers_json
)
assert compile_response.status_code == 201


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
with open("example_solution.json","r") as proof_file:
    proof_input = json.dumps(json.load(proof_file))
proof_response = requests.post(
    API_URL + f"circuit/{circuit_id}/prove",
    headers=headers_urlencode,
    data={
        "proof_input": proof_input,
    },
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