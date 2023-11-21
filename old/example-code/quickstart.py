import json
import os
import sys
import time

import requests  # pip install requests

# NOTE: Provide your API Key here
API_KEY = os.getenv("SINDRI_API_KEY", "")

API_VERSION = "v1"
API_URL = f"https://forge.sindri.app/api/{API_VERSION}/"

api_key_querystring = f"?api_key={API_KEY}"
headers_json = {"Accept": "application/json"}
headers_multipart = {"Accept": "multipart/form-data"}
headers_urlencode = {
    "Accept": "application/json",
    "Content-Type": "application/x-www-form-urlencoded",
}

# Create new circuit
print("1. Creating circuit...")
response = requests.post(
    API_URL + "circuit/create" + api_key_querystring,
    headers=headers_json,
    data={"circuit_name": "multiplier_example", "circuit_type": "Circom"},
)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."
circuit_id = response.json().get("circuit_id")  # Obtain circuit_id

# Load the circuit .tar.gz file
files = {"files": open(os.path.join("..", "circom", "multiplier2.tar.gz"), "rb")}

# Upload the circuit file
response = requests.post(
    API_URL + f"circuit/{circuit_id}/uploadfiles" + api_key_querystring,
    headers=headers_multipart,
    files=files,
)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."

# Initiate circuit compilation
response = requests.post(
    API_URL + f"circuit/{circuit_id}/compile" + api_key_querystring, headers=headers_json
)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."

# Poll circuit detail unitl it has a status of Ready or Failed
TIMEOUT = 600  # timeout after 10 minutes
for i in range(TIMEOUT):
    response = requests.get(
        API_URL + f"circuit/{circuit_id}/detail" + api_key_querystring,
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
else:
    print("Circuit compilation succeeded!")

# Initiate proof generation
print("2. Proving circuit...")
proof_input = json.dumps({"a": "7", "b": "42"})
response = requests.post(
    API_URL + f"circuit/{circuit_id}/prove" + api_key_querystring,
    headers=headers_urlencode,
    data={
        "proof_input": proof_input,
    },
)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."
proof_id = response.json()["proof_id"]  # Obtain proof_id

# Poll proof detail unitl it has a status of Ready or Failed
TIMEOUT = 1200  # timeout after 20 minutes
for i in range(TIMEOUT):
    response = requests.get(
        API_URL + f"proof/{proof_id}/detail" + api_key_querystring,
        headers=headers_json,
        params={
            "include_proof_input": False,
            "include_public": True,
            "include_verification_key": False,
            "include_proof": False,
        },
    )
    assert (
        response.status_code == 200
    ), f"Expected status code 200, received {response.status_code}."
    status = response.json()["status"]
    if status in ["Ready", "Failed"]:
        print(f"Proof poll exited after {i} seconds with status: {status}")
        break
    time.sleep(1)
else:
    sys.exit("Proof polling timed out")

# Check for proving issues
if status == "Failed":
    sys.exit("Proving failed")
else:
    # Retrieve output from the proof
    public_output = response.json()["public"]
    print(f"Circuit proof output signal: {public_output[0]}")
