use halo2_base::halo2_proofs::{
    plonk::{create_proof, keygen_pk, keygen_vk, verify_proof},
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    poly::kzg::{
        commitment::KZGCommitmentScheme,
        multiopen::{ProverSHPLONK, VerifierSHPLONK},
        strategy::SingleStrategy,
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use rand_core::OsRng;

use radius_circuit::{
    circuit_def::CircuitInput,
    gadgets::{FixedPointChip,FixedPointInstructions}
};
use halo2_base::gates::builder::GateThreadBuilder;
use halo2_base::utils::fs::gen_srs;


fn main() {

    std::env::set_var("RUST_BACKTRACE","1");
    std::env::set_var("LOOKUP_BITS","12");

    let setup = gen_srs(13);

    let input = CircuitInput::<Fr>::default(); 
    let circuit = input.create_circuit(GateThreadBuilder::keygen(), None);

    let vk = keygen_vk(&setup, &circuit).expect("vk should not fail");
    let bk = circuit.break_points();
    let pk = keygen_pk(&setup, vk, &circuit).expect("toubles");

    let input = CircuitInput::<Fr>::from_json("input.json"); 
    let circuit = input.create_circuit(GateThreadBuilder::prover(), Some(bk));
    let instances = circuit.instance();

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        _,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
        _,
    >(&setup, &pk, &[circuit], &[&[&instances]], OsRng, &mut transcript).expect("something went wrong in proof gen");
    let proof = transcript.finalize();
    let mut transcript =  Blake2bRead::<_, G1Affine, Challenge255<_>>::init(&proof[..]);
    
    let strategy = SingleStrategy::new(&setup);
    assert!(verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<_, G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(&setup, pk.get_vk(), strategy, &[&[&instances]], &mut transcript)
    .is_ok());


    //instantiate the Fixed Point Chip to dequantize the instance variables
    let lookup_bits = std::env::var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    const PRECISION_BITS: u32 = 32;
    let fixed_point_chip = FixedPointChip::<Fr, PRECISION_BITS>::default(lookup_bits);

    let radius = fixed_point_chip.dequantization(instances[0]);
    println!("radius: {:?}", radius);

}
