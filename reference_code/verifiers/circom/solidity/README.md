# Adding Smart Contract Verification

We demonstrate how to produce a smart contract which will verify your Sindri proofs on the EVM.  The code in this repo assumes you are uploading a Circom circuit and generating Groth16 proofs, but other than those two assumptions, is fully generalized.  In particular, if you'd like to swap out the circuit for any other circuits in our database (or one of your own), see lines X, Y, and Z.

## Prepare the circuit and contract

For this step, ensure that you have the `sindri-labs` python SDK installed.

```
python3 compile_sol.py
```

## Generate a proof


## Verify the proof




This project demonstrates a basic Hardhat use case. It comes with a sample contract, a test for that contract, and a script that deploys that contract.

Try running some of the following tasks:

```shell
npx hardhat help
npx hardhat test
REPORT_GAS=true npx hardhat test
npx hardhat node
npx hardhat run scripts/deploy.js
```
