# v0.2.2 Circuit Requirements

- Upload all your rust language source code for a circuit
  - include the manifest `Cargo.toml`
  - exclude executable targets
- All code imports must use relative paths
- Your main circuit struct should be public
- Indicate the main circuit upon which we build our prover via the `Sindri.json` file
- Your main circuit should implement the following three functionalities:
  - `circuit::default()`
  - `circuit.instance()`
  - `circuit.from_json(path)`

For greater clarity regarding the three functions above, either examine the **v0.2.2** examples or see our documentation walkthrough.

## Structure
In this hypothetical example, the user has a main circuit which is defined in the top level of src which references some inner circuitry defined in the chips folder.  For their offline proof testing, they build `local_prover` and even though Sindri doesn't require any of the code in `src/bin/local_prover.rs`, it should be included so that the entire package can be built and referenced by our prover binaries.
```
my_repo/
    src/
        bin/
            local_prover.rs
        chips/
            mod.rs
        circuit.rs
    Cargo.toml
    Sindri.json
```

The file `Sindri.json` specifies what you have named your package and how to reference the circuit
```
{
    "PACKAGE_NAME": "my-circuit",
    "CLASS_NAME": "circuit::myCircuit",
    "DEGREE": "12",
    "HALO2_BASE_VERSION": "0.2.2"
}
```