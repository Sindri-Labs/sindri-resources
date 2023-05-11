# Forge Sample Data
Sample data for Sindri Labs' Forge API

# Instructions for compressing your code
- Forge code uploads must be archived in a gzip tarfile (`.tar.gz`).
- Forge expects a tarfile of a directory.
- Even if you have 1 file, place the file a directory and archive that entire directory, not just the single file.

# Circom
## Requirements
- The main component file of your circom circuit must be named `main.circom`
- All code imports must use relative paths

## Structure
```
my_repo/
    main.circom
    supplementary.circom
```
## How to compress properly
To prepare your repo, `my_repo`, for Forge upload, run the following command ***from the parent directory*** of `my_repo`:
```
tar -zcvf my_repo.tar.gz my_repo/
```
*Note: The `my_repo` portion of `my_repo.tar.gz` may be called anything.*

## Invalid compression example
Your repo may only have 1 circom file. Do not compress only the `main.cirom` file. You must still compress the entire repo directory.

Example structure (only 1 circom file):
```
my_repo/
    main.circom
```

Invalid compression:
```bash
# INVALID
cd my_repo/
tar -zcvf my_repo.tar.gz main.circom
```


# Halo2
## Requirements
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

For greater clarity regarding the three functions above, either examine the halo2 examples or see our gitbook walkthroughs.

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
```

To generate your `tar.gz` upload file, follow the same compression instructions listed for Circom.