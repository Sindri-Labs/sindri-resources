use std::time::Duration;
use serde::{Deserialize,Serialize};
use serde_json::Value;
use reqwest::{
    Client, 
    header::{HeaderMap, HeaderValue}
};

// Structs to decode JSON endpoint responses
#[derive(Debug,Clone,Deserialize)]
pub struct CircuitResponse {
    pub circuit_id: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct ProofResponse {
    pub proof_id: String,
}

// Functions which return Reqwest Header
pub fn headers_json(api_key: &str) -> HeaderMap {
    let mut headers_json = HeaderMap::new();
    headers_json.insert("Accept", "application/json".parse().unwrap());
    headers_json.insert("Authorization", HeaderValue::from_str(&format!("Bearer {api_key}").to_string()).unwrap());
    headers_json
}

// Polling loop while circuit compile or proof is in progress
pub async fn poll_status(
    endpoint: String, 
    api_url: &str,
    api_key: &str,
    timeout: i64
) -> Value {
    let client = Client::new();
    for i in 1..timeout {

        let response = client
        .get(format!("{api_url}{endpoint}"))
        .headers(headers_json(api_key))
        .send()
        .await
        .unwrap();
        assert_eq!(&response.status().as_u16(), &200u16, "Expected status code 201");
    
        let data = response.json::<Value>().await.unwrap();
        let status = &data["status"].to_string();
        if ["Ready", "Failed"].iter().any(|&s| status.as_str().contains(s)) {
            println!("Polling exited after {} seconds with status: {}", i, &status);
            return data
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Polling timed out after {} seconds", timeout);
    std::process::exit(1);
}
