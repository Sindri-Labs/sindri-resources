use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub n: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub output: u128,
}