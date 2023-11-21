use std::io::{self, Write};
use halo2_base::{
    halo2_proofs::{
        plonk::{create_proof, keygen_pk, keygen_vk, verify_proof},
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        poly::commitment::{Params,ParamsProver},
        poly::kzg::{
            commitment::{KZGCommitmentScheme,ParamsKZG},
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
    }
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

/* 
fn format_inst(instance: Vec<Fr>) -> Option<&[&[&[<KZGCommitmentScheme<Fr>::Scalar]]]>{
    if instance.len()>0 {
        &[&[&instance]]
    } else {
        &[&[]]
    }
}*/

fn main() {

    let k = 11;

    let circuit = MyCircuit::<Fr>::default(); 
    let params = read_params(k);  

    //let circuit: StandardPlonk<Fr> =  StandardPlonk::from_json("./plonk_input.json") ;
    let vk = keygen_vk(&params, &circuit).expect("something wrong with verifier key");
    let pk = keygen_pk(&params, vk, &circuit).expect("something wrong with proving key");    
    let circuit = MyCircuit::<Fr>::from_json("./mult_in.json") ;
    let instance = circuit.instance();

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

    let anyinst = instance.len() > usize::try_from(0).unwrap();
    if anyinst{
        create_proof::<
            KZGCommitmentScheme<Bn256>,
            ProverSHPLONK<'_, Bn256>,
            Challenge255<G1Affine>,
            _,
            Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
            _,
        >(&params, &pk, &[circuit], &[&[&instance]], OsRng, &mut transcript).expect("something went wrong in proof gen");
    } else {
        create_proof::<
            KZGCommitmentScheme<Bn256>,
            ProverSHPLONK<'_, Bn256>,
            Challenge255<G1Affine>,
            _,
            Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
            _,
        >(&params, &pk, &[circuit], &[&[]], OsRng, &mut transcript).expect("something went wrong in proof gen");
    }

    let proof = transcript.finalize();
    io::stdout().write_all(&proof).expect("can't write proof to stdout");


    /* 
    let verifier_params = params.verifier_params();
    let strategy = SingleStrategy::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(verifier_params, pk.get_vk(), strategy, &[&[]], &mut transcript).expect("something went wrong in verification");
    */
}

