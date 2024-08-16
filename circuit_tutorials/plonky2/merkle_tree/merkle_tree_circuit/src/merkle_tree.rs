// Simple Merkle Tree impl

use itertools::Itertools;
use num::Integer;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::HashOut;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::Hasher;

use plonky2::util::log2_strict;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub count_levels: usize,
    pub tree: Vec<Vec<HashOut<GoldilocksField>>>, // contains vectors of hashes for the levels in the tree (count_levels-1 vectors)
    pub root: HashOut<GoldilocksField>
}

impl MerkleTree {

  // From list of hashes with length len, take each pair and hash them, resulting in a new vector of hashes of length len/2
  fn next_level_hashes(current_level: Vec<HashOut<GoldilocksField>>) -> Vec<HashOut<GoldilocksField>> {
    let temp: Vec<&[HashOut<GoldilocksField>]> = current_level.chunks(2).into_iter().collect_vec();
    let next_level: Vec<HashOut<GoldilocksField>> = temp.into_iter().map(|x| PoseidonHash::two_to_one(x[0], x[1])).collect();
    next_level
  }

  // Create a Merkle Tree given 2^n leaves. 
  pub fn build(leaves: Vec<GoldilocksField>) -> Self {
    // This panics if length is not a power of 2
    let count_levels = log2_strict(leaves.len());    
    // To get the first level, hash all leaves
    let level0: Vec<HashOut<GoldilocksField>> = leaves.into_iter().map(|leaf| { PoseidonHash::hash_or_noop(&[leaf])}).collect();

    let mut levels = Vec::new();
    levels.push(level0);
    // For next levels, hash every hashes. Ends at 2 hashes.
    for i in 0..(count_levels-1) {
      let next_level = Self::next_level_hashes(levels[i].clone());
      levels.push(next_level);
    }

    // Final hash for root.
    let last_hashes: Vec<HashOut<GoldilocksField>> = levels.clone().last().unwrap().to_vec();
    let root = PoseidonHash::two_to_one(last_hashes[0], last_hashes[1]);
    MerkleTree { 
      count_levels: count_levels, 
      tree: levels.clone(), 
      root: root
    }
  }

  // Returns count_levels elements that together with the leaf show that a leaf is part of this Merkle Tree, given the root
  // starts at the element at the lowest level and goes up
  pub fn get_merkle_proof(self, leaf_index: usize) -> Vec<HashOut<GoldilocksField>> {
    assert!(leaf_index < self.tree[0].len());

    let mut proof_hashes = Vec::new();
    let mut updated_index = leaf_index;

    // Grab the correct hash per level
    for i in 0..(self.count_levels) {
      let level_i: &Vec<HashOut<GoldilocksField>> = &self.tree[i];  
      let selected_hash = if updated_index.is_odd() {
        level_i[updated_index-1]
      } else {
        level_i[updated_index+1]
      };
      proof_hashes.push(selected_hash);
      updated_index = updated_index/2;
    }

    proof_hashes
  }

  pub fn get_in_between_hashes(self, leaf_index: usize) -> Vec<HashOut<GoldilocksField>>{
    assert!(leaf_index < self.tree[0].len());
    let mut index = leaf_index / 2;
    let mut hashes = Vec::new();
    for i in 1..self.count_levels {
      hashes.push(self.tree[i][index]);
      index = index / 2;
    }
    hashes.push(self.root);
    hashes
  }
}


// Returns true if the given proof indeed leads to the same root when hashing the leaf with the given hashes consequently
pub fn verify_merkle_proof(leaf: GoldilocksField, leaf_index: usize, root: HashOut<GoldilocksField>, hashes: Vec<HashOut<GoldilocksField>>) -> bool {
  // Step 1: hash leaf
  let leaf_hashed: HashOut<GoldilocksField> = PoseidonHash::hash_or_noop(&[leaf]);

  // Repeat: take 1 hash from list and current hash, hash together
  let mut next_hash: HashOut<GoldilocksField> = leaf_hashed;
  let mut updated_index = leaf_index;
  for i in 0..hashes.len() {
    if updated_index.is_even() {
      next_hash = PoseidonHash::two_to_one(next_hash, hashes[i]);
    } else {
      next_hash = PoseidonHash::two_to_one(hashes[i], next_hash);
    }
    updated_index = updated_index/2;
  }
  
  // Finally: compare final hash with root
  next_hash == root
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use plonky2::{plonk::config::{GenericConfig, PoseidonGoldilocksConfig}, hash::hash_types::HashOut, field::{goldilocks_field::GoldilocksField, types::Field}};
  use crate::simple_merkle_tree::simple_merkle_tree::{MerkleTree, verify_merkle_proof};

  #[test]
  fn test_build_merkle_tree_4_leaves() -> Result<()> {

    let leaves = [
      GoldilocksField::from_canonical_u64(2890852870), 
      GoldilocksField::from_canonical_u64(156728478), 
      GoldilocksField::from_canonical_u64(2876514289), 
      GoldilocksField::from_canonical_u64(984286162)
      ].to_vec();
    let _tree: MerkleTree = MerkleTree::build(leaves);
    
    // println!( "{:?}", tree.count_levels);
    // println!( "{:?}", tree.tree);
    // println!( "{:?}", tree.root);
    /*
    Output looks something like this:

    2
    level0
    [[HashOut { elements: [2890852870, 0, 0, 0] }, HashOut { elements: [156728478, 0, 0, 0] }, HashOut { elements: [2876514289, 0, 0, 0] }, HashOut { elements: [984286162, 0, 0, 0] }], 
    level1
    [HashOut { elements: [6678006133445961348, 15827935749738443865, 6295652393730592048, 1546515167911236130] }, HashOut { elements: [6698018865469624861, 12486244005715193285, 11330639022572315007, 6059804404595156248] }]]
    root
    HashOut { elements: [13451271846715771774, 4069913004933160254, 14528216580130305557, 9716424959297545638] }
    */
    Ok(())
  }

  #[test]
  fn test_build_merkle_tree_16_leaves() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let leaves = [
      F::from_noncanonical_u128(14786323743454721611), 
      F::from_noncanonical_u128(976503040092093812), 
      F::from_noncanonical_u128(4644130751253292674), 
      F::from_noncanonical_u128(6522877527545910706),
      F::from_noncanonical_u128(11021172818651636092), 
      F::from_noncanonical_u128(12048403458499719587), 
      F::from_noncanonical_u128(11457874926809001558), 
      F::from_noncanonical_u128(14982007443548219923),
      F::from_noncanonical_u128(4546369223935415035), 
      F::from_noncanonical_u128(7205140577604465038), 
      F::from_noncanonical_u128(4644130751253292674), 
      F::from_noncanonical_u128(4208177174652750506),
      F::from_noncanonical_u128(16147116534354400672), 
      F::from_noncanonical_u128(18147003476480002882), 
      F::from_noncanonical_u128(14133393155459789216), 
      F::from_noncanonical_u128(9890944065319669426),
      ].to_vec();
    let _tree: MerkleTree  = MerkleTree::build(leaves);
    
    // println!( "{:?}", tree.count_levels);
    // println!( "{:?}", tree.tree);
    // println!( "{:?}", tree.root);
    /*
    Output looks something like this:

    level
    4

    16 hashes
    [[HashOut { elements: [14786323743454721611, 0, 0, 0] }, HashOut { elements: [976503040092093812, 0, 0, 0] }, HashOut { elements: [4644130751253292674, 0, 0, 0] }, HashOut { elements: [6522877527545910706, 0, 0, 0] }, HashOut { elements: [11021172818651636092, 0, 0, 0] }, HashOut { elements: [12048403458499719587, 0, 0, 0] }, HashOut { elements: [11457874926809001558, 0, 0, 0] }, HashOut { elements: [14982007443548219923, 0, 0, 0] }, HashOut { elements: [4546369223935415035, 0, 0, 0] }, HashOut { elements: [7205140577604465038, 0, 0, 0] }, HashOut { elements: [4644130751253292674, 0, 0, 0] }, HashOut { elements: [4208177174652750506, 0, 0, 0] }, HashOut { elements: [16147116534354400672, 0, 0, 0] }, HashOut { elements: [18147003476480002882, 0, 0, 0] }, HashOut { elements: [14133393155459789216, 0, 0, 0] }, HashOut { elements: [9890944065319669426, 0, 0, 0] }], 
    8 hashes
    [HashOut { elements: [16072672881132969138, 16679487992876356669, 4319836168073005766, 14599992432910949662] }, HashOut { elements: [13939588529466633382, 10763620781372339433, 2004324520800166618, 12719277447629989832] }, HashOut { elements: [14656282042201240311, 6170970616712589521, 11157357638961986056, 5438125353060943827] }, HashOut { elements: [2852357662721872796, 9295427221128388695, 16660079770794313894, 16593848747552204277] }, HashOut { elements: [1050226065911802874, 3351649260676358938, 8999655484267522522, 8103734492761011635] }, HashOut { elements: [13846094456198376205, 9253307167504386126, 4161612500674764592, 7803567864857746324] }, HashOut { elements: [12908760732062415960, 7139572648574712447, 5556204547814336347, 3348534187924876571] }, HashOut { elements: [3098588497934611450, 6742117982852617524, 916011858354471608, 2853499230617662209] }], 
    4 hashes
    [HashOut { elements: [2804654470754882522, 10755905498140000489, 4068725548728740598, 3390508811108791323] }, HashOut { elements: [12809105246780417325, 17913287784403914705, 15645466341003679334, 9087376211576685650] }, HashOut { elements: [47027546173659393, 10025915649153530159, 13833576669443475941, 16760048762533095483] }, HashOut { elements: [9702041242754623164, 9442892912940285811, 2205638039663440432, 4535189628500499303] }], 
    2 hashes
    [HashOut { elements: [13714743953980303059, 8312649863783971264, 16925076622785735252, 9754399915922106354] }, HashOut { elements: [14079844864384152521, 6499705357519308869, 16026207645313349904, 15079809878245341298] }]]

    root
    HashOut { elements: [2659148958598424285, 16496267010313658247, 12216516055477211974, 15749220035779350537] }
    */
    Ok(())
  }

  #[test]
  fn test_merkle_proof_small_tree() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let leaves = [
      F::from_canonical_u64(2890852870), 
      F::from_canonical_u64(156728478), 
      F::from_canonical_u64(2876514289), 
      F::from_canonical_u64(984286162)
      ].to_vec();
    let tree: MerkleTree  = MerkleTree::build(leaves);
    
    let res_leaf_0 = tree.clone().get_merkle_proof(0);
    assert!(res_leaf_0[0] == HashOut { elements: [F::from_canonical_u64(156728478), F::default(), F::default(), F::default()] });
    assert!(res_leaf_0[1] == HashOut { elements: [F::from_canonical_u64(6698018865469624861), F::from_canonical_u64(12486244005715193285), F::from_canonical_u64(11330639022572315007), F::from_canonical_u64(6059804404595156248)] });
    Ok(())
  }

  #[test]
  fn test_verify_small_merkle_proof() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let leaves = [
      F::from_canonical_u64(2890852870), 
      F::from_canonical_u64(156728478), 
      F::from_canonical_u64(2876514289), 
      F::from_canonical_u64(984286162)
      ].to_vec();
    let tree: MerkleTree = MerkleTree::build(leaves.clone());
    
    let res_leaf_0 = tree.clone().get_merkle_proof(0);
    let res_leaf_3 = tree.clone().get_merkle_proof(3);

    assert!(verify_merkle_proof(leaves[0], 0, tree.root, res_leaf_0));
    assert!(verify_merkle_proof(leaves[3], 3, tree.root, res_leaf_3));
    Ok(())
  }

  #[test]
  fn test_verify_merkle_proof_16() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let leaves = [
      F::from_noncanonical_u128(14786323743454721611), 
      F::from_noncanonical_u128(976503040092093812), 
      F::from_noncanonical_u128(4644130751253292674), 
      F::from_noncanonical_u128(6522877527545910706),
      F::from_noncanonical_u128(11021172818651636092), 
      F::from_noncanonical_u128(12048403458499719587), 
      F::from_noncanonical_u128(11457874926809001558), 
      F::from_noncanonical_u128(14982007443548219923),
      F::from_noncanonical_u128(4546369223935415035), 
      F::from_noncanonical_u128(7205140577604465038), 
      F::from_noncanonical_u128(4644130751253292674), 
      F::from_noncanonical_u128(4208177174652750506),
      F::from_noncanonical_u128(16147116534354400672), 
      F::from_noncanonical_u128(18147003476480002882), 
      F::from_noncanonical_u128(14133393155459789216), 
      F::from_noncanonical_u128(9890944065319669426),
      ].to_vec();
    let tree: MerkleTree = MerkleTree::build(leaves.clone());
    
    let res_leaf_0 = tree.clone().get_merkle_proof(0);
    let res_leaf_1 = tree.clone().get_merkle_proof(1);
    let res_leaf_2 = tree.clone().get_merkle_proof(2);
    let res_leaf_3 = tree.clone().get_merkle_proof(3);
    let res_leaf_4 = tree.clone().get_merkle_proof(4);
    let res_leaf_5 = tree.clone().get_merkle_proof(5);
    let res_leaf_6 = tree.clone().get_merkle_proof(6);
    let res_leaf_7 = tree.clone().get_merkle_proof(7);
    let res_leaf_8 = tree.clone().get_merkle_proof(8);
    let res_leaf_9 = tree.clone().get_merkle_proof(9);
    let res_leaf_10 = tree.clone().get_merkle_proof(10);
    let res_leaf_11 = tree.clone().get_merkle_proof(11);
    let res_leaf_12 = tree.clone().get_merkle_proof(12);
    let res_leaf_13 = tree.clone().get_merkle_proof(13);
    let res_leaf_14 = tree.clone().get_merkle_proof(14);
    let res_leaf_15 = tree.clone().get_merkle_proof(15);

    // Assert correct proofs
    assert!(verify_merkle_proof(leaves[0], 0, tree.root, res_leaf_0.clone()));
    assert!(verify_merkle_proof(leaves[1], 1, tree.root, res_leaf_1.clone()));
    assert!(verify_merkle_proof(leaves[2], 2, tree.root, res_leaf_2));
    assert!(verify_merkle_proof(leaves[3], 3, tree.root, res_leaf_3));
    assert!(verify_merkle_proof(leaves[4], 4, tree.root, res_leaf_4));
    assert!(verify_merkle_proof(leaves[5], 5, tree.root, res_leaf_5));
    assert!(verify_merkle_proof(leaves[6], 6, tree.root, res_leaf_6));
    assert!(verify_merkle_proof(leaves[7], 7, tree.root, res_leaf_7));
    assert!(verify_merkle_proof(leaves[8], 8, tree.root, res_leaf_8));
    assert!(verify_merkle_proof(leaves[9], 9, tree.root, res_leaf_9));
    assert!(verify_merkle_proof(leaves[10], 10, tree.root, res_leaf_10));
    assert!(verify_merkle_proof(leaves[11], 11, tree.root, res_leaf_11));
    assert!(verify_merkle_proof(leaves[12], 12, tree.root, res_leaf_12));
    assert!(verify_merkle_proof(leaves[13], 13, tree.root, res_leaf_13));
    assert!(verify_merkle_proof(leaves[14], 14, tree.root, res_leaf_14));
    assert!(verify_merkle_proof(leaves[15], 15, tree.root, res_leaf_15));

    // Assert incorrect proof fails
    // wrong leaf
    assert!(!verify_merkle_proof(leaves[1], 0, tree.root, res_leaf_0.clone()));
    // wrong index
    assert!(!verify_merkle_proof(leaves[0], 1, tree.root, res_leaf_0.clone()));
    // wrong proof
    assert!(!verify_merkle_proof(leaves[0], 0, tree.root, res_leaf_1.clone()));
    // wrong root
    assert!(!verify_merkle_proof(leaves[0], 0, tree.tree[0][0], res_leaf_0.clone()));

    Ok(())
  }

}