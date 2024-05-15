## Vector Multiplier Circuit

This circuit computes the public dot product of two private input vectors.

While the circuit is defined over a generic field, the Sindri functions (`keygen_circuit` and `from_json`) are only defined when the circuit is defined over `halo2_proofs::halo2curves::bn256::Fr`.
Additionally, since the circuit definition is contained in `circuit_def.rs` which is exported by the `lib.rs` file, our `sindri.json` classname has to reflect that:
```
  "className": "vector_multiply::circuit_def::VectorMultiplier",
```

The private inputs are vectors of arbitrary length, the public input is the dot product of those vectors.
