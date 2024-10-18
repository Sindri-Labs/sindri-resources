# Using Jolt with Sindri
This Rust script provides a template for using the Jolt zkVM with the Sindri API.  For information on the Jolt zkVM, refer to the [official documentation](https://jolt.a16zcrypto.com/).

### Sha3 Chain Guest Code
The guest code used in this tutorial is adapted from the Sha3-chain example from the [Jolt zkVM examples repository](https://github.com/a16z/jolt/tree/main/examples/sha3-chain/guest).

### Guest Code Input and Output
In order for the guest code to be compatible with Sindri, users must define an `Input` struct and an `Output` struct.  This can be done within the guest code's `lib.rs` file, though by convention these structs are usually defined in a separate `utils.rs` file and subsequently imported into `lib.rs`.  This tutorial uses the latter approach.  Regardless of how users choose to structure their guest code, it is vital that that the `Input` and `Output` structs have public visibility from inside `lib.rs`.  This is done by importing the two structs as follows:
```Rust
pub mod utils;
pub use utils::{Input, Output};
```
The Jolt function signature can only take one input of type `Input` and must return an output of type `Output`:
```Rust
fn sha3_chain(pre_image: Input) -> Output
```

For the Sha3-chain example, the user uploads an input JSON file with two fields:  an array of 32 `u64` integers and a `usize` value for the number of hashing iterations. The output is a hash digest consisting of an array of 32 `u64` integers.

### Instructions
To run the Rust script, change the `sample.env` to `.env` and enter your Sindri API Key.  From the root of the `sha3_chain` directory, run:
```bash
cargo run --release
```

Refer to the [Sindri Jolt Tutorial](https://sindri.app/docs/how-to-guides/frameworks/jolt/) for more information on this example, the proof output file, the zkVM public inputs, and the verification code.