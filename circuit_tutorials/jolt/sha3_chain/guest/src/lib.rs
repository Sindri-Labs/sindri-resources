#![no_main]
pub mod utils;
pub use utils::{Input, Output};

use sha3::{Digest, Keccak256};


#[jolt::provable]
fn sha3_chain(pre_image: Input) -> Output {
    let mut hash = pre_image.input;
    for _ in 0..pre_image.num_iters {
        let mut hasher = Keccak256::new();
        hasher.update(pre_image.input);
        let res = &hasher.finalize();
        hash = Into::<[u8; 32]>::into(*res);
    }

    let output = Output {output: hash};
    
    output
}