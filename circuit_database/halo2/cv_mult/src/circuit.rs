use std::fs::File;
use std::hash::Hash;
use halo2curves::bn256::Fr;
use halo2_proofs::circuit::Value;


use chiquito::{
    field::Field,
    frontend::dsl::circuit, // main function for constructing an AST circuit
    plonkish::compiler::{
        cell_manager::SingleRowCellManager, // input for constructing the compiler
        compile,                            // input for constructing the compiler
        config,
        step_selector::SimpleStepSelectorBuilder,
    },
    plonkish::ir::{assignments::AssignmentGenerator, Circuit}, // compiled circuit type
    poly::ToField,
};
use chiquito::frontend::dsl::cb::*; // functions for constraint building

#[derive(Clone, Default)]
pub struct MyInput {
    a: u32,
    b: u32,
}

impl MyInput{
    pub fn from_json(infile: &str) -> Self {
        let witness: serde_json::Value = serde_json::from_reader(File::open(infile).unwrap()).unwrap();
        let ain: u32 = serde_json::from_value(witness["a"].clone()).unwrap();
        let bin: u32 = serde_json::from_value(witness["b"].clone()).unwrap();

        MyInput {         
            a: ain,
            b: bin,
        } 
    }
}

pub fn mult<F: Field + From<u64> + Hash>(infile: &str) -> (Circuit<F>, Option<AssignmentGenerator<F, ()>>) {

    let json_data = MyInput::from_json(infile);
    let mult_circ = circuit::<F, (), _>("multipy", |ctx|{

        let a = ctx.forward("a");
        let b = ctx.forward("b");

        let mult_step = ctx.step_type_def("mult step", |ctx| {
            let c = ctx.internal("c");

            ctx.setup(move |ctx| {
                ctx.constr(eq(a * b, c));

            });

            ctx.wg(move |ctx, (a_value, b_value): (u32, u32)| {
                // println!("Circuit answer: {}", a_value * b_value);
                ctx.assign(a, a_value.field());
                ctx.assign(b, b_value.field());
                ctx.assign(c, (a_value * b_value).field());
            })
        });

        ctx.pragma_num_steps(1);

        ctx.trace(move |ctx: _, _| {
            let mut a = json_data.a;
            let mut b = json_data.b;
            ctx.add(&mult_step, (a, b));
        })
    });

    compile(
        config(SingleRowCellManager {}, SimpleStepSelectorBuilder {}),
        &mult_circ,
    )
}