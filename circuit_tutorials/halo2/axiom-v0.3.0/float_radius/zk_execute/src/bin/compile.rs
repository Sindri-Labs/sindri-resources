use zk_execute::{
    headers_json,
    poll_status
};
use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::{
    Client, 
    multipart::Part
};
use std::{
    env,
    fs::File,
    io::{BufWriter,Cursor,Write}
};
use serde_json::Value;

#[tokio::main]
async fn main() {

    let api_key: String = env::var("SINDRI_API_KEY").unwrap();
    let api_url_prefix: &str = "https://forge.sindri.app/api/";
    let api_version: &str = "v1/";
    let api_url: String = api_url_prefix.to_owned()  + api_version;

    let circuit_dir = std::env::current_dir().unwrap().join("circuit");
    let mut contents = Vec::new();
    { // has to be scoped so that contents can be accessed after written to
        let buffer = Cursor::new(&mut contents);
        let enc = GzEncoder::new(buffer, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.append_dir_all("float_radius/",circuit_dir).unwrap();
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
