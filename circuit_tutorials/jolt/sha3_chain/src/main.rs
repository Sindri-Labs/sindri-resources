mod sindri;
mod utils;

use ark_bn254::{Bn254, Fr};
use dotenvy::dotenv;
use jolt::{
    jolt::vm::{rv32i_vm::RV32IJoltVM, Jolt},
    poly::commitment::hyperkzg::HyperKZG,
};
use serde_json::Value;
use sindri::{compile_guest_code, headers_json, prove_guest_code};
use std::{fs::File, io::Read};
use utils::{deserialize_jolt_proof_data_from_base64, JsonProofData};

#[tokio::main]
async fn main() {
    // Obtain the user's API key from the .env file.
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

    // Import the data necessary for verifying a Jolt proof.
    let proof_path: &str = "./data/prove_out.json";
    let mut file = File::open(proof_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let proof_details: Value = serde_json::from_str(&contents).unwrap();

    let json_data: JsonProofData = serde_json::from_value(proof_details["proof"].clone()).unwrap();

    // Separate out the proof and preprocessing components from the JSON data into their respective structs.
    let (jolt_proof_struct, jolt_preprocessing_struct) =
        deserialize_jolt_proof_data_from_base64::<Fr, HyperKZG<Bn254>>(json_data);

    // The preprocessing struct is constructed the same way here as it is during
    // proof generation in the Jolt zkVM.
    let preprocessing = RV32IJoltVM::preprocess(
        jolt_preprocessing_struct.bytecode,
        jolt_preprocessing_struct.memory_init,
        1 << 20,
        1 << 20,
        1 << 22,
    );

    let verification_result = RV32IJoltVM::verify(
        preprocessing,
        jolt_proof_struct.proof,
        jolt_proof_struct.commitments,
        None,
    );

    // This data corresponds to the public inputs for the zkVM.  It contains the
    // inputs and outputs of the program computed in the guest code, a boolean field
    // indicating whether the guest code panicked during execution, and the
    // memory layout of the zkVM.
    let public_data = proof_details["public"].clone();

    if verification_result.is_ok() {
        println!("Proof is valid");
        println!("zkVM public inputs: {}", public_data);
    } else {
        println!("Proof is invalid");
    }
}
