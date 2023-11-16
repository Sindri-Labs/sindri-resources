use halo2_base::gates::{GateChip, GateInstructions};
use halo2_base::utils::ScalarField;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};

use halo2_base::{
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

pub struct CircuitInput<F: ScalarField> {
    pub x: F, 
}

//return default inputs
impl<F: ScalarField> Default for CircuitInput<F> {
    fn default() -> Self {
        Self {
            x: F::from(1)
        }        
    }
}

impl<F: ScalarField> CircuitInput<F> {
    //return inputs from json
    pub fn from_json(infile: &str) -> Self {
        let witness: serde_json::Value = serde_json::from_reader(File::open(infile).unwrap()).unwrap();
        let xin: u64 = serde_json::from_value(witness["x"].clone()).unwrap();
        Self {
            x: F::from(xin)
        }        
    }

    //From the witness input, this will return a circuit constructed from the various
    //modes of GateThreadBuilder - it returns the ScaffoldCircuitBuilder which implements
    //the final requirements of a circuit and has some handy instance methods
    pub fn create_circuit(
        self,
        mut builder: GateThreadBuilder<F>,
        break_points: Option<MultiPhaseThreadBreakPoints>,
    ) -> QuadraticCircuitBuilder<F> {

        //initialize instance
        let mut assigned_instances = vec![];
        //circuit definition via Axiom's halo2-lib
        let ctx = builder.main(0);
        let c = F::from(72);
        let gate = GateChip::<F>::default();
        let x = ctx.load_witness(self.x); 
        let _val_assigned = gate.mul_add(ctx, x, x, Constant(c));

        assigned_instances.push(x);

        let k: usize = 9; 
        let minimum_rows: usize = 9;
        builder.config(k, Some(minimum_rows));

        let circuit = match builder.witness_gen_only() {
            true => RangeCircuitBuilder::prover( builder, break_points.unwrap()),
            false => RangeCircuitBuilder::keygen(builder),
        };
        
        QuadraticCircuitBuilder(RangeWithInstanceCircuitBuilder::new(circuit, assigned_instances))
    }
}

pub struct QuadraticCircuitBuilder<F: ScalarField>(pub RangeWithInstanceCircuitBuilder<F>);


impl<F: ScalarField> Circuit<F> for QuadraticCircuitBuilder<F> {
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
impl<F: ScalarField> QuadraticCircuitBuilder<F> {

    pub fn instance(&self) -> Vec<F> {
        self.0.instance()
    }

    pub fn break_points(&self) -> MultiPhaseThreadBreakPoints {
        self.0.circuit.0.break_points.borrow().clone()
    }
}