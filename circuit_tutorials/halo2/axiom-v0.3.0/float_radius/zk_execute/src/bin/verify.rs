use std::{
    fs,
    io::{BufReader,Cursor,Read}
};
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};

use halo2_base::{
    halo2_proofs::{
        SerdeFormat,
        plonk::{verify_proof, VerifyingKey},
        halo2curves::bn256::{Bn256,Fr, G1Affine},
        poly::commitment::Params,
        poly::kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::VerifierSHPLONK,
            strategy::SingleStrategy,
        },

        transcript::{
            Blake2bRead, Challenge255, TranscriptReadBuffer
        },
    }
};
use radius_circuit::{
    circuit_def::RadiusCircuitBuilder,
    gadgets::FixedPointChip,
};


#[tokio::main]
async fn main() {

    println!("Reading proof details locally");
    let mut file = fs::File::open("./data/prove_out.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let proof_data: Value = serde_json::from_str(&data).unwrap();

    let public = &proof_data["public"]["data"].as_array().unwrap();
    //decode the instance from string to a field element
    let instance_str = public[0][0].as_str().unwrap();
    let field_instance = Fr::from_bytes(&general_purpose::STANDARD.decode(instance_str).unwrap().try_into().unwrap()).unwrap();

    //instantiate the Fixed Point Chip (which will dequantize the instance variable)
    let lookup_bits = 12;
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<Fr, PRECISION_BITS>::default(lookup_bits);

    let radius = fixed_point_chip.dequantization(field_instance);
    println!("The claimed radius for this proof: {:?}", radius);
    println!("");

    // download SRS if it doesn't exist in ./data already
    if !std::path::Path::new("./data/kzg_bn254_15.srs").is_file() {
        let srs_link = "https://axiom-crypto.s3.amazonaws.com/challenge_0085/kzg_bn254_15.srs";
        let response = reqwest::get(srs_link).await.unwrap();
        let mut file = std::fs::File::create("./data/kzg_bn254_15.srs").unwrap();
        let mut content =  Cursor::new(response.bytes().await.unwrap());
        std::io::copy(&mut content, &mut file).unwrap();
    }
    let setup_fp = fs::File::open("./data/kzg_bn254_15.srs").expect("can't open setup_fn");
    let mut setup_bufreader = BufReader::new(setup_fp);
    let setup = ParamsKZG::<Bn256>::read(&mut setup_bufreader).expect("can't read setup");

    std::env::set_var("LOOKUP_BITS","12");   
    std::env::set_var("FLEX_GATE_CONFIG_PARAMS", r#"{"strategy":"Vertical","k":13,"num_advice_per_phase":[3,0,0],"num_lookup_advice_per_phase":[1,0,0],"num_fixed":1}"#);
    let verification_key = &proof_data["verification_key"]["data"].as_str().unwrap();
    let b64_data = general_purpose::STANDARD.decode(verification_key).unwrap();
    let vk: VerifyingKey<G1Affine> = VerifyingKey::from_bytes::<RadiusCircuitBuilder<Fr>>(&b64_data[..], SerdeFormat::RawBytesUnchecked).unwrap();

    let proof = &proof_data["proof"]["data"].as_str().unwrap();
    let b64_data = general_purpose::STANDARD.decode(proof).unwrap();
    let mut transcript =  Blake2bRead::<_, G1Affine, Challenge255<_>>::init(&b64_data[..]);

    let strategy = SingleStrategy::new(&setup);

    println!("Verifying Proof + Public");
    let verify_status = verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<_, G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(&setup, &vk, strategy, &[&[&[field_instance]]], &mut transcript);
    if !verify_status.is_ok() { // function technically executes, but proof is incorrect
        eprintln!("Verify failed!");
        std::process::exit(1);
    }

    println!("Verification Successful!")


}
