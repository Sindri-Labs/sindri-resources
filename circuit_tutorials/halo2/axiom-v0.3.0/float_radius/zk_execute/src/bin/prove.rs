use zk_execute::{
    headers_json,
    poll_status
};
use std::{
    env,
    fs,
    io::{BufWriter,Read,Write}
};
use reqwest::Client;
use serde_json::Value;


#[tokio::main]
async fn main() {

    let api_key: String = env::var("SINDRI_API_KEY").unwrap();
    let api_url_prefix: &str = "https://sindri.app/api/";
    
    let api_version: &str = "v1/";
    let api_url: String = api_url_prefix.to_owned()  + api_version;

    println!("Reading circuit details locally");
    let mut file = fs::File::open("./data/compile_out.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let circuit_data: Value = serde_json::from_str(&data).unwrap();
    let circuit_id = &circuit_data["circuit_id"].as_str().unwrap();

    // Initiate proof generation.
    println!("Reading proof input from example-input.json");
    let mut proof_input_file = fs::File::open("example-input.json").unwrap();
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
    let file = fs::File::create("./data/prove_out.json").unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &proof_data).unwrap();
    writer.flush().unwrap(); 

}
