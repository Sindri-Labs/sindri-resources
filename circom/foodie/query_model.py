import json
import sys, argparse, os
import time
import requests 

parser = argparse.ArgumentParser()
parser.add_argument('--circuit', type=str, required=True)
parser.add_argument('--ingredients', type=str, required=True)
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
 
proof_response = requests.post(
    API_URL + f"circuit/{args.circuit}/prove",
    headers=prove_header,
    data={
        "proof_input": json.dumps(query),
    },
).json()
PROOF_ID = proof_response["proof_id"]


#2. Poll
TIMEOUT = 1200 #timeout after 20 minutes
for i in range(TIMEOUT):
    poll_response = requests.get(
        API_URL + f"proof/{PROOF_ID}/detail",
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
    API_URL + f"proof/{PROOF_ID}/detail",
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
