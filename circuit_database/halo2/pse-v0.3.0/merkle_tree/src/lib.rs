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
    halo2curves::{
        bn256::Fr,
        serde::SerdeObject
    },
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

#[derive(serde::Serialize, serde::Deserialize)]
struct InputJSONSchema {
    root: Vec<u8>,
    leaf: Vec<u8>,
    path: Vec<Vec<Vec<u8>>>,
}
fn convert_u8(raw_input: &Vec<u8>) -> Fr {
    Fr::from_raw_bytes(raw_input).expect("field conversion failed")
}

impl MTPCircuit<Fr> {
    // Specific functions required for compatibility with Sindri
    pub fn from_json(json_loc: &str) -> (Self, Vec<Vec<Fr>>) {
        let mut file = File::open(json_loc).expect("Unable to open input JSON file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read input JSON file");
        let data: InputJSONSchema = serde_json::from_str(&contents).expect("JSON parse error");

        let root: Fr = convert_u8(&data.root);
        // Field elements below are wrapped in Value struct to mask their true assignment
        let leaf: Value<Fr> = Value::known(convert_u8(&data.leaf));

        let mut path: [[Value<Fr>; 2]; 40] = data
            .path
            .into_iter()
            .map(|instance_column| {
                instance_column
                    .iter()
                    .map(|strhex| Value::known(convert_u8(strhex)))
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("Path element does not have correct length.")
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("Path does not have correct length");

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
