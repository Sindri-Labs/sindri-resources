use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};

use halo2_base::{
    halo2_proofs::{
        SerdeFormat,
        plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, VerifyingKey, ProvingKey},
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        halo2curves::serde::SerdeObject,
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
use ark_std::{end_timer, start_timer};  

use axiom_eth::{
    rlp::builder::RlcThreadBuilder,
    block_header::EthBlockHeaderChainCircuit
};


fn main() {

    //let cmd = std::env::args().nth(1).expect("no cmd given");

    //if cmd == "genkeys" {
    let k = 14;

    let input = EthBlockHeaderChainCircuit::<Fr>::default();
    let circuit = input.create_circuit(RlcThreadBuilder::keygen(), None);

    let params = gen_srs(k);

    let vk_time = start_timer!(|| "vk gen");
    let vk = keygen_vk(&params, &circuit).expect("vk should not fail");
    end_timer!(vk_time);
    let pk_time = start_timer!(|| "pk gen");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("pk should not fail");
    end_timer!(pk_time);
        
    //let save_time = start_timer!(|| "save keys");
    //io::stdout().write_all(&vk.to_bytes(SerdeFormat::RawBytes)).expect("can't write pk to stdout");
    //io::stdout().write_all(&pk.to_bytes(SerdeFormat::RawBytes)).expect("can't write pk to stdout");
    //end_timer!(save_time);
    //}

     
    //if cmd == "prove" {
    let break_points = circuit.circuit.break_points.take();
    let input_fn = "default_blocks_goerli.json";
    let input = EthBlockHeaderChainCircuit::<Fr>::from_json(&input_fn);
    let circuit = input.create_circuit(RlcThreadBuilder::prover(), Some(break_points));
    let instance = circuit.instance();

    //let proof_fn = std::env::args().nth(5).expect("no proof_fn given");
    //let mut proof_fp = File::create(proof_fn).expect("can't open proof_fn");
    //let instance_fn = std::env::args().nth(6).expect("no instance_fn given");
    //let instance_fp = File::create(instance_fn).expect("can't open instance_fn");

    let pf_time = start_timer!(|| "proof gen");
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
    end_timer!(pf_time);


        /*proof_fp.write_all(&proof).expect("can't write proof to proof_fp");

        {
            let mut instance_wtr = BufWriter::new(instance_fp);
            let vecsize: [u8; 8] = instance.len().to_le_bytes();
            instance_wtr.write_all(&vecsize).unwrap();
            for el in instance.iter() {
                el.write_raw(&mut instance_wtr).unwrap();
            }
            instance_wtr.flush().unwrap();
        }*/

    //}
    /* 
    if cmd == "verify" {
        let setup_fn = std::env::args().nth(2).expect("no setup_fn given");
        let mut setup_fp = File::open(setup_fn).expect("can't open setup_fn");
        let setup = ParamsKZG::<Bn256>::read(&mut setup_fp).expect("can't read setup");

        let vk_fn = std::env::args().nth(3).expect("no vk_fn given");
        let mut vk_fp = File::open(vk_fn).expect("can't open vk_fn");
        let vk = VerifyingKey::read::<std::fs::File, CLASS_NAME<_>>(&mut vk_fp, SerdeFormat::RawBytes).expect("can't read vk");

        let proof_fn = std::env::args().nth(4).expect("no proof_fn given");
        let proof_fp = File::open(&proof_fn).expect("can't open proof_fn");
        // let mut proof = Vec::<u8>::new();
        // proof_fp.read_to_end(&mut proof).expect("can't read proof");

        let verifier_params = setup.verifier_params();
        let strategy = SingleStrategy::new(&setup);

        let mut transcript = Blake2bRead::<File, G1Affine, Challenge255<_>>::init(proof_fp);
        // let mut transcript =  Blake2bRead::<_, G1Affine, Challenge255<_>>::init(&proof[..]);

        let instance_fn = std::env::args().nth(5).expect("no instance_fn given");
        let instance_fp = File::open(instance_fn).expect("can't open instance_fn");
        let mut instance_rdr = BufReader::new(instance_fp);
        let mut veclen = [0u8; 8];
        instance_rdr.read_exact(&mut veclen).unwrap();
        let veclen = u64::from_le_bytes(veclen);
        let instance: Vec<Fr> = (0..veclen)
            .map(|_| Fr::read_raw(&mut instance_rdr).unwrap())
            .collect();

        // verify the proof to make sure everything is ok
        let anyinst = instance.len() > usize::try_from(0).unwrap();
        if anyinst{
            assert!(verify_proof::<
                KZGCommitmentScheme<Bn256>,
                VerifierSHPLONK<'_, Bn256>,
                Challenge255<G1Affine>,
                Blake2bRead<_, G1Affine, Challenge255<G1Affine>>,
                SingleStrategy<'_, Bn256>,
            >(verifier_params, &vk, strategy, &[&[&instance]], &mut transcript)
            .is_ok());
        } else {
            assert!(verify_proof::<
                KZGCommitmentScheme<Bn256>,
                VerifierSHPLONK<'_, Bn256>,
                Challenge255<G1Affine>,
                Blake2bRead<_, G1Affine, Challenge255<G1Affine>>,
                SingleStrategy<'_, Bn256>,
            >(verifier_params, &vk, strategy, &[&[]], &mut transcript)
            .is_ok());
        }    


        return;
    } */
}