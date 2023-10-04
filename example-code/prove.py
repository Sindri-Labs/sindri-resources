import json
import sys
import time
import requests  # pip install requests

# NOTE: Provide your API key here
api_key = ""

# NOTE: Provide your circuit ID and the path to your circuit input here
CIRCUIT_ID = ""
PROOF_INPUT = "../gnark/cubic/input.json"

API_VERSION = "v1"
API_URL = f"https://forge.sindri.app/api/{API_VERSION}/"
api_key_querystring = f"?api_key={api_key}"
headers_json = {"Accept": "application/json"}
headers_multipart = {"Accept": "multipart/form-data"}
headers_urlencode = {
    "Accept": "application/json",
    "Content-Type": "application/x-www-form-urlencoded",
}

# Initiate proof generation 3 times
for _ in range(3):
  print("Proving circuit...")
  with open(PROOF_INPUT, "r") as proof_file:
    proof_input = json.dumps(json.load(proof_file))

  response = requests.post(
      API_URL + f"circuit/{CIRCUIT_ID}/prove" + api_key_querystring,
      headers=headers_urlencode,
      data={
          "proof_input": proof_input
          # "prover_implementation": json.dumps({"name":"CPU Stock 1"})
      },
  )
  assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."
  proof_id = response.json()["proof_id"]  # Obtain proof_id

  # Poll proof detail until it has a status of Ready or Failed
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
    assert (response.status_code == 200
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
    print("Proof generation succeeded!")
    print(json.dumps(response.json(), indent=2))
