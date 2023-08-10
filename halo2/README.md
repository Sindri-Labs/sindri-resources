# Sample Halo2 Circuits

In this directory, we provide a handful of Halo2 circuits which have already been prepared to use with Forge.  You may find them useful as a reference when writing your own circuits; or you may use these as circuit uploads if you are just beginning to try out Forge.

All of the tar files are compressed versions of the corresponding folder.  Rather than uploading the entire directory to Forge, you will upload it's compressed `.tar.gz` file.  In order to request a proof for one of these circuits, consult the `input.json` file in the corresponding folder.  

It is allowed (but unneccesary) to include extraneous files in your compressed directory such as readme's, example inputs, and unused code.  However, you should not include any executables from your Cargo build process with your upload (e.g. the `debug/` or `target/` directories), as they are unnecessary for Forge purposes and will slow the upload.

See Sindri's GitBook for more complete instructions. 

## Available Circuits

### v0.2.2

| Name | Size (DEGREE) | Original Source | Functionality | 
| ---- | ---- | --------------- | ------------- | 
|mult-example| 11 | [halo2-lib's benchmarking](https://github.com/axiom-crypto/halo2-lib/blob/v0.2.2/halo2-base/benches/mul.rs) | Performs $b*c$ 120 times with no public output (120 is a tunable parameter to time proofs for circuits of various sizes)|
|storage_proof| 17 |[axiom-eth repo](https://github.com/axiom-crypto/axiom-eth/tree/v0.2.0) | Produces a ZKP verifying the validity of an Ethereum storage query |

### v0.3.0

| Name | Size (DEGREE) | Original Source | Functionality | 
| ---- | ---- | --------------- | ------------- |
| quadratic_circuit | 10 | Adapted from [halo2-scaffold](https://github.com/axiom-crypto/halo2-scaffold) | Computes $x^2+72$ with $x$ a public input | 
| float_radius | 13 | [Fixed point gadget from DCMMC's repo](https://github.com/DCMMC/halo2-scaffold/tree/main/src/gadget) | Computes $\sqrt{x^2+y^2}$ for two private **floating point** inputs
| axiom_header_goerli | 14 | [axiom-eth repo](https://github.com/axiom-crypto/axiom-eth/tree/axiom-dev-0406) | Produces a ZKP verifying the validity of a sequence of Goerli block headers

## Circuit Requirements

- Our proving package is built with Rust toolchain version `nightly-2022-10-28`.  To ensure compatibility, try compiling your circuit with the same version.
- Upload all of your circuit code (and auxilliary files) in one compressed gzip tarfile.
- See the individual version directories (`axiom-v0.2.2/` or `axiom-v0.3.0/`) for specific requirements by circuit type.


