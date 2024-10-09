mod utils;
mod sindri;

use ark_bn254::{Bn254, Fr};
use dotenvy::dotenv;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use sindri::{compile_guest_code, prove_guest_code, headers_json};
use utils::{JsonProofData, deserialize_jolt_proof_data_from_base64};
use jolt::{
    jolt::vm::{
        rv32i_vm::RV32IJoltVM,
        Jolt,
    },
    poly::commitment::hyperkzg::HyperKZG
};

#[tokio::main]
async fn main() {
    // Obtain the user's API key from the .env file
    dotenv().expect("Failed to read .env file");
    let api_key: String = std::env::var("SINDRI_API_KEY").unwrap();

    // Create a headers map with the API key.
    let header = headers_json(&api_key);

    // Upload the guest code to Sindri and compile it to RISCV bytecode.
    compile_guest_code(header.clone()).await;

    // Uploads an input to the guest code consisting of an array of 32 u64 integers
    // and a usize value. Proof artifacts are saved as a JSON file in the /data/
    // directory.
    let input_path: &str = "input.json";
    prove_guest_code(input_path, header).await;

    // Verifies the proof.
    let proof_path: &str = "./data/prove_out.json";
    let mut file = File::open(proof_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let proof_details: Value = serde_json::from_str(&contents).unwrap();

    let json_data: JsonProofData =
        serde_json::from_value(proof_details["proof"].clone()).unwrap();

    let (jolt_proof_struct, jolt_preprocessing_struct) =
        deserialize_jolt_proof_data_from_base64::<Fr, HyperKZG<Bn254>>(json_data);
    

    let preprocessing = RV32IJoltVM::preprocess(
        jolt_preprocessing_struct.bytecode,
        jolt_preprocessing_struct.memory_init,
        1 << 20,
        1 << 20,
        1 << 22,
    );

    println!("created preprocessing struct");

    let verification_result = RV32IJoltVM::verify(
        preprocessing,
        jolt_proof_struct.proof,
        jolt_proof_struct.commitments,
        None,
    );

    // This data corresponds to the public inputs to the zkVM.  It contains the
    // inputs and outputs of the guest code, a boolean field indicating whether the
    // guest code panicked during execution, and the memory layout of the zkVM.
    let public_data = proof_details["public"].clone();

    if verification_result.is_ok() {
        println!("Proof is valid");
        println!("zkVM public inputs: {}", public_data);
    } else {
        println!("Proof is invalid");
    }
}
