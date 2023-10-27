import json
import sys, argparse, os
import time
import requests 

parser = argparse.ArgumentParser()
parser.add_argument('--name', type=str, required=True)
args = parser.parse_args()

#reads credentials from environment variable
API_KEY = os.getenv("FORGE_API_KEY", "")

API_VERSION = "v1"
API_URL = f"https://forge.sindri.app/api/{API_VERSION}/"

#initialize your header arguments
HEADERS = {
    'Accept': 'application/json',
    "Authorization": f"Bearer {API_KEY}",
} 

print('Creating circuit.')
circuit_name = sys.argv[1]
"""  UPLOAD  """
# 1. Create a circuit entity within Forge
creation_response = requests.post(
    API_URL+"circuit/create",
    headers=HEADERS,
    data={
        "circuit_name": args.name, 
        "circuit_type": "Circom"
        },
).json()
CIRCUIT_ID = creation_response.get("circuit_id")
print("   CIRCUIT_ID: "+CIRCUIT_ID)

print('Uploading.')
#2. Upload your local (compressed) circuit directory
files = {"files": open("../foodie.tar.gz", "rb")}
# Modify standard header for payload type
upload_header = HEADERS.copy()
upload_header["Accept"] = "multipart/form-data" #replace json header 
upload_response = requests.post(
    API_URL+f"circuit/{CIRCUIT_ID}/uploadfiles", 
    headers=upload_header, 
    files=files)

print('Compiling.')
#3. Compile
compile_response = requests.post(
    API_URL+f"circuit/{CIRCUIT_ID}/compile",
    headers=HEADERS)

#4. Poll
TIMEOUT = 600 #timeout after 10 minutes
for i in range(TIMEOUT):
    poll_response = requests.get(
        API_URL + f"circuit/{CIRCUIT_ID}/detail",
        headers=HEADERS,
        params={"include_verification_key": False}
    ).json()
    status = poll_response["status"]
    if status in ["Ready", "Failed"]:
        print(f"   Circuit poll exited after {i} seconds with status: {status}")
        break
    time.sleep(1)
if i==TIMEOUT-1:
    sys.exit("   Circuit compile polling timed out")



