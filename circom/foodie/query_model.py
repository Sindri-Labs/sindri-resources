import json
import sys, argparse, os
import time
import requests 

parser = argparse.ArgumentParser()
parser.add_argument('--circuit', type=str, required=True)
parser.add_argument('--ingredients', type=str, required=True)
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


print("Transforming ingredient list to model input.")
with open("vocab.txt", "r") as vocabfile:
    inglist = [elem[:-1] for elem in vocabfile.readlines()]
ing2idx = lambda x: inglist.index(x)

invec = [0]*len(inglist)
for elem in args.ingredients.split():
    if elem not in inglist:
        print("   "+elem+" not found.")
    else:
        invec[ing2idx(elem)]+=1

if sum(invec)<1:
    print("No ingredients identified")
    sys.exit()

with open("model_weights.json","r") as modelfile:
    query = json.loads(modelfile.read())

query["in"] = [str(i) for i in invec]


print("Initiating proof.")
#1. Initiate Proof
prove_header = HEADERS.copy()
prove_header["Content-Type"] = "application/x-www-form-urlencoded"
proof_type = json.dumps({"name":"CPU Default"})
 
proof_response = requests.post(
    URL + f"api/v0/circuit/{args.circuit}/prove",
    headers=prove_header,
    data={
        "proof_input": json.dumps(query),
        "prover_implementation": proof_type,
    },
).json()
PROOF_ID = proof_response["proof_id"]


#2. Poll
TIMEOUT = 1200 #timeout after 20 minutes
for i in range(TIMEOUT):
    poll_response = requests.get(
        URL + f"api/v0/proof/{PROOF_ID}/detail",
        headers=HEADERS,
        params={
            "include_circuit_input": False,
            "include_public": False,
            "include_verification_key": False,
            "include_proof": False,  
        }
    ).json()
    status = poll_response["status"]
    if status in ["Ready", "Failed"]:
        print(f"   Proof poll exited after {i} seconds with status: {status}")
        break
if i==TIMEOUT-1:
    sys.exit("   Proof polling timed out")


#3. Retrieve an output from the proof
detail_response = requests.get(
    URL + f"api/v0/proof/{PROOF_ID}/detail",
    headers=HEADERS,
    params={
        "include_circuit_input": False,
        "include_public": True,
        "include_verification_key": False,
        "include_proof": True,  
    }
).json()

public_output = detail_response["public"]
regions = ['NorthernEuropean', 'MiddleEastern', 'SouthernEuropean', 'WesternEuropean', 'EasternEuropean', 'LatinAmerican', 'SoutheastAsian', 'EastAsian', 'African', 'NorthAmerican', 'SouthAsian']
print("   Predicted region: "+regions[int(public_output[0])])

print("   Proof Arguments:")
print("   Proof ID: "+PROOF_ID)
print(json.dumps(detail_response["proof"],indent=4))
print()

print("Verifying proof.")
verify_request = requests.get(
    URL+f"api/v0/proof/{PROOF_ID}/verify",
    headers=HEADERS
    ).json()
validity = verify_request["success"]
if validity:
    print("   Proof was valid")
else:
    print("   Proof was not valid")
