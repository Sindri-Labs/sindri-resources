import json
import sys
import time
import requests 

api_key = ""

headers_json = {"Accept": "application/json"}
headers_multipart = {"Accept": "multipart/form-data"}
headers_urlencode = {
    "Accept": "application/json",
    "Content-Type": "application/x-www-form-urlencoded",
}

API_URL = "https://forge.sindri.app/api/"
api_key_querystring = f"?api_key={api_key}"

# Create new circuit
creation_response = requests.post(
    API_URL + "v1/circuit/create" + api_key_querystring,
    headers=headers_json,
    data={
        "circuit_name": "multiplier_example", 
        "circuit_type": "Circom C Groth16 bn254"
        },
)
assert creation_response.status_code == 201
circuit_id = creation_response.json().get("circuit_id")

# Load the circuit .tar.gz file
files = {"files": open("../circom/multiplier2.tar.gz", "rb")}

# Upload the circuit file
upload_response = requests.post(
    API_URL + f"v1/circuit/{circuit_id}/uploadfiles" + api_key_querystring, 
    headers=headers_multipart, 
    files=files
)
assert upload_response.status_code == 201

# Initiate compilation
compile_response = requests.post(
    API_URL + f"v1/circuit/{circuit_id}/compile" + api_key_querystring,
    headers=headers_json
)
assert compile_response.status_code == 201

# Poll circuit status
TIMEOUT = 600 #timeout after 10 minutes
action_complete = False
for i in range(TIMEOUT):
    poll_response = requests.get(
        API_URL + f"v1/circuit/{circuit_id}/detail" + api_key_querystring,
        headers=headers_json,
        params={"include_verification_key": False}
    )
    status = poll_response.json()["status"]
    if status in ["Ready", "Failed"]:
        print(f"Circuit poll exited after {i} seconds with status: {status}")
        action_complete = True
        break
    time.sleep(1)

# Check for compilation issues
if not action_complete:
    sys.exit("Circuit compile polling timed out")
elif status == "Failed":
    sys.exit("Circuit compilation failed")
else:
    print("Circom compilation succeeded!")

# Initiate proof generation
proof_input = json.dumps({"a":"7", "b":"42"})
proof_response = requests.post(
    API_URL + f"v1/circuit/{circuit_id}/prove" + api_key_querystring,
    headers=headers_urlencode,
    data={
        "proof_input": proof_input,
    },
)
assert proof_response.status_code == 201
proof_id = proof_response.json()["proof_id"]

# Poll proof status
TIMEOUT = 1200 #timeout after 20 minutes
action_complete = False
for i in range(TIMEOUT):
    poll_response = requests.get(
        API_URL + f"v1/proof/{proof_id}/detail" + api_key_querystring,
        headers=headers_json,
        params={
            "include_proof_input": False,
            "include_public": True,
            "include_verification_key": False,
            "include_proof": False,  
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
    # Retrieve output from the proof
    public_output = poll_response.json()["public"]
    print(f"Circuit output signal: {public_output[0]}")

# Verify proof
verify_response = requests.get(
    API_URL + f"v1/proof/{proof_id}/verify" + api_key_querystring,
    headers=headers_json
)
assert verify_response.status_code == 200
valid_proof = verify_response.json()["success"]
if valid_proof:
    print("Proof was valid")
else:
    print("Proof was not valid")
