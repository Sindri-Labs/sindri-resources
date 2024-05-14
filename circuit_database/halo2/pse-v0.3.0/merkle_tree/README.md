## Merkle Tree Circuit

This circuit makes use of many gadgets from the blockchain research group [Cardinal Cryptography](https://cardinal.co/).
Specifically, the gadgets are from this group's [shielder package](https://github.com/Cardinal-Cryptography/client-side-proving-benchmarks/tree/main) which implements a merkle tree proof verifier circuit via the vanilla fork of PSE's halo2 (v0.3.0.)

The only necessary components to add on top of the original `TestCircuit` inside the `shielder` package, are the following two functions.
```
impl MTPCircuit<Fr> {
    pub fn from_json(json_loc: &str,) -> (Self, Vec<Vec<Fr>>) {
        // Returns a fully synthesized circuit plus the instance vec over Fr
        // After reading and parsing a JSON file located at some arbitrary input path
    }
    pub fn keygen_circuit() -> Self {
        // Returns an empty circuit over Fr
    }
}
```
