use dotenvy::dotenv;

use flate2::{write::GzEncoder, Compression};
use base64::{engine::general_purpose, Engine as _};

use reqwest::{
    header::{HeaderMap, HeaderValue},
    multipart::Part,
    Client,
};
use serde_json::{json, Value};
use serde::Deserialize;
use std::io::{BufWriter, Read, Write};
use std::time::Duration;
use std::fs::{File};

use plonky2::plonk::circuit_data::{CommonCircuitData, VerifierCircuitData, VerifierOnlyCircuitData};
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::util::serialization::DefaultGateSerializer;
use plonky2::{iop::target::Target, plonk::config::{GenericConfig, PoseidonGoldilocksConfig}};

pub const D: usize = 2;
pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

#[derive(Deserialize, Debug)]
pub struct JsonProofData {
    pub proof: String,
    pub common: String,
    pub verifier_data: String,
}

#[tokio::main]
async fn main() {
    // Uploads the circuit to Sindri
    use_rust_compile_script().await;

    // Sends an input to the circuit 
    // This input is a vector of 1024 leaves and an index to prove
    let input_path: &str = "input_1024.json";
    use_rust_prove_circuit_script(input_path).await;

    // Verifies the proof
    let proof_path: &str = "./data/prove_out.json";
    verify_proof().await;
}

async fn use_rust_compile_script() {
    dotenv().expect("Failed to read .env file");
    let api_key: String = std::env::var("SINDRI_API_KEY").unwrap();
    let api_url_prefix: &str = "https://sindri.app/api/";
    let api_version: &str = "v1/";
    let api_url: String = api_url_prefix.to_owned()  + api_version;

    let circuit_dir = std::env::current_dir().unwrap().join("merkle_tree");
    let mut contents = Vec::new();
    { // has to be scoped so that contents can be accessed after written to
        let buffer = std::io::Cursor::new(&mut contents);
        let enc = GzEncoder::new(buffer, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.append_dir_all("merkle-tree/",circuit_dir).unwrap();
    }

    let part = Part::bytes(contents).file_name("filename.filetype");
    let upload = reqwest::multipart::Form::new().part("files", part);

    // Create new circuit
    println!("Compiling circuit");
    let client = Client::new();
    let response = client
        .post(format!("{api_url}circuit/create"))
        .headers(headers_json(&api_key))
        .multipart(upload)
        .send()
        .await
        .unwrap();
    assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");
    let response_body = response.json::<Value>().await.unwrap();
    let circuit_id = response_body["circuit_id"].as_str().unwrap(); 
    println!("Circuit ID: {:?}", &circuit_id);

    // Poll circuit detail until it has a status of Ready or Failed
    let circuit_data = poll_status(
        format!("circuit/{circuit_id}/detail"),
        &api_url,
        &api_key,
        600).await;
    if circuit_data["status"].as_str().unwrap().contains("Failed") {
        println!("Circuit compilation failed.");
        std::process::exit(1);
    }

    println!("Saving circuit details locally");
    std::fs::create_dir_all("./data").unwrap();
    let file = File::create("./data/compile_out.json").unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &circuit_data).unwrap();
    writer.flush().unwrap();
}

async fn use_rust_prove_circuit_script(json_input_path: &str) {
    {
        dotenv().expect("Failed to read .env file");
        let api_key: String = std::env::var("SINDRI_API_KEY").unwrap();
        let api_url_prefix: &str = "https://sindri.app/api/";
        
        let api_version: &str = "v1/";
        let api_url: String = api_url_prefix.to_owned()  + api_version;
    
        println!("Reading circuit details locally");
        let mut file = File::open("./data/compile_out.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let circuit_data: Value = serde_json::from_str(&data).unwrap();
        let circuit_id = &circuit_data["circuit_id"].as_str().unwrap();
        let circuit_id = circuit_id;
    
        // Initiate proof generation.
        println!("Reading proof input from input.json file");
        let mut proof_input_file = File::open(json_input_path).unwrap();
        let mut proof_input = String::new();
        proof_input_file.read_to_string(&mut proof_input).unwrap();
        let map = serde_json::json!({"proof_input": proof_input});
    
        println!("Requesting a proof");
        let client = Client::new();
        let response = client
            .post(format!("{api_url}circuit/{circuit_id}/prove"))
            .headers(headers_json(&api_key))
            .json(&map)
            .send()
            .await
            .unwrap();
        assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");
        let response_body = response.json::<Value>().await.unwrap();
        let proof_id = response_body["proof_id"].as_str().unwrap();
    
        // Poll proof detail until it has a status of Ready or Failed
        let proof_data = poll_status(
            format!("proof/{proof_id}/detail"),
            &api_url,
            &api_key,
            600).await;
        if proof_data["status"].as_str().unwrap().contains("Failed") {
            println!("Proving failed.");
            std::process::exit(1);
        }
    
        println!("Saving proof details locally");
        let file = File::create("./data/prove_out.json").unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &proof_data).unwrap();
        writer.flush().unwrap(); 
    
    }
}

fn verify_proof(input: &str) {
    let mut file = File::open(input).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let proof_details: Value = serde_json::from_str(&contents).unwrap();
    
    let proof_data: JsonProofData = serde_json::from_value(proof_details["proof"].clone()).unwrap();

    let proof_bytes = general_purpose::STANDARD.decode(proof_data.proof).unwrap();
    let common_bytes = general_purpose::STANDARD.decode(proof_data.common).unwrap();
    let verifier_only_bytes = general_purpose::STANDARD.decode(proof_data.verifier_data).unwrap();

    let default_gate_serializer = DefaultGateSerializer;

    let common = CommonCircuitData::<F, D>::from_bytes(common_bytes, &default_gate_serializer).unwrap();
    let proof = ProofWithPublicInputs::<F, C, D>::from_bytes(proof_bytes, &common).unwrap();
    let verifier_data = VerifierOnlyCircuitData::<C, D>::from_bytes(verifier_only_bytes).unwrap();

    let verifier: VerifierCircuitData<F, C, D> = VerifierCircuitData {
        verifier_only: verifier_data,
        common: common,
    };

    match verifier.verify(proof) {
        Ok(_) => println!("Proof has been verified!"),
        Err(e) => println!("Verification failed with error: {:?}", e),
    }
}

fn headers_json(api_key: &str) -> HeaderMap {
    let mut headers_json = HeaderMap::new();
    headers_json.insert("Accept", "application/json".parse().unwrap());
    headers_json.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {api_key}").to_string()).unwrap(),
    );
    headers_json
}

async fn poll_status(endpoint: String, api_url: &str, api_key: &str, timeout: i64) -> Value {
    let client = Client::new();
    for _i in 1..timeout {
        let response = client
            .get(format!("{api_url}{endpoint}"))
            .headers(headers_json(api_key))
            .send()
            .await
            .unwrap();
        assert_eq!(
            &response.status().as_u16(),
            &200u16,
            "Expected status code 201"
        );

        let data = response.json::<Value>().await.unwrap();
        let status = &data["status"].to_string();
        if ["Ready", "Failed"]
            .iter()
            .any(|&s| status.as_str().contains(s))
        {
            return data;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Polling timed out after {} seconds", timeout);
    std::process::exit(1);
}