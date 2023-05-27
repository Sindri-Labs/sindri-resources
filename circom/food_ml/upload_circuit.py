import json
import sys, argparse, os
import time
import requests 

parser = argparse.ArgumentParser()
parser.add_argument('--name', type=str, required=True)
args = parser.parse_args()

#reads credentials from local json
with open("forge_credentials.json","r") as f:
    user_creds = json.load(f)
    USERNAME = user_creds["user"]
    PASSWORD = user_creds["pass"]

#initialize your header arguments
HEADERS = {'Accept': 'application/json'} 
PROTO = "https"
HOST = "forge.sindri.app/"
URL = f"{PROTO}://{HOST}"

#retrieve access key
print('Signing in.')
auth_response = requests.post(
    URL + "api/token/pair", 
    headers=HEADERS, 
    json={"username": USERNAME, "password": PASSWORD}
).json()
ACCESS_KEY = auth_response["access"]

#add your token to the header of any future calls
HEADERS["Authorization"]= f"Bearer {ACCESS_KEY}"

print('Creating circuit.')
circuit_name = sys.argv[1]
"""  UPLOAD  """
# 1. Create a circuit entity within Forge
creation_response = requests.post(
    URL+"api/v0/circuit/create",
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
files = {"files": open("foodie.tar.gz", "rb")}
# Modify standard header for payload type
upload_header = HEADERS.copy()
upload_header["Accept"] = "multipart/form-data" #replace json header 
upload_response = requests.post(
    URL+f"api/v0/circuit/{CIRCUIT_ID}/uploadfiles", 
    headers=upload_header, 
    files=files)

print('Compiling.')
#3. Compile
compile_response = requests.post(
    URL+f"api/v0/circuit/{CIRCUIT_ID}/compile",
    headers=HEADERS)

#4. Poll
TIMEOUT = 600 #timeout after 10 minutes
for i in range(TIMEOUT):
    poll_response = requests.get(
        URL + f"api/v0/circuit/{CIRCUIT_ID}/detail",
        headers=HEADERS,
        params={"include_verification_key": False}
    ).json()
    status = poll_response["status"]
    if status in ["Ready", "Failed"]:
        print(f"   Circuit poll exited after {i} seconds with status: {status}")
        break
if i==TIMEOUT-1:
    sys.exit("   Circuit compile polling timed out")



