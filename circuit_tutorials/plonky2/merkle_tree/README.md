# Merkle Tree Circuit

This Rust script uploads and proves a Plonky2 circuit that verifies a Merkle inclusion proof for a 1024-leaf Merkle tree.

The code used to construct Merkle trees in `circuit/src/merkle_tree.rs` is cloned from Hashcloak's [Merkle tree repository](https://github.com/hashcloak/plonky2-merkle-trees/blob/master/src/simple_merkle_tree/simple_merkle_tree.rs)

To run the code, change the `sample.env` to `.env` and enter your Sindri API Key. From the root of the `merkle_tree` directory, run:
```bash
cargo run --release
```
### Circuit Inputs
The user uploads an input JSON file with two fields:  a vector containing all of the values of the leaf nodes and an index value that corresponds to the leaf node whose inclusion the circuit is proving.

Before the Plonky2 circuit can verify a correct Merkle inclusion proof, the full Merkle tree must be constructed from the leaf nodes.  This is done by calling the `build` method from `merkle_tree.rs`.  This method populates the Merkle tree nodes with hash digests.  The `get_merkle_proof` method takes the populated Merkle tree along with the index value as inputs and uses the index value to construct the correct Merkle path for the Merkle inclusion proof.  The Merkle path, the index value, and the leaf node value corresponding to the index are supplied as inputs to the Plonky2 circuit.   

### Instructions
Refer to the [Sindri Plonky2 Tutorial](https://sindri.app/docs/how-to-guides/frameworks/plonky2/) for more information on this circuit, the proof output file, and the verification code.