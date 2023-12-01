#!/usr/bin/python3
import json
import os
import sys

from sindri_labs.sindri import Sindri  # pip install sindri-labs

# NOTE: Provide your API Key and API Url
API_KEY = os.getenv("SINDRI_API_KEY", "")
API_URL = os.getenv("SINDRI_API_URL", "https://forge.sindri.app/api/")

# Paths to auxiliary files
model_vocab_file_path = "vocab.txt"
model_weights_file_path = "model_weights.json"
model_regions_file_path = "regions.txt"


# Transform raw query into proof input string
def prepare_query(query_raw) -> dict:
    # Transform ingredient list to proof input
    with open(model_vocab_file_path, "r") as vocabfile:
        inglist = [elem[:-1] for elem in vocabfile.readlines()]

    invec = [0] * len(inglist)
    for elem in query_raw.split():
        if elem not in inglist:
            print(f"Ingredient not found: {elem}")
        else:
            invec[inglist.index(elem)] += 1
    if sum(invec) < 1:
        sys.exit("No ingredients identified.")

    with open(model_weights_file_path, "r") as modelfile:
        query = json.loads(modelfile.read())

    query["in"] = [str(i) for i in invec]
    return query


def interpret_result(proof_public) -> str:
    # Transform the resulting public output to the region
    with open(model_regions_file_path, "r") as f:
        regions = [elem[:-1] for elem in f.readlines()]
    return regions[proof_public]


# Initialize Sindri API SDK
sindri = Sindri(API_KEY, api_url=API_URL, verbose_level=1)

# Create the circuit
circuit_upload_path = "circuit/"
circuit_id = sindri.create_circuit(circuit_upload_path)


# Transform text input to proof input
expected_result = "NorthAmerican"
query_raw = "mango soy_sauce peanut_butter spaghetti watermelon beef"

# expected_result = "EastAsian"
# query_raw = "ginger garlic soy_sauce"

# expected_result = "SouthernEuropean"
# query_raw = "tomato olive_oil basil"


# Convert the plaintext input into the proof_input_dict
proof_input = json.dumps(prepare_query(query_raw))

# Prove the circuit
proof_id = sindri.prove_circuit(circuit_id, proof_input)

# Obtain the result
sindri.set_verbose_level(0)
proof = sindri.get_proof(proof_id)

# Obtain the plaintext result
proof_public = proof["public"]
region = interpret_result(int(proof_public[0]))

print(f"\nInputted query: {query_raw}")
print(f"\nExpected region: {expected_result}")
print(f"\nPredicted region: {region}")

print("Done!\nUsing Sindri Labs' Forge API is EZ!\n")
