use halo2_base::gates::{
    flex_gate::{FlexGateConfig, GateStrategy},
    GateInstructions,
};
use halo2_base::halo2_proofs::{
    circuit::*,
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::*,
    poly::kzg::{
        commitment::{KZGCommitmentScheme, ParamsKZG},
        multiopen::ProverGWC,
    },
    transcript::{Blake2bWrite, Challenge255, TranscriptWriterBuffer},
};
use halo2_base::{
    Context, ContextParams,
    QuantumCell::{Existing, Witness},
    SKIP_FIRST_PASS,
};
use std::fs::File;

#[derive(Clone, Default)]
pub struct MyCircuit<F> {
    a: Value<F>,
    b: Value<F>,
    c: Value<F>,
}


impl MyCircuit<Fr> {
    pub fn from_json(infile: &str) -> Self {
        let witness: serde_json::Value = serde_json::from_reader(File::open(infile).unwrap()).unwrap();
        let ain: u64 = serde_json::from_value(witness["a"].clone()).unwrap();
        let bin: u64 = serde_json::from_value(witness["b"].clone()).unwrap();
        let cin: u64 = serde_json::from_value(witness["c"].clone()).unwrap();

        MyCircuit {         
            a: Value::known(Fr::from(ain)),
            b: Value::known(Fr::from(bin)),
            c: Value::known(Fr::from(cin)),
        } 
    }

    pub fn instance(&self) -> Vec<Fr> {
        Vec::<Fr>::new()
    }
}

const NUM_ADVICE: usize = 1;
const K: u32 = 9;

impl Circuit<Fr> for MyCircuit<Fr> {
    type Config = FlexGateConfig<Fr>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        FlexGateConfig::configure(meta, GateStrategy::PlonkPlus, &[NUM_ADVICE], 1, 0, K as usize)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        let mut first_pass = SKIP_FIRST_PASS;

        layouter.assign_region(
            || "gate",
            |region| {
                if first_pass {
                    first_pass = false;
                    return Ok(());
                }

                let mut aux = Context::new(
                    region,
                    ContextParams {
                        max_rows: config.max_rows,
                        num_context_ids: 1,
                        fixed_columns: config.constants.clone(),
                    },
                );
                let ctx = &mut aux;

                let (_a_cell, b_cell, c_cell) = {
                    let cells = config.assign_region_smart(
                        ctx,
                        vec![Witness(self.a), Witness(self.b), Witness(self.c)],
                        vec![],
                        vec![],
                        vec![],
                    );
                    (cells[0].clone(), cells[1].clone(), cells[2].clone())
                };

                for _ in 0..120 {
                    config.mul(ctx, Existing(&c_cell), Existing(&b_cell));
                }

                Ok(())
            },
        )
    }
}
