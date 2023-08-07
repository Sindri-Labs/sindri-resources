# v0.3.0 Circuit Requirements

- Upload all your rust language source code for a circuit
  - include the manifest `Cargo.toml`
  - exclude executable targets
- All code imports must use relative paths
- Your main circuit struct should be public
- Indicate the main circuit upon which we build our prover via the `Sindri.json` file
- Your main circuit should implement the following three functionalities:
  - `circuit::default()`
  - `circuit.from_json(path)`
  - `circuit.break_points()`
  - `circuit.instance()`

For greater clarity regarding the three functions above, either examine the **v0.3.0** examples or see our gitbook walkthrough.

## Structure
In this hypothetical example, the user has a main circuit which is defined in the top level of src which references some inner circuitry defined in the chips folder.  For their offline proof testing, they build `local_prover` and even though Forge doesn't require any of the code in `src/bin/local_prover.rs`, it should be included so that the entire package can be built and referenced by our prover binaries.
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
    config.json   [OPTIONAL]
```

The file `Sindri.json` specifies what you have named your package and how to reference the circuit
```
{
    "package_name": "my-circuit",
    "circuit_path": "circuit::myCircuit"
}
```
The file `config.json` specifies any environment variables that are necessary to define your circuit configuration.  For example, in the `float_radius` example we have
```
{
    "LOOKUP_BITS": "12"
}
```
In our prover binary, we call `std::env::set_var(LOOKUP_BITS, "12");` 



If you do not require any environment variables to be set you can either exclude the file entirely or include an empty JSON structure: `{}`.
