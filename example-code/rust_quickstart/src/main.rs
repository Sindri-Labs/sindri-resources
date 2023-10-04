use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use serde::Deserialize;

use reqwest::{
    header::HeaderMap,
    Client, 
    multipart::Part
};


// Structs to decode JSON endpoint responses
#[derive(Debug,Clone,Deserialize)]
pub struct CreateResponse {
    circuit_id: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct ProofResponse {
    proof_id: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct PollResponse {
    status: String,
}

// Functions which return Reqwest Header
fn headers_json() -> HeaderMap {
    let mut headers_json = HeaderMap::new();
    headers_json.insert("Accept", "application/json".parse().unwrap());
    //headers_json.insert(
    //    "content-Type",
    //    "application/json".parse().unwrap(),
    //);
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
    timeout: i64,
    client: &reqwest::Client
) {
    for _i in 1..timeout {

        let response = client
        .get(format!("{api_url}{endpoint}{api_key_querystring}"))
        .headers(headers_json())
        .send()
        .await
        .unwrap();
        assert_eq!(&response.status().as_u16(), &200u16, "Expected status code 201");
    
        let status = response.json::<PollResponse>().await.unwrap().status;
        if status == "Failed" {
            println!("fail");
            std::process::exit(1);
        }
        else if status == "Ready" {
            println!("ready");
            return
        }
        
        // early out logic when status in {Ready, Failed}
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("polling timed out");
    std::process::exit(1);
}

#[tokio::main]
async fn main() {

    let api_key: String = env::var("FORGE_API_KEY").unwrap();
    let api_key_querystring: String = format!("?api_key={api_key}");
    
    let api_version: &str = "v1";
    let api_url: String = format!("https://forge.sindri.app/api/{api_version}/");
    
    
    // Create new circuit
    println!("1. Creating circuit...");
    let client = reqwest::Client::new();
    let mut response = client
        .post(format!("{api_url}circuit/create{api_key_querystring}"))
        .headers(headers_json())
        .form(&[
            ("circuit_name", "multiplier_example"),
            ("circuit_type", "Circom C Groth16 bn254"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");
    let circuit_id = response.json::<CreateResponse>().await.unwrap().circuit_id; // Obtain circuit_id
    println!("{:?}",&circuit_id);
    // Load the circuit .tar.gz file and load as multipart form
    let mut file = File::open("../../circom/multiplier2.tar.gz").unwrap();
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
    poll_status(
        format!("circuit/{circuit_id}/detail"),
        &api_url,
        &api_key_querystring,
        600,
        &client).await;
    
    let proof_input = r#"{"a": 7, "b": 47}"#;
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
    poll_status(
        format!("proof/{proof_id}/detail"),
        &api_url,
        &api_key_querystring,
        600,
        &client).await;

    
    
}
