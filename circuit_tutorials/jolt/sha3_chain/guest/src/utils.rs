use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub input: [u8; 32],
    pub num_iters: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub output: [u8; 32],
}
