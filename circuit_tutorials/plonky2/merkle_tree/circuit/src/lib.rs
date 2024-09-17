use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierOnlyCircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof,
    },
};

mod merkle_tree;
use merkle_tree::MerkleTree;

use serde::{Deserialize, Serialize};
use std::fs;

pub const D: usize = 2;
pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

// The user configures the InputData struct with the necessary fields.
// For the Merkle Tree circuit, we need the Merkle leafs and the index of the leaf to prove.
#[derive(Serialize, Deserialize)]
pub struct InputData {
    pub inputs: Vec<u64>,
    pub index: usize,
}

// At minimum, we need to implement a method to read the input data from a JSON file.
// We also implement methods to select the index of the leaf for proving inclusion and for 
// caculuating the depth of the Merkle tree.
impl InputData {
    pub fn from_json(path: &str) -> Self {
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        serde_json::from_str(&contents).unwrap()
    }
    pub fn set_leaf_index(&mut self, index: usize) {
        // make sure that the user doesn't select an index that is out of bounds
        assert!(index < self.inputs.len());
        self.index = index;
    }

    pub fn get_layers(&self) -> usize {
        let total_leaves = self.inputs.len();
        let layers = ((total_leaves as f32).log2()) as usize;
        layers
    }
}

// Users should rename the struct depending on the circuit they are proving, but the fields of 
// this struct remain the same.
pub struct MerkleTreeCircuit {
    pub proof: ProofWithPublicInputs<F, C, D>,
    pub verifier_only: VerifierOnlyCircuitData<C, D>,
    pub common: CommonCircuitData<F, D>,
}

// Users only implement one method called "prove" for their circuit struct.
// The prove method should include importing the InputData struct, creating the circuit, creating
//  the partial witness, and proving the circuit.
impl MerkleTreeCircuit {
    pub fn prove(path: &str) -> Self {
        let input_data = InputData::from_json(path);
        let total_leaves = input_data.inputs.len();
        let nr_layers = input_data.get_layers();

        // Integers are converted to Goldilocks field elements using a from_canonical_TYPE method.
        // Create the Merkle tree using the correctly formatted input data.
        let mut leaves: Vec<F> = Vec::new();
        for i in 0..total_leaves {
            let leaf = F::from_canonical_u64(input_data.inputs[i] as u64);
            leaves.push(leaf);
        }
        let tree: MerkleTree = MerkleTree::build(leaves.clone());

        // Choose the leaf to prove in the Merkle inclusion proof by selecting its index in the 
        // base layer of the merkle tree.
        let prove_leaf_index = input_data.index;

        // Create the Merkle proof that corresponds to the leaf whose membership is being proven.
        let merkle_proof = tree.clone().get_merkle_proof(prove_leaf_index);

        // Create the circuit using the verify_merkle_proof_circuit function.
        let (circuit_data, targets) = verify_merkle_proof_circuit(prove_leaf_index, nr_layers);

        // Instantiate the partial witness.
        let mut pw = PartialWitness::new();

        // The Merkle path and corresponding hashes are inputted into the partial witness.
        pw.set_hash_target(targets[0], tree.tree[0][prove_leaf_index]);
        for i in 0..nr_layers {
            pw.set_hash_target(targets[i + 1], merkle_proof[i]);
        }

        // The root hash is a 256 bit value represented with four 64 bit field elements.
        let expected_public_inputs = circuit_data.prover_only.public_inputs.clone();
        for i in 0..4 {
            pw.set_target(expected_public_inputs[i], tree.root.elements[i]);
        }

        // The prove method is called on circuit_data to return a proof with public inputs.
        let proof_with_pis = circuit_data.prove(pw).unwrap();

        // It's best practice to verify the proof after creating it
        let verified = circuit_data.verify(proof_with_pis.clone()).unwrap();
        println!("Verified: {:?}", verified);

        // Sindri always returns this information for every Plonky2 circuit.
        // The proof, the verifier data, and the common data contain all the infomration required 
        // by a verifier to verify the proof.
        Self {
            proof: proof_with_pis,
            verifier_only: circuit_data.verifier_only,
            common: circuit_data.common,
        }
    }
}

pub fn verify_merkle_proof_circuit(
    leaf_index: usize,
    nr_layers: usize,
) -> (
    CircuitData<plonky2::field::goldilocks_field::GoldilocksField, PoseidonGoldilocksConfig, 2>,
    Vec<HashOutTarget>,
) {
    // Public inputs corresponding to nodes in the Merkle path are specified have to specified as 
    // HashOutTarget types.
    let mut targets: Vec<plonky2::hash::hash_types::HashOutTarget> = Vec::new();

    // Plonky2 circuit constructions begins with configurikng a CircuitBuilder struct.
    let config = CircuitConfig::standard_recursion_config();
    let mut builder: CircuitBuilder<plonky2::field::goldilocks_field::GoldilocksField, 2> =
        CircuitBuilder::<F, D>::new(config);

    // The leaf to prove is in the Merkle Tree.
    let leaf_to_prove = builder.add_virtual_hash();
    targets.push(leaf_to_prove);

    // The first hashing outside of the loop, since it uses the leaf_to_prove.
    let merkle_proof_elm = builder.add_virtual_hash();
    targets.push(merkle_proof_elm);

    // The hash output of the base layer must be inputted correctly in either the left or right 
    // position for the next hashing operation.
    let mut next_hash: plonky2::hash::hash_types::HashOutTarget;
    if leaf_index % 2 == 0 {
        next_hash = builder.hash_or_noop::<PoseidonHash>(
            [leaf_to_prove.elements.to_vec(), merkle_proof_elm.elements.to_vec()].concat(),
        );
    } else {
        next_hash = builder.hash_or_noop::<PoseidonHash>(
            [merkle_proof_elm.elements.to_vec(), leaf_to_prove.elements.to_vec()].concat(),
        );
    }

    let mut current_layer_index = leaf_index / 2;

    // Similarly, the hash output of the previous layer must be inputted in the correct position 
    // for the subsequent layers of the Merkle tree to ensure the correct hash is computed.
    for _layer in 1..nr_layers {
        let merkle_proof_elm = builder.add_virtual_hash();
        targets.push(merkle_proof_elm);

        if current_layer_index % 2 == 0 {
            next_hash = builder.hash_or_noop::<PoseidonHash>(
                [next_hash.elements.to_vec(), merkle_proof_elm.elements.to_vec()].concat(),
            );
        } else {
            next_hash = builder.hash_or_noop::<PoseidonHash>(
                [merkle_proof_elm.elements.to_vec(), next_hash.elements.to_vec()].concat(),
            );
        }
        current_layer_index = current_layer_index / 2;
    }
    // This is the expected root value.
    builder.register_public_inputs(&next_hash.elements);

    // This method builds a Plonky2 circuit from the CircuitBuilder struct.
    let data = builder.build::<C>();

    (data, targets)
}
