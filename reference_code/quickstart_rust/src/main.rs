use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    multipart::Part,
    Client,
};
use serde_json::Value;
use std::{io::Cursor, option_env, time::Duration};

// Functions which return Reqwest Header
fn headers_json(api_key: &str) -> HeaderMap {
    let mut headers_json = HeaderMap::new();
    headers_json.insert("Accept", "application/json".parse().unwrap());
    headers_json.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {api_key}").to_string()).unwrap(),
    );
    headers_json
}

// Polling loop while circuit compile or proof is in progress
async fn poll_status(endpoint: String, api_url: &str, api_key: &str, timeout: i64) -> Value {
    let client = Client::new();
    for i in 1..timeout {
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
            println!(
                "Polling exited after {} seconds with status: {}",
                i, &status
            );
            return data;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Polling timed out after {} seconds", timeout);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let api_key: &str = option_env!("SINDRI_API_KEY").unwrap_or("");
    let api_url_prefix: &str = option_env!("SINDRI_API_URL").unwrap_or("https://sindri.app/api/");

    let api_version: &str = "v1/";
    let api_url: String = api_url_prefix.to_owned() + api_version;

    let mut contents = Vec::new();
    {
        // has to be scoped so that contents can be accessed after written to
        let buffer = Cursor::new(&mut contents);
        let enc = GzEncoder::new(buffer, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.append_dir_all("multiplier2/", "../../circuit_database/circom/multiplier2/")
            .unwrap();
    }
    let part = Part::bytes(contents).file_name("filename.filetype");
    let upload = reqwest::multipart::Form::new().part("files", part);

    // Create new circuit
    println!("1. Creating circuit...");
    let client = Client::new();
    let response = client
        .post(format!("{api_url}circuit/create"))
        .headers(headers_json(api_key))
        .multipart(upload)
        .send()
        .await
        .unwrap();
    assert_eq!(
        &response.status().as_u16(),
        &201u16,
        "Expected status code 201"
    );
    let response_body = response.json::<Value>().await.unwrap();
    let circuit_id = response_body["circuit_id"].as_str().unwrap();

    // Poll circuit detail until it has a status of Ready or Failed
    let circuit_data = poll_status(
        format!("circuit/{circuit_id}/detail"),
        &api_url,
        api_key,
        600,
    )
    .await;
    if circuit_data["status"].as_str().unwrap().contains("Failed") {
        println!("Circuit compilation failed.");
        std::process::exit(1);
    }
    println!("Circuit compilation succeeded!");

    // Initiate proof generation.
    println!("2. Proving circuit...");
    let proof_input = r#"{"a": 7, "b": 42}"#;
    let map = serde_json::json!({"proof_input": proof_input});

    let response = client
        .post(format!("{api_url}circuit/{circuit_id}/prove"))
        .headers(headers_json(api_key))
        .json(&map)
        .send()
        .await
        .unwrap();
    assert_eq!(
        &response.status().as_u16(),
        &201u16,
        "Expected status code 201"
    );

    let response_body = response.json::<Value>().await.unwrap();
    let proof_id = response_body["proof_id"].as_str().unwrap();

    // Poll proof detail until it has a status of Ready or Failed
    let proof_data = poll_status(format!("proof/{proof_id}/detail"), &api_url, api_key, 600).await;
    if proof_data["status"].as_str().unwrap().contains("Failed") {
        println!("Proving failed.");
        std::process::exit(1);
    }

    // Retrieve output from the proof.
    let output_signal = &proof_data["public"];
    println!("Circuit proof output signal: {}", output_signal);
}
