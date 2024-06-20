use super::{C, D, F};

use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, VerifierCircuitData, CircuitData};
use plonky2::util::serialization::{Buffer, GateSerializer, IoResult, Read, WitnessGeneratorSerializer, Write, DefaultGateSerializer, DefaultGeneratorSerializer};

pub const INITIAL_INPUTS: usize = 2;
pub const PUBLIC_OUTPUTS: usize = 1;

pub struct MyFibCircuit {
    pub data: CircuitData<F, C, D>
}

//user needs to specify the correct input type.  Should match the type in the from_json function
#[derive(Serialize, Deserialize)]
pub struct InputData {
    pub inputs: Vec<u64>
}

impl MyFibCircuit {
    pub fn config() -> CircuitConfig {
        let config = CircuitConfig::standard_recursion_config();
        config
    }

    pub fn build(config: CircuitConfig) -> Self {

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

        // Provide initial values.
        // let mut pw = PartialWitness::new();
        // pw.set_target(initial_a, F::ZERO);
        // pw.set_target(initial_b, F::ONE);

        let data = builder.build::<C>();

        Self{data}
    }

    pub fn from_json(path: &str) -> PartialWitness<F> {
        let mut data: String = String::new();
        let mut file = std::fs::File::open(path).unwrap();
        let _ = file.read_to_string(&mut data).unwrap();
        let data: InputData = serde_json::from_str(&data).unwrap();
        
        let input_targets = self.data.prover_only.public_inputs;
        let mut pw = PartialWitness::new();

        // user needs to provide the correct input type
        for i in data.inputs.len() {
            pw.set_target(input_targets[i], F::from_u64(data.inputs[i]));
        }

        pw
    }
}