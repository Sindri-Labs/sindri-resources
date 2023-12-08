use std::{
    env,
    fs,
    io:,Read
};
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use radius_circuit::gadgets::FixedPointChip;
use base64::{engine::general_purpose, Engine as _};


#[tokio::main]
async fn main() {

    let mut file = fs::File::open("./data/prove_out.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    //decode the instance from string to a field element
    let instance_str = "MNMOJwYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    let field_instance = Fr::from_bytes(&general_purpose::STANDARD.decode(instance_str).unwrap().try_into().unwrap()).unwrap();

    //instantiate the Fixed Point Chip (which will dequantize the instance variable)
    let lookup_bits = 12;
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<Fr, PRECISION_BITS>::default(lookup_bits);

    let radius = fixed_point_chip.dequantization(field_instance);
    println!("radius: {:?}", radius);

}
