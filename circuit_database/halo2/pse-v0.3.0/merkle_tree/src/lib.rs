//Use chips defined by Cardinal Cryptography's "shielder" package
use shielder::circuits::{
    merkle::{
        generate_example_path,
        membership::{MembershipChip, MembershipConfig},
        merkle::{MerkleProofChip, MerkleProofConfig},
        poseidon_spec::PoseidonSpec
    },
    FieldExt,
};
//Poseidon definition from PSE's halo2_poseidon package
use halo2_poseidon::poseidon::{
    primitives::{ConstantLength, Hash},
    Pow5Chip,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    halo2curves::bn256::Fr,
    plonk::{Circuit,ConstraintSystem, Error}
};
use std::fs::File;

const ARITY: usize = 2;
const WIDTH: usize = 3;
const TREE_HEIGHT: usize = 40;
pub type AssignedCell<F> = halo2_proofs::circuit::AssignedCell<F, F>;

pub struct MTPCircuit<F: FieldExt> {
    path: [[Value<F>; ARITY]; TREE_HEIGHT],
    leaf: Value<F>,
}

impl<F: FieldExt> Circuit<F> for MTPCircuit<F>
{
    type Config = MerkleProofConfig<F, WIDTH, ARITY, TREE_HEIGHT>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            path: [[Value::unknown(); ARITY]; TREE_HEIGHT],
            leaf: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advices_path = [(); TREE_HEIGHT].map(|_| meta.advice_column());
        let leaf = meta.advice_column();
        let root = meta.instance_column();

        let poseidon_state = [(); WIDTH].map(|_| meta.advice_column());
        let poseidon_partial_sbox = meta.advice_column();

        let membership_needle = meta.advice_column();
        let membership_haystack = [(); ARITY].map(|_| meta.advice_column());

        for advice in advices_path
            .iter()
            .chain(poseidon_state.iter())
            .chain([poseidon_partial_sbox].iter())
            .chain([membership_needle].iter())
            .chain(membership_haystack.iter())
        {
            meta.enable_equality(*advice);
        }
        meta.enable_equality(leaf);
        meta.enable_equality(root);

        let col_const = meta.fixed_column();
        meta.enable_constant(col_const);

        let poseidon_rc_a = [(); WIDTH].map(|_| meta.fixed_column());
        let poseidon_rc_b = [(); WIDTH].map(|_| meta.fixed_column());

        let poseidon_config = Pow5Chip::configure::<PoseidonSpec<WIDTH, ARITY>>(
            meta,
            poseidon_state,
            poseidon_partial_sbox,
            poseidon_rc_a,
            poseidon_rc_b,
        );

        let membership_config = MembershipChip::configure(
            meta,
            membership_needle,
            membership_haystack,
        );

        MerkleProofChip::<F, WIDTH, ARITY, TREE_HEIGHT>::configure(
            meta,
            advices_path,
            leaf,
            root,
            poseidon_config,
            membership_config,
        )
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // Initialize the Path chip
        let path_chip = MerkleProofChip::new(config.clone());

        // Witness values
        let (path, leaf) = layouter.assign_region(
            || "witness",
            |mut region| {
                let path: [[AssignedCell<F>; ARITY]; TREE_HEIGHT] = (0
                    ..TREE_HEIGHT)
                    .map(|level| {
                        (0..ARITY)
                            .map(|sibling| {
                                region
                                    .assign_advice(
                                        || "witness root",
                                        config.advices_path[level],
                                        sibling,
                                        || self.path[level][sibling],
                                    )
                                    .unwrap()
                            })
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap()
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();

                let leaf = region.assign_advice(
                    || "witness leaf",
                    config.leaf,
                    0,
                    || self.leaf,
                )?;

                Ok((path, leaf))
            },
        )?;

        path_chip.synthesize(&mut layouter, leaf, path)
    }
}

impl MTPCircuit<Fr> {

    // Specific functions required for compatibility with Sindri
    pub fn from_json(
        json_loc: &str,
    ) -> (Self, Vec<Vec<Fr>>) {

        let (root, leaf, path) = generate_example_path::<Fr, WIDTH, ARITY, TREE_HEIGHT>();
        println!("{:?}", &root); 
        let leaf = Value::known(leaf);
        let path = {
            let mut value_path = [[Value::<Fr>::default(); ARITY]; TREE_HEIGHT];
            for level in 0..TREE_HEIGHT {
                for node in 0..ARITY {
                    value_path[level][node] = Value::known(path[level][node]);
                }
            }
            value_path
        };
        println!("{:?}", &path);
    
        // Create circuit from inputs
        (MTPCircuit { path, leaf }, vec![vec![root]])
    }

    pub fn keygen_circuit() -> Self {
        // Nearly identical to `empty_circuit` without generics
        MTPCircuit {
            path: [[Value::unknown(); ARITY]; TREE_HEIGHT],
            leaf: Value::unknown(),
        }
    }

}
