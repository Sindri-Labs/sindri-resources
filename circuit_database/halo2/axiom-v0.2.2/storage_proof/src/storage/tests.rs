use super::*;
use crate::{
    halo2_proofs::{
        dev::MockProver,
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        plonk::*,
        poly::commitment::ParamsProver,
        poly::kzg::{
            commitment::KZGCommitmentScheme,
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
    },
    providers::{GOERLI_PROVIDER_URL, MAINNET_PROVIDER_URL},
    Network,
};
use ark_std::{end_timer, start_timer};
use ethers_core::utils::keccak256;
use halo2_base::utils::fs::gen_srs;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use snark_verifier_sdk::halo2::aggregation::AggregationConfigParams;
use std::{
    env::set_var,
    fs::{self, File},
    io::{BufReader, Write},
};

fn get_test_circuit<F: Field>(network: Network, num_slots: usize) -> EthBlockStorageCircuit<F> {
    let infura_id =
        std::fs::read_to_string("scripts/input_gen/INFURA_ID").expect("Infura ID not found");
    let provider_url = match network {
        Network::Mainnet => format!("{MAINNET_PROVIDER_URL}{infura_id}"),
        Network::Goerli => format!("{GOERLI_PROVIDER_URL}{infura_id}"),
    };
    let provider = Provider::<Http>::try_from(provider_url.as_str())
        .expect("could not instantiate HTTP Provider");
    let addr;
    let block_number;
    match network {
        Network::Mainnet => {
            // cryptopunks
            addr = "0xb47e3cd837dDF8e4c57F05d70Ab865de6e193BBB".parse::<Address>().unwrap();
            block_number = 16356350;
            //block_number = 0xf929e6;
        }
        Network::Goerli => {
            addr = "0xf2d1f94310823fe26cfa9c9b6fd152834b8e7849".parse::<Address>().unwrap();
            block_number = 0x713d54;
        }
    }
    let slot_nums = vec![0u64, 1u64, 2u64, 3u64, 6u64, 8u64];
    let mut slots = (0..4).map(|x| {
        let mut bytes = [0u8; 64];
        bytes[31] = x;
        bytes[63] = 10;
        H256::from_slice(&keccak256(bytes))
    }).collect::<Vec<_>>();
    slots.extend(slot_nums.iter().map(|x| H256::from_low_u64_be(*x)));
    EthBlockStorageCircuit::from_provider(
        &provider,
        block_number,
        addr,
        slots[..num_slots].to_vec(),
        8,
        8,
        network,
    )


}

#[test]
pub fn test_mock_single_eip1186() -> Result<(), Box<dyn std::error::Error>> {
    set_var("STORAGE_CONFIG", "configs/tests/storage.json");
    let k = EthConfigParams::get_storage().degree;

    let circuit = get_test_circuit::<Fr>(Network::Mainnet, 1);
    MockProver::run(k, &circuit, vec![circuit.instance()]).unwrap().assert_satisfied();
    Ok(())
}


#[derive(Serialize, Deserialize)]
struct BenchParams(EthConfigParams, usize);

#[test]
pub fn bench_eip1186() -> Result<(), Box<dyn std::error::Error>> {
    let bench_params_file = File::open("configs/bench/storage.json").unwrap();
    std::fs::create_dir_all("data/bench")?;
    let mut fs_results = File::create("data/bench/storage.csv").unwrap();
    writeln!(fs_results, "degree,total_advice,num_rlc_columns,num_advice,num_lookup,num_fixed,proof_time,verify_time")?;

    let bench_params_reader = BufReader::new(bench_params_file);
    let bench_params: Vec<BenchParams> = serde_json::from_reader(bench_params_reader).unwrap();
    for bench_params in bench_params {
        println!(
            "---------------------- degree = {} ------------------------------",
            bench_params.0.degree
        );

        set_var("STORAGE_CONFIG", "configs/bench/storage_tmp.json");
        let mut f = File::create("configs/bench/storage_tmp.json")?;
        write!(f, "{}", serde_json::to_string(&bench_params.0).unwrap())?;
        //let circuit = get_test_circuit::<Fr>(Network::Mainnet, bench_params.1);
        let circuit = EthBlockStorageCircuit::from_json("./full_block_proof.json");
        println!("{:?}",&circuit.inputs.block_header);
        //let mut bh = &circuit.clone().inputs.block_header;
        //println!("{:?}",hex::serialize(bh, ethers_core::utils::__serde_json::value::Serializer));

        //let mut writer = BufWriter::new(File::create("try2.json"));
        //serde_json::to_writer_pretty(writer, &circuit);

        let instance = circuit.instance();
        //println!("{:?}",&instance);

        let params = gen_srs(bench_params.0.degree);
        let vk = keygen_vk(&params, &circuit)?;
        let pk = keygen_pk(&params, vk, &circuit)?;

        // create a proof
        let proof_time = start_timer!(|| "SHPLONK");
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        create_proof::<
            KZGCommitmentScheme<Bn256>,
            ProverSHPLONK<'_, Bn256>,
            Challenge255<G1Affine>,
            _,
            Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
            _,
        >(&params, &pk, &[circuit.clone()], &[&[&instance]], OsRng, &mut transcript)?;
        let proof = transcript.finalize();
        end_timer!(proof_time);

        let verify_time = start_timer!(|| "Verify time");
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
        end_timer!(verify_time);

        fs::remove_file("configs/bench/storage_tmp.json")?;
        let keccak_advice = std::env::var("KECCAK_ADVICE_COLUMNS")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<usize>()
            .unwrap();
        writeln!(
            fs_results,
            "{},{},{},{:?},{:?},{},{:.2}s,{:?}",
            bench_params.0.degree,
            bench_params.0.num_rlc_columns
                + bench_params.0.num_range_advice.iter().sum::<usize>()
                + bench_params.0.num_lookup_advice.iter().sum::<usize>()
                + keccak_advice,
            bench_params.0.num_rlc_columns,
            bench_params.0.num_range_advice,
            bench_params.0.num_lookup_advice,
            bench_params.0.num_fixed,
            proof_time.time.elapsed().as_secs_f64(),
            verify_time.time.elapsed()
        )
        .unwrap();
    }
    Ok(())
}

#[test]
pub fn bench_evm_eip1186() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, path::Path};

    use rand::SeedableRng;
    use snark_verifier_sdk::{
        evm::{evm_verify, gen_evm_proof_shplonk, gen_evm_verifier_shplonk, write_calldata},
        gen_pk,
        halo2::{
            aggregation::{load_verify_circuit_degree, PublicAggregationCircuit},
            gen_snark_shplonk, 
        },
        CircuitExt,
    };    
    let bench_params_file = File::open("configs/bench/storage.json").unwrap();
    let evm_params_file = File::open("configs/bench/storage_evm.json").unwrap();
    std::fs::create_dir_all("data/bench")?;
    let mut fs_results = File::create("data/bench/storage.csv").unwrap();
    writeln!(fs_results, "degree,total_advice,num_rlc_columns,num_advice,num_lookup,num_fixed,storage_proof_time,evm_proof_time")?;

    let bench_params_reader = BufReader::new(bench_params_file);
    let bench_params: Vec<BenchParams> = serde_json::from_reader(bench_params_reader).unwrap();
    let evm_params_reader = BufReader::new(evm_params_file);
    let evm_params: Vec<AggregationConfigParams> = serde_json::from_reader(evm_params_reader).unwrap();    
    for (bench_params, evm_params) in bench_params.iter().zip(evm_params.iter()) {
        println!(
            "---------------------- degree = {} ------------------------------",
            bench_params.0.degree
        );

        set_var("STORAGE_CONFIG", "configs/bench/storage_tmp.json");
        let mut f = File::create("configs/bench/storage_tmp.json")?;
        write!(f, "{}", serde_json::to_string(&bench_params.0).unwrap())?;
        let mut rng = rand_chacha::ChaChaRng::from_seed([0; 32]);

        let (storage_snark, storage_proof_time) = {
            let k = EthConfigParams::get_storage().degree;
            let circuit = get_test_circuit::<Fr>(Network::Mainnet, bench_params.1);
            let params = gen_srs(k);
            let pk = gen_pk(&params, &circuit, None);
            let storage_proof_time = start_timer!(|| "Storage Proof SHPLONK");
            let snark = gen_snark_shplonk(&params, &pk, circuit, &mut rng, None::<&str>);
            end_timer!(storage_proof_time);
            (snark, storage_proof_time)
        };

        set_var("VERIFY_CONFIG", "configs/bench/storage_evm_tmp.json");
        let mut f = File::create("configs/bench/storage_evm_tmp.json")?;
        write!(f, "{}", serde_json::to_string(&evm_params).unwrap())?;        
        let k = load_verify_circuit_degree();
        let params = gen_srs(k);
        let evm_circuit = PublicAggregationCircuit::new(
            &params,
            vec![storage_snark],
            false,
            &mut rng,
        );
        let pk = gen_pk(&params, &evm_circuit, None);
    
        let instances = evm_circuit.instances();
        let num_instances = instances[0].len();
        let evm_proof_time = start_timer!(|| "EVM Proof SHPLONK");
        let proof = gen_evm_proof_shplonk(&params, &pk, evm_circuit, instances.clone(), &mut rng);
        end_timer!(evm_proof_time);
        fs::create_dir_all("data/storage").unwrap();
        write_calldata(&instances, &proof, Path::new("data/storage/test.calldata")).unwrap();
    
        let deployment_code = gen_evm_verifier_shplonk::<PublicAggregationCircuit>(
            &params,
            pk.get_vk(),
            vec![num_instances],
            Some(Path::new("data/storage/test.yul")),
        );
    
        // this verifies proof in EVM and outputs gas cost (if successful)
        evm_verify(deployment_code, instances, proof);

        fs::remove_file("configs/bench/storage_tmp.json")?;
        let keccak_advice = std::env::var("KECCAK_ADVICE_COLUMNS")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<usize>()
            .unwrap();
        writeln!(
            fs_results,
            "{},{},{},{:?},{:?},{},{:.2}s,{:?}",
            bench_params.0.degree,
            bench_params.0.num_rlc_columns
                + bench_params.0.num_range_advice.iter().sum::<usize>()
                + bench_params.0.num_lookup_advice.iter().sum::<usize>()
                + keccak_advice,
            bench_params.0.num_rlc_columns,
            bench_params.0.num_range_advice,
            bench_params.0.num_lookup_advice,
            bench_params.0.num_fixed,
            storage_proof_time.time.elapsed().as_secs_f64(),
            evm_proof_time.time.elapsed()
        )
        .unwrap();
    }
    Ok(())
}

#[cfg(feature = "evm")]
#[test]
pub fn test_evm_single_eip1186() {
    use std::{fs, path::Path};

    use rand::SeedableRng;
    use snark_verifier_sdk::{
        evm::{evm_verify, gen_evm_proof_shplonk, gen_evm_verifier_shplonk, write_calldata},
        gen_pk,
        halo2::{
            aggregation::{load_verify_circuit_degree, PublicAggregationCircuit},
            gen_snark_shplonk,
        },
        CircuitExt,
    };
    let mut rng = rand_chacha::ChaChaRng::from_seed([0; 32]);

    set_var("STORAGE_CONFIG", "configs/tests/storage.json");
    let storage_snark = {
        let k = EthConfigParams::get_storage().degree;
        let circuit = get_test_circuit::<Fr>(Network::Mainnet, 1);
        let params = gen_srs(k);
        let pk = gen_pk(&params, &circuit, None);
        gen_snark_shplonk(&params, &pk, circuit, &mut rng, None::<&str>)
    };

    std::env::set_var("VERIFY_CONFIG", "./configs/tests/storage_evm.json");
    let k = load_verify_circuit_degree();
    let params = gen_srs(k);
    let evm_circuit = PublicAggregationCircuit::new(
        &params,
        vec![storage_snark],
        false,
        &mut rng,
    );
    let pk = gen_pk(&params, &evm_circuit, None);

    let instances = evm_circuit.instances();
    let num_instances = instances[0].len();
    let proof = gen_evm_proof_shplonk(&params, &pk, evm_circuit, instances.clone(), &mut rng);
    fs::create_dir_all("data/storage").unwrap();
    write_calldata(&instances, &proof, Path::new("data/storage/test.calldata")).unwrap();

    let deployment_code = gen_evm_verifier_shplonk::<PublicAggregationCircuit>(
        &params,
        pk.get_vk(),
        vec![num_instances],
        Some(Path::new("data/storage/test.yul")),
    );

    // this verifies proof in EVM and outputs gas cost (if successful)
    evm_verify(deployment_code, instances, proof);
}

