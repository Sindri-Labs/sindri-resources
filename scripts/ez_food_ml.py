#!/usr/bin/python3

import json
import os
import sys

from sindri_sdk import SindriSdk  # type: ignore

food_ml_dir_path = "../circom/food_ml/"

model_vocab_file_path = os.path.join(food_ml_dir_path, "vocab.txt")
model_weights_file_path = os.path.join(food_ml_dir_path, "model_weights.json")
model_regions_file_path = os.path.join(food_ml_dir_path, "regions.txt")

circuit_upload_path = os.path.join(food_ml_dir_path, "food_ml.tar.gz")

circuit_name = "circom food_ml"


# Transform raw query into proof input string
def prepare_query(query_raw) -> dict:
    # Transform ingredient list to proof input
    with open(model_vocab_file_path, "r") as vocabfile:
        inglist = [elem[:-1] for elem in vocabfile.readlines()]

    invec = [0] * len(inglist)
    for elem in query_raw.split():
        if elem not in inglist:
            print("   " + elem + " not found.")
        else:
            invec[inglist.index(elem)] += 1
    if sum(invec) < 1:
        print("No ingredients identified")
        sys.exit(1)

    with open(model_weights_file_path, "r") as modelfile:
        query = json.loads(modelfile.read())

    query["in"] = [str(i) for i in invec]
    return query


def interpret_result(proof_public) -> str:
    with open(model_regions_file_path, "r") as f:
        regions = [elem[:-1] for elem in f.readlines()]
    return regions[proof_public]


if __name__ == "__main__":
    # expected_result = "NorthAmerican"
    # query_raw = "mango soy_sauce peanut_butter spaghetti watermelon beef"

    expected_result = "EastAsian"
    query_raw = "ginger garlic soy_sauce"

    # expected_result = "SouthernEuropean"
    # query_raw = "tomato olive_oil basil"

    # Convert the plaintext input into the proof_input_dict
    proof_input = json.dumps(prepare_query(query_raw))

    sindri_sdk = SindriSdk(verbose_level=1)
    circuit_id = sindri_sdk.create_circuit(circuit_name, circuit_upload_path)
    proof_id = sindri_sdk.prove_circuit(circuit_id, proof_input)

    sindri_sdk.set_verbose_level(0)
    proof = sindri_sdk.get_proof(proof_id)
    proof_public = proof["public"]

    # Obtain the plaintext result
    region = interpret_result(int(proof_public[0]))

    print(f"\nInputted query: {query_raw}")
    print(f"\nExpected region: {expected_result}")
    print(f"\nPredicted region: {region}")

    print("Done!\nUsing Sindri Labs' API is EZ!\n")
