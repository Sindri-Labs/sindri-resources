# Food ML demo


# Usage
```python

model_vocab_file_path = "vocab.txt"
model_weights_file_path = "model_weights.json"
model_regions_file_path = "regions.txt"


expected_result = "EastAsian"
query_raw = "ginger garlic soy_sauce"


# Transform raw query into proof input string
def prepare_query(query_raw) -> str:
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
    return json.dumps(query)


def interpret_result(proof_public) -> str:
    with open(model_regions_file_path, "r") as f:
        regions = [elem[:-1] for elem in f.readlines()]
    return regions[proof_public]

# Convert the plaintext input into the proof_input
proof_input = prepare_query(query_raw)


# Prove the circuit by hitting the /circuit/<circuit_id>/prove API endpoint with 
#   data={"proof_input": proof_input}

# Poll for the proof response:
# response_json is the result of hitting the API endpoint: 
#   /proof/<proof_id>/detail with proof_input in the data

# assert response_json["status"] == "Ready"
proof_public = response_json["public"]

# Obtain the plaintext result
region = interpret_result(int(proof_public[0]))


print(f"\nInputted query: {query_raw}")
print(f"\nExpected region: {expected_result}")
print(f"\nPredicted region: {region}")
```