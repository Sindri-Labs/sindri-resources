use halo2_base::utils::{ScalarField, BigPrimeField};
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
    gates::builder::{
        GateThreadBuilder, MultiPhaseThreadBreakPoints, RangeCircuitBuilder,
        RangeWithInstanceCircuitBuilder, RangeWithInstanceConfig,
    },
    halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::{Circuit, ConstraintSystem, Error},
    }
};

use std::fs::File;
use std::env::var;
use crate::gadgets::{FixedPointChip,FixedPointInstructions};

pub struct CircuitInput<F: ScalarField> {
    pub x: f64,
    pub y: f64, 
    _marker: std::marker::PhantomData<F>
}

//return default inputs
impl<F: ScalarField> Default for CircuitInput<F> {
    fn default() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            _marker: std::marker::PhantomData
        }        
    }
}

impl<F: ScalarField + std::convert::From<[u64; 4]>> CircuitInput<F> where F: BigPrimeField, [u64; 4]: std::convert::From<F> {
    //return inputs from json
    pub fn from_json(infile: &str) -> Self {
        let witness: serde_json::Value = serde_json::from_reader(File::open(infile).unwrap()).unwrap();
        let xin: f64 = serde_json::from_value(witness["x"].clone()).unwrap();
        let yin: f64 = serde_json::from_value(witness["y"].clone()).unwrap();
        Self {
            x: xin,
            y: yin,
            _marker: std::marker::PhantomData
        }        
    }

    //From the witness input, this will return a circuit constructed from the various
    //modes of GateThreadBuilder - it returns the ScaffoldCircuitBuilder which implements
    //the final requirements of a circuit and has some handy instance methods
    pub fn create_circuit(
        self,
        mut builder: GateThreadBuilder<F>,
        break_points: Option<MultiPhaseThreadBreakPoints>,
    )  -> RadiusCircuitBuilder<F> where F: BigPrimeField, [u64; 4]: std::convert::From<F> {

        //initialize instance
        let mut assigned_instances = vec![];

        //circuit definition via Axiom's halo2-lib
        let ctx = builder.main(0);

        // Need to add LOOKUP_BITS to config.json, as it is used to configure the lookup advice
        let lookup_bits = var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
        const PRECISION_BITS: u32 = 32;
        // using 32-bit fixed-point exp arithmetic
        let fixed_point_chip = FixedPointChip::<F, PRECISION_BITS>::default(lookup_bits);

        let x = fixed_point_chip.quantization(self.x);
        let y = fixed_point_chip.quantization(self.y);

        let x = ctx.load_witness(x);
        let y = ctx.load_witness(y);
  
        let x_magnitude_sq = fixed_point_chip.qmul(ctx, x, x);
        let y_magnitude_sq = fixed_point_chip.qmul(ctx,y,y);

        let radius_squared = fixed_point_chip.qadd(ctx,x_magnitude_sq,y_magnitude_sq);
        let radius = fixed_point_chip.qsqrt(ctx, radius_squared);

        assigned_instances.push(radius);

        let k: usize = 13; 
        let minimum_rows: usize = 9;
        builder.config(k, Some(minimum_rows));

        let circuit = match builder.witness_gen_only() {
            true => RangeCircuitBuilder::prover(builder, break_points.unwrap()),
            false => RangeCircuitBuilder::keygen(builder),
        };
        
        RadiusCircuitBuilder(RangeWithInstanceCircuitBuilder::new(circuit, assigned_instances))
    }

}

pub struct RadiusCircuitBuilder<F: ScalarField>(pub RangeWithInstanceCircuitBuilder<F>);


impl<F: ScalarField> Circuit<F> for RadiusCircuitBuilder<F> {
    type Config = RangeWithInstanceConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        unimplemented!()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        RangeWithInstanceCircuitBuilder::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        self.0.synthesize(config, layouter)
    }
}

// returning features of the circuit to the prover
impl<F: ScalarField> RadiusCircuitBuilder<F> {

    pub fn instance(&self) -> Vec<F> {
        self.0.instance()
    }

    pub fn break_points(&self) -> MultiPhaseThreadBreakPoints {
        self.0.circuit.0.break_points.borrow().clone()
    }
}