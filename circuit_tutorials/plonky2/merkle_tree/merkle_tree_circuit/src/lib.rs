use plonky2::{plonk::{config::{PoseidonGoldilocksConfig, GenericConfig}, circuit_builder::CircuitBuilder, circuit_data::{CircuitConfig, VerifierOnlyCircuitData, CommonCircuitData, CircuitData}, proof}, hash::{hash_types::{RichField, HashOut, HashOutTarget}, poseidon::PoseidonHash}, iop::witness::{WitnessWrite, PartialWitness}, field::{goldilocks_field::GoldilocksField, types::Field}};
use plonky2::plonk::proof::ProofWithPublicInputs;


mod merkle_tree;
use merkle_tree::MerkleTree;

use serde::{Deserialize, Serialize};
use std::fs;

pub const D: usize = 2;
pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

#[derive(Serialize, Deserialize)]
pub struct InputData {
    pub inputs: Vec<u64>,
    pub index: usize,
}

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
        // while total_leaves > 1 {
        //     total_leaves = (total_leaves + 1) / 2;
        //     layers += 1;
        // }
        let layers = ((total_leaves as f32).log2()) as usize;
        layers
    }
}

pub struct MerkleTreeCircuit {
    pub proof: ProofWithPublicInputs<F, C, D>,
    pub verifier_only: VerifierOnlyCircuitData<C, D>,
    pub common: CommonCircuitData<F, D>,
}

impl MerkleTreeCircuit {
    pub fn prove(path: &str) -> Self {
        //let mut targets: Vec<plonky2::hash::hash_types::HashOutTarget> = Vec::new();
        let input_data = InputData::from_json(path);
        let total_leaves = input_data.inputs.len();
        let nr_layers = input_data.get_layers();
    
        // create the merkle tree
        let mut leaves: Vec<F> = Vec::new();
        for i in 0..total_leaves {
          let leaf = F::from_canonical_u64(input_data.inputs[i] as u64);
          leaves.push(leaf);
        }
        let tree: MerkleTree = MerkleTree::build(leaves.clone());
        
        // choose the leaf to prove in the merkle inclusion proof by selecting its index in the base layer of the merkle tree
        let prove_leaf_index = input_data.index;

        // create the merkle proof
        let merkle_proof = tree.clone().get_merkle_proof(prove_leaf_index);

        let (circuit_data, targets) = verify_merkle_proof_circuit(prove_leaf_index, nr_layers);
        

        // create the partial witness
        let mut pw = PartialWitness::new();

        pw.set_hash_target(targets[0], tree.tree[0][prove_leaf_index]);
        for i in 0..nr_layers {
            pw.set_hash_target(targets[i+1], merkle_proof[i]);
        }

        // public input: root of merkle tree
        let expected_public_inputs = circuit_data.prover_only.public_inputs.clone();
        for i in 0..4 {
            pw.set_target(expected_public_inputs[i], tree.root.elements[i]);
        }
        
        // prove the circuit
        let proof_with_pis = circuit_data.prove(pw).unwrap();

        let verified = circuit_data.verify(proof_with_pis.clone()).unwrap();
        println!("Verified: {:?}", verified);

        Self {
            proof: proof_with_pis,
            verifier_only: circuit_data.verifier_only,
            common: circuit_data.common,
        } 
    }
}

pub fn verify_merkle_proof_circuit(leaf_index: usize, nr_layers: usize) -> (CircuitData<plonky2::field::goldilocks_field::GoldilocksField, PoseidonGoldilocksConfig, 2>, Vec<HashOutTarget>) {
    let mut targets: Vec<plonky2::hash::hash_types::HashOutTarget> = Vec::new();
    
    let config = CircuitConfig::standard_recursion_config();
    let mut builder: CircuitBuilder<plonky2::field::goldilocks_field::GoldilocksField, 2> = CircuitBuilder::<F, D>::new(config);
    
    // The leaf to prove is in the Merkle Tree
    let leaf_to_prove = builder.add_virtual_hash();
    targets.push(leaf_to_prove);
  
    // The first hashing outside of the loop, since it uses the leaf_to_prove
    let merkle_proof_elm= builder.add_virtual_hash();
    targets.push(merkle_proof_elm);
  
    let mut next_hash: plonky2::hash::hash_types::HashOutTarget;
    if leaf_index % 2 == 0 {
      next_hash = builder.hash_or_noop::<PoseidonHash>([
        leaf_to_prove.elements.to_vec(), 
        merkle_proof_elm.elements.to_vec()
      ].concat());
    } else {
      next_hash = builder.hash_or_noop::<PoseidonHash>([
        merkle_proof_elm.elements.to_vec(),
        leaf_to_prove.elements.to_vec()
      ].concat());
    }
  
    let mut current_layer_index = leaf_index / 2;
    
    for _layer in 1..nr_layers {
      let merkle_proof_elm= builder.add_virtual_hash();
      targets.push(merkle_proof_elm);
  
      if current_layer_index % 2 == 0 {
        next_hash = builder.hash_or_noop::<PoseidonHash>([
          next_hash.elements.to_vec(), 
          merkle_proof_elm.elements.to_vec()
        ].concat());
      } else {
        next_hash = builder.hash_or_noop::<PoseidonHash>([
          merkle_proof_elm.elements.to_vec(),
          next_hash.elements.to_vec()
        ].concat());
      }
      current_layer_index = current_layer_index/2;
    }
    // This is the expected root value
    builder.register_public_inputs(&next_hash.elements);
  
    let data = builder.build::<C>();
    (data, targets)
  }