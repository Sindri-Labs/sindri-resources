#![no_main]
pub mod utils;
pub use utils::{Input, Output};

#[jolt::provable]
fn fib(input: Input) -> Output {
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    let mut sum: u128;
    for _ in 1..input.n {
        sum = a + b;
        a = b;
        b = sum;
    }

    let out: Output = Output { output: b };

    out
}