# Sample Halo2 Circuits

In this directory, we provide a handful of Halo2 circuits which have already been prepared to use with Sindri.  You may find them useful as a reference when writing your own circuits; or you may use these as circuit uploads if you are just beginning to try out Sindri.

All of the tar files are compressed versions of the corresponding folder.  Rather than uploading the entire directory to Sindri, you will upload it's compressed `.tar.gz` file.  In order to request a proof for one of these circuits, consult the `input.json` file in the corresponding folder.  

It is allowed (but unneccesary) to include extraneous files in your compressed directory such as readme's, example inputs, and unused code.  However, you should not include any executables from your Cargo build process with your upload (e.g. the `debug/` or `target/` directories), as they are unnecessary for Sindri purposes and will slow the upload.

See Sindri's Documentation for more complete instructions. 

## Available Circuits

### v0.2.2

| Name | Size (DEGREE) | Original Source | Functionality | 
| ---- | ---- | --------------- | ------------- | 
|storage_proof| 17 |[axiom-eth repo](https://github.com/axiom-crypto/axiom-eth/tree/v0.2.0) | Produces a ZKP verifying the validity of an Ethereum storage query |

### v0.3.0

| Name | Size (DEGREE) | Original Source | Functionality | 
| ---- | ---- | --------------- | ------------- |
| float_radius | 13 | [Fixed point gadget from DCMMC's repo](https://github.com/DCMMC/halo2-scaffold/tree/main/src/gadget) | Computes $\sqrt{x^2+y^2}$ for two private **floating point** inputs


## Circuit Requirements

- Our proving package is built with Rust toolchain version `nightly-2022-10-28`.  To ensure compatibility, try compiling your circuit with the same version.
- Upload all of your circuit code (and auxilliary files) in one compressed gzip tarfile.
- See the individual version directories (`axiom-v0.2.2/` or `axiom-v0.3.0/`) for specific requirements by circuit type.


