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
    timeout: u8,
    task_type: &str
) {
    for _i in 1..timeout {

        // early out logic when status in {Ready, Failed}
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("{} polling timed out",task_type);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    // NOTE: Provide your API Key here
    let api_key = "bjZtEQeFWXAAnNI26LYW6S2Ro46lTC7E";
    
    let api_version = "v1";
    let api_url = format!("https://forge.sindri.app/api/{api_version}/");
    let api_key_querystring = format!("?api_key={api_key}");

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

    // Poll circuit detail unitl it has a status of Ready or Failed
    //poll_status(60, "compile").await;

}
