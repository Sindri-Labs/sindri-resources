use halo2_base::{
    halo2_proofs::{
        SerdeFormat,
        plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, VerifyingKey, ProvingKey},
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        poly::commitment::{ParamsProver, Params},
        poly::kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },

        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
    },
    utils::{fs::gen_srs},
};
use rand_core::OsRng;  
use axiom_eth::storage::EthBlockStorageCircuit;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};


fn main() {

    let k = 17;
    let params = gen_srs(k);

    let circuit = EthBlockStorageCircuit::<Fr>::default();
    let vk = keygen_vk(&params, &circuit).unwrap();
    let pk = keygen_pk(&params, vk, &circuit).unwrap();


    let circuit = EthBlockStorageCircuit::<Fr>::from_json("diff_block_same_add.json");
    let instance = circuit.instance();
    // create a proof
    println!("beginning prove step");
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        _,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
        _,
    >(&params, &pk, &[circuit], &[&[&instance]], OsRng, &mut transcript).unwrap();
    let proof = transcript.finalize();


    println!("beginning verify");
    let verifier_params = params.verifier_params();
    let strategy = SingleStrategy::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(verifier_params, pk.get_vk(), strategy, &[&[&instance]], &mut transcript)
    .unwrap();

}
