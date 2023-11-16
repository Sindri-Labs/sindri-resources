
use std::io::{self, Write};
use halo2_proofs::{
    halo2curves::{bn256::Bn256,bn256::Fr as Fp,bn256::G1Affine},
    plonk::*,
    poly::{
        kzg::{
            commitment::{KZGCommitmentScheme,ParamsKZG},
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy},
        commitment::Params,
    },
    transcript::{Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer},
};

use rand_core::OsRng;
use mult_example::mult_circ::MyCircuit;

use std::{
    env::var,
    fs::File,
    io::BufReader,
};
pub fn read_params(k: u32) -> ParamsKZG<Bn256> {
    let dir = var("PARAMS_DIR").unwrap_or_else(|_| "./params".to_string());
    ParamsKZG::<Bn256>::read(&mut BufReader::new(
        File::open(format!("{dir}/kzg_bn254_{k}.srs").as_str())
            .expect("Params file does not exist"),
    ))
    .unwrap()
}

fn main() {

    // Set circuit size
    let k = 11;
    //MockProver::run(k, &circuit, vec![]).unwrap().assert_satisfied();
    let circuit = MyCircuit::<Fp>::default(); 
    let params = read_params(k);  

    let vk = keygen_vk(&params, &circuit).expect("something wrong with verifier key");
    let pk = keygen_pk(&params, vk, &circuit).expect("something wrong with proving key");    
    let circuit = MyCircuit::from_json("./mult_in.json") ;

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        _,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
        _,
    >(&params, &pk, &[circuit], &[&[]], OsRng, &mut transcript).expect("something went wrong in proof gen");

    let proof = transcript.finalize();

    io::stdout().write_all(&proof).expect("can't write proof to stdout");

    let strategy = SingleStrategy::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(&params, pk.get_vk(), strategy, &[&[]], &mut transcript).expect("something went wrong in verification");
    
}