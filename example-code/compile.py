import json
import sys
import time
import requests  # pip install requests

# NOTE: Provide your API key here
api_key = ""

# NOTE: Give your circuit a name here
CIRCUIT_NAME = "cubic_example"

# NOTE: Provide the path to your .tar.gz file, and circuit type
FILE_NAME = "../gnark/cubic.tar.gz"
CIRCUIT_TYPE = "Gnark"

API_VERSION = "v1"
API_URL = f"https://forge.sindri.app/api/{API_VERSION}/"
api_key_querystring = f"?api_key={api_key}"
headers_json = {"Accept": "application/json"}
headers_multipart = {"Accept": "multipart/form-data"}
headers_urlencode = {
    "Accept": "application/json",
    "Content-Type": "application/x-www-form-urlencoded",
}

# Create new circuit
print("Creating circuit...")
response = requests.post(
    API_URL + "circuit/create" + api_key_querystring,
    headers=headers_json,
    data={
        "circuit_name": CIRCUIT_NAME,
        "circuit_type": CIRCUIT_TYPE
    },
)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."
circuit_id = response.json().get("circuit_id")  # Obtain circuit_id

# Load the circuit .tar.gz file
files = {"files": open(FILE_NAME, "rb")}

# Upload the circuit file
response = requests.post(
    API_URL + f"circuit/{circuit_id}/uploadfiles" + api_key_querystring,
    headers=headers_multipart,
    files=files,
)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."

# Initiate circuit compilation
response = requests.post(API_URL + f"circuit/{circuit_id}/compile" +
                         api_key_querystring,
                         headers=headers_json)
assert response.status_code == 201, f"Expected status code 201, received {response.status_code}."

# Poll circuit detail unitl it has a status of Ready or Failed
TIMEOUT = 600  # timeout after 10 minutes
for i in range(TIMEOUT):
  response = requests.get(
      API_URL + f"circuit/{circuit_id}/detail" + api_key_querystring,
      headers=headers_json,
      params={"include_verification_key": False},
  )
  assert (response.status_code == 200
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
  print(json.dumps(response.json(), indent=2))