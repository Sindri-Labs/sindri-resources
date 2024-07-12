pub mod utils;

use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

use crate::utils::{run, ProofTuple};

use rand::rngs::OsRng;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use plonky2::field::types::Field;
use plonky2::field::types::PrimeField64;

use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CommonCircuitData, VerifierOnlyCircuitData};

use plonky2::plonk::proof::ProofWithPublicInputs;

pub const D: usize = 2;
pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

pub struct HashCircuit {
    pub proof: ProofWithPublicInputs<F, C, D>,
    pub common: CommonCircuitData<F, D>,
    pub verifier_only: VerifierOnlyCircuitData<C, D>,
}
// User can change the type of the input data to match their own data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputData {
    pub inputs: Vec<u64>,
}

// user is responsible for ensuring that the input are deserialized and converted into the correct types
fn from_json(path: &str) -> [F; 4] {
    let inputs = std::fs::read_to_string(path).unwrap();
    let mut data: InputData = serde_json::from_str(&inputs).unwrap();

    let circuit_inputs: Vec<F> = data
        .inputs
        .into_iter()
        .map(|x| F::from_canonical_u64(x))
        .collect::<Vec<F>>()
        .try_into()
        .unwrap();

    circuit_inputs.as_slice().try_into().unwrap()
}

impl HashCircuit {
    pub fn prove(inputs_path: &str) -> Self {
        let rng_seed = OsRng::default().next_u64();
        let mut rng = ChaCha8Rng::seed_from_u64(rng_seed);

        // If the user is importing an input, they will need to specify the path to the file.  If the user is not importing an input, the path will be None.

        let mut init_values = from_json(inputs_path);
        for i in 0..init_values.len() {
            init_values[i] =
                F::from_noncanonical_u128(rng.next_u64() as u128 + (rng.next_u64() as u128) << 64);
        }

        let mut cutoff = (rng.next_u64() % 35) as u128;

        println!(
            "Running program with cutoff = {}, init_value = {:#x} | {:#x} | {:#x} | {:#x}",
            cutoff,
            init_values[0].to_canonical_u64(),
            init_values[1].to_canonical_u64(),
            init_values[2].to_canonical_u64(),
            init_values[3].to_canonical_u64(),
        );

        let data: ProofTuple<F, C, D> = run::<F, C, D>(cutoff, init_values).unwrap(); // Currently not using rayon. Maybe should (it gives some performance gain even on my machine).

        Self {
            proof: data.proof,
            common: data.cd,
            verifier_only: data.vd,
        }
    }
}
