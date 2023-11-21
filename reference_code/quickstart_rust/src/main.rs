use std::{
    env,
    fs::File,
    io::Read,
    time::Duration
};
use serde::Deserialize;
use reqwest::{
    Client, 
    header::HeaderMap,
    multipart::Part
};


// Structs to decode JSON endpoint responses
#[derive(Debug,Clone,Deserialize)]
pub struct CircuitResponse {
    circuit_id: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct ProofResponse {
    proof_id: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct PollResponse {
    status: String,
    public: Option<Vec<String>>
}

// Functions which return Reqwest Header
fn headers_json() -> HeaderMap {
    let mut headers_json = HeaderMap::new();
    headers_json.insert("Accept", "application/json".parse().unwrap());
    headers_json
}
fn headers_multipart() -> HeaderMap {
    let mut headers_multipart = HeaderMap::new();
    headers_multipart.insert("Accept", "multipart/form-data".parse().unwrap());
    headers_multipart
}


// Polling loop while circuit compile or proof is in progress
async fn poll_status(
    endpoint: String, 
    api_url: &String,
    api_key_querystring: &String,
    timeout: i64
) -> PollResponse {
    let client = Client::new();
    for i in 1..timeout {

        let response = client
        .get(format!("{api_url}{endpoint}{api_key_querystring}"))
        .headers(headers_json())
        .send()
        .await
        .unwrap();
        assert_eq!(&response.status().as_u16(), &200u16, "Expected status code 201");
    
        let data = response.json::<PollResponse>().await.unwrap();
        let status = &data.status;
        if ["Failed", "Ready"].contains(&status.as_str()) {
            println!("Polling exited after {} seconds with status: {}", i, &status);
            return data
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Polling timed out after {} seconds", timeout);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {

    let api_key: String = env::var("SINDRI_API_KEY").unwrap();
    let api_key_querystring: String = format!("?api_key={api_key}");
    
    let api_version: &str = "v1";
    let api_url: String = format!("https://forge.sindri.app/api/{api_version}/");
    
    // Create new circuit
    println!("1. Creating circuit...");
    let client = Client::new();
    let mut response = client
        .post(format!("{api_url}circuit/create{api_key_querystring}"))
        .headers(headers_json())
        .form(&[
            ("circuit_name", "multiplier_example"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");
    let circuit_id = response.json::<CircuitResponse>().await.unwrap().circuit_id; 

    // Load the circuit .tar.gz file and create multipart form
    let mut file = File::open("../circuit_database/circom/multiplier2").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).expect("Unable to read tar file");
    let part = Part::bytes(contents).file_name("filename.filetype");
    let upload = reqwest::multipart::Form::new().part("files", part);

    // Upload the circuit file
    let client = Client::new();
    response = client
    .post(format!("{api_url}circuit/{circuit_id}/uploadfiles{api_key_querystring}"))
        .headers(headers_multipart())
        .multipart(upload)
        .send()
        .await
        .unwrap();
    assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");

    // Initiate circuit compilation
    response = client
    .post(format!("{api_url}circuit/{circuit_id}/compile{api_key_querystring}"))
    .headers(headers_json())
    .send()
    .await
    .unwrap();
    assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");

    // Poll circuit detail until it has a status of Ready or Failed
    let circuit_data = poll_status(
        format!("circuit/{circuit_id}/detail"),
        &api_url,
        &api_key_querystring,
        600).await;
    if circuit_data.status == "Failed" {
        println!("Circuit compilation failed.");
        std::process::exit(1);
    }
    println!("Circuit compilation succeeded!");    

    // Initiate proof generation.
    println!("2. Proving circuit...");
    let proof_input = r#"{"a": 7, "b": 42}"#;
    let map = serde_json::json!({"proof_input": proof_input});

    let response = client
    .post(format!("{api_url}circuit/{circuit_id}/prove{api_key_querystring}"))
    .headers(headers_json())
    .json(&map)
    .send()
    .await
    .unwrap();
    assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");
    
    let proof_id = response.json::<ProofResponse>().await.unwrap().proof_id;

    // Poll proof detail until it has a status of Ready or Failed
    let proof_data = poll_status(
        format!("proof/{proof_id}/detail"),
        &api_url,
        &api_key_querystring,
        600).await;
    if &proof_data.status == "Failed" {
        println!("Proving failed.");
        std::process::exit(1);
    }

    // Retrieve output from the proof.
    let output_signal = proof_data.public.unwrap_or(["none".to_owned()].to_vec());
    println!("Circuit proof output signal: {}", output_signal.first().unwrap());

}
