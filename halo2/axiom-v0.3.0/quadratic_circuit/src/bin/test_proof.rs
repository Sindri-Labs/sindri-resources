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

use quadratic_circuit::circuit_def::CircuitInput;
use halo2_base::gates::builder::{GateThreadBuilder,MultiPhaseThreadBreakPoints};
use halo2_base::utils::fs::gen_srs;

use std::fs::File;

fn main() {

    let setup = gen_srs(8);

    let input = CircuitInput::<Fr>::default(); 
    let circuit = input.create_circuit(GateThreadBuilder::keygen(), None);
    let vk = keygen_vk(&setup, &circuit).expect("vk should not fail");

    let bk = circuit.break_points();
    serde_json::to_writer(File::create("breakpoints.json").unwrap(), &bk).unwrap();

    let pk = keygen_pk(&setup, vk, &circuit).expect("pk should not fail");
    let break_points: MultiPhaseThreadBreakPoints = serde_json::from_reader(File::open("breakpoints.json").unwrap()).unwrap();
    
    let input = CircuitInput::<Fr>::default(); 
    let circuit = input.create_circuit(GateThreadBuilder::prover(), Some(break_points));
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
    
    
    let input = CircuitInput::<Fr>::default(); 
    let circuit = input.create_circuit(GateThreadBuilder::keygen(), None);
    let vk = keygen_vk(&setup, &circuit).expect("vk should not fail");
    let strategy = SingleStrategy::new(&setup);
    assert!(verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<_, G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(&setup, &vk, strategy, &[&[&instances]], &mut transcript)
    .is_ok());

}
