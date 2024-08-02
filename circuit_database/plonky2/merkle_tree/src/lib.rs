pub mod merkle_tree;

use crate::merkle_tree::MerkleTree;

use plonky2::field::types::Field;
use plonky2::hash::hash_types::{HashOut, RichField};
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CommonCircuitData, VerifierOnlyCircuitData};
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::{
    iop::target::Target,
    plonk::config::{GenericConfig, Hasher, PoseidonGoldilocksConfig},
};

use serde::{Deserialize, Serialize};
use std::fs;

pub const D: usize = 2;

pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

pub struct MerkleTreeCircuit {
    pub proof: ProofWithPublicInputs<F, C, D>,
    pub verifier_only: VerifierOnlyCircuitData<C, D>,
    pub common: CommonCircuitData<F, D>,
}

//user needs to specify the correct input type.  Should match the type in the from_json function
#[derive(Serialize, Deserialize)]
pub struct InputData {
    pub inputs: Vec<u64>,
}

impl MerkleTreeCircuit {
    pub fn prove(path: &str) -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // The arithmetic circuit.
        // The leaf to prove is in the Merkle Tree
        let leaf_to_prove = builder.add_virtual_hash();

        // The elements of the proof
        let merkle_proof_elm_0 = builder.add_virtual_hash();
        let merkle_proof_elm_1 = builder.add_virtual_hash();

        let level1_hash: plonky2::hash::hash_types::HashOutTarget = builder
            .hash_or_noop::<PoseidonHash>(
                [
                    leaf_to_prove.elements.to_vec(),
                    merkle_proof_elm_0.elements.to_vec(),
                ]
                .concat(),
            );

        // This is how we set the constraint of the expected root wrt the computed value
        let expected_root = builder.hash_or_noop::<PoseidonHash>(
            [
                level1_hash.elements.to_vec(),
                merkle_proof_elm_1.elements.to_vec(),
            ]
            .concat(),
        );

        // Registering a hash target actually registers 4 target elements
        builder.register_public_inputs(&leaf_to_prove.elements);
        builder.register_public_inputs(&merkle_proof_elm_0.elements);
        builder.register_public_inputs(&merkle_proof_elm_1.elements);
        builder.register_public_inputs(&expected_root.elements);

        let data = builder.build::<C>();

        // We import the input data and format it to the field element type
        let input_data = from_json(path);
        let input_len = input_data.len();
        let leaves: Vec<F> = input_data
            .clone()
            .into_iter()
            .map(F::from_canonical_u64)
            .collect();

        // Create the partial witness
        let mut pw = PartialWitness::new();

        let tree: MerkleTree  = MerkleTree::build(leaves.clone());

        let merkle_proof_leaf0 = tree.clone().get_merkle_proof(0);

        // Obtain the target elements
        let expected_public_inputs = data.prover_only.public_inputs.clone();

        // Set first hash; the leaf we're trying to prove membership of
        let leaf_to_prove = PoseidonHash::hash_or_noop(&[leaves[0]]);

        for i in 0..input_len {
            pw.set_target(expected_public_inputs[i], leaf_to_prove.elements[i]);
        }

        // Set first elm of proof
        for i in 0..input_len {
            pw.set_target(
                expected_public_inputs[i + 4],
                merkle_proof_leaf0[0].elements[i],
            );
        }

        // Set second elm of proof
        for i in 0..input_len {
            pw.set_target(
                expected_public_inputs[i + 8],
                merkle_proof_leaf0[1].elements[i],
            );
        }

        // Set root
        for i in 0..input_len {
            pw.set_target(expected_public_inputs[i + 12], tree.root.elements[i]);
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
