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
    x: u32,
    y: u32,
}

impl MyInput{
    pub fn from_json(infile: &str) -> Self {
        let witness: serde_json::Value = serde_json::from_reader(File::open(infile).unwrap()).unwrap();
        let xin: u32 = serde_json::from_value(witness["x"].clone()).unwrap();
        let yin: u32 = serde_json::from_value(witness["y"].clone()).unwrap();

        MyInput {         
            x: xin,
            y: yin,
        } 
    }
}

pub fn cubic_circuit<F: Field + From<u64> + Hash>(infile: &str) -> (Circuit<F>, Option<AssignmentGenerator<F, ()>>) {

    use chiquito::{
        ast::ExposeOffset::*, // for exposing witnesses
        frontend::dsl::cb::*, // functions for constraint building
    };
    let json_data = MyInput::from_json(infile);
    let cubic_circ = circuit::<F, (), _>("cubic", |ctx|{

        let x = ctx.forward("x");
        let y = ctx.forward("y");

        let cubic_first_step = ctx.step_type_def("cubic step", |ctx| {
            let c = ctx.internal("c");

            ctx.setup(move |ctx| {
                ctx.constr(eq(((x*x*x) + x + 5), c));
                ctx.transition(eq(c, x.next()));
                ctx.transition(eq(y, y.next()));
            });

            ctx.wg(move |ctx, (x_value, y_value): (u32, u32)| {
                ctx.assign(x, x_value.field());
                ctx.assign(y, y_value.field());
                ctx.assign(c, ((x_value * x_value * x_value) + x_value + 5).field());
            })
        });
        let cubic_final_step = ctx.step_type_def("cubic step", |ctx| {

            ctx.setup(move |ctx| {
                ctx.constr(eq(x, y));
                ctx.transition(eq(x, x.next()));
                ctx.transition(eq(y, y.next()));
            });

            ctx.wg(move |ctx, (x_value, y_value): (u32, u32)| {
                ctx.assign(x, x_value.field());
                ctx.assign(y, y_value.field());
            })
        });

        ctx.pragma_num_steps(2);
        ctx.pragma_first_step(&cubic_first_step);
        ctx.pragma_last_step(&cubic_final_step);
        
        ctx.expose(y, Last);

        ctx.trace(move |ctx: _, _| {
            let mut x = json_data.x;
            let mut y = json_data.y;
            ctx.add(&cubic_first_step, (x, y));
            x = ((x*x*x) + x + 5);
            ctx.add(&cubic_final_step, (x, y));
        })
    });

    compile(
        config(SingleRowCellManager {}, SimpleStepSelectorBuilder {}),
        &cubic_circ,
    )
}