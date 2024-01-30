# Smart Contract Verification

We demonstrate how to produce a smart contract which will verify your Sindri proofs on the EVM.
The code in this repo assumes you are uploading a Circom circuit and generating Groth16 proofs, but other than those two assumptions, is fully generalized.
In particular, if you'd like to swap out the circuit for any other circuits in our database (or one of your own), see line 13 of `compile_sol_sdk.py` and line 10 of `prove_verify.py`

## Installation & Authorization

In order to build the solidity code generator and other helpful utilities (converting Sindri API return data into solidity compatible calldata), you'll need to install dependencies and build:
```
npm install
npm run build
```

Also, recall that you need to authenticate with Sindri when using the API.
Before running any of the python scripts in this directory, you should set your `SINDRI_API_KEY` as an environment variable.  
```
export SINDRI_API_KEY=<YOUR_API_KEY>
```

Finally, ensure that you have the `sindri-labs` python SDK installed.
The [Python SDK Quickstart](https://sindri.app/docs/getting-started/api-sdk/#python-sdk) contains installation instructions and a high-level walkthrough of the functionality of this package, but the following will suffice if you have pip installed:
```
pip install sindri-labs
```

## Prepare the circuit and contract

**Non-linux Users**

Line 22 of `compile_sol.py` refers to the linux build of our solidity verifier.  Replace that with `./build/solidity-gen-macos` or `./build/solidity-gen-win.exe` according to your operating system.  

When you run the following script, a Circom circuit is uploaded to Sindri.
Since there are no proving and verification keys contained in the upload, Sindri runs a mock key generation process to produce those keys (for testing and development purposes).
The script then retrieves the verification key from Sindri and passes that to the solidity generator.
```
python3 compile_sol.py
```
After the script completes, you should see a contract calls `verifier.sol` in the `contracts/` directory.


## Generate a proof and verify via hardhat

**Non-linux Users**

Lines 22 and 36 of `prove_verify.py` refers to the linux builds.  Replace them with `./build/solidity-gen-macos` or `./build/solidity-gen-win.exe` according to your operating system.  

```
python3 prove_verify.py --circuit_id "uuid-from-previous-step"
```

The script above requests for a zero-knowledge proof verifing the inputs and outputs of our Circom circuit.
The proof along with any public data is then converted into a calldata vector, which is routed  to a specific line of `test/groth16verifier.test.js`.
After the smart contract testing script has been filled out, we use hardhat to deploy the contract and test with our specific proof output.
As an optional step, you could revise any character in line 8 of `test/groth16verifier.test.js` and rerun `npx hardhat test` via the command line to see what happens when a proof does not successfully verify.
