use plonky2::{iop::target::Target, plonk::config::{GenericConfig, PoseidonGoldilocksConfig}};
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, VerifierOnlyCircuitData, CommonCircuitData};
use plonky2::util::serialization::{Buffer, GateSerializer, IoResult, Read, WitnessGeneratorSerializer, Write, DefaultGateSerializer, DefaultGeneratorSerializer};
use plonky2::plonk::proof::ProofWithPublicInputs;
use serde::{Deserialize, Serialize};
use std::fs;

pub const INITIAL_INPUTS: usize = 2;
pub const PUBLIC_OUTPUTS: usize = 1;
pub const D: usize = 2;

pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

pub struct FibonacciCircuit{
    pub proof: ProofWithPublicInputs<F, C, D>,
    pub verifier_only: VerifierOnlyCircuitData<C, D>,
    pub common: CommonCircuitData<F, D>,
}

//user needs to specify the correct input type.  Should match the type in the from_json function
#[derive(Serialize, Deserialize)]
pub struct InputData {
    pub inputs: Vec<u64>
}

impl FibonacciCircuit {
    pub fn prove(path: &str) -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // The arithmetic circuit.
        let initial_a = builder.add_virtual_target();
        let initial_b = builder.add_virtual_target();
        let mut prev_target = initial_a;
        let mut cur_target = initial_b;
        for _ in 0..99 {
            let temp = builder.add(prev_target, cur_target);
            prev_target = cur_target;
            cur_target = temp;
        }

        // Public inputs are the two initial values (provided below) and the result (which is generated).
        builder.register_public_input(initial_a);
        builder.register_public_input(initial_b);
        builder.register_public_input(cur_target);

        let data = builder.build::<C>();
        
        // We construct the partial witness using inputs from the JSON file
        let input_data = from_json(path);
        let input_targets = data.prover_only.public_inputs.clone();

        let mut pw = PartialWitness::new();
        for i in 0..input_data.len() {
            pw.set_target(input_targets[i], F::from_canonical_u64(input_data[i]));
        }

        let proof_with_pis = data.prove(pw).unwrap();

        Self {
            proof: proof_with_pis,
            verifier_only: data.verifier_only,
            common: data.common,
        }
    }    
}

pub fn from_json(path: &str) -> Vec<u64> {
    let inputs = fs::read_to_string(path).unwrap();
    let data: InputData = serde_json::from_str(&inputs).unwrap();
    
    data.inputs
}
