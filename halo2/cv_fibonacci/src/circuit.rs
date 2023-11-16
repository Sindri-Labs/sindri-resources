use std::hash::Hash;

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


pub fn fibo_circuit<F: Field + From<u64> + Hash>(infile: &str) -> (Circuit<F>, Option<AssignmentGenerator<F, ()>>) {
    // PLONKish table for the Fibonacci circuit:
    // | a | b | c |
    // | 1 | 1 | 2 |
    // | 1 | 2 | 3 |
    // | 2 | 3 | 5 |
    // | 3 | 5 | 8 |
    // ...

    use chiquito::{
        ast::ExposeOffset::*, // for exposing witnesses
        frontend::dsl::cb::*, // functions for constraint building
    };
    _ = infile;

    let fibo = circuit::<F, (), _>("fibonacci", |ctx| {
        // the following objects (forward signals, steptypes) are defined on the circuit-level

        // forward signals can have constraints across different steps
        let a = ctx.forward("a");
        let b = ctx.forward("b");

        // define step type
        let fibo_step = ctx.step_type_def("fibo step", |ctx| {
            // the following objects (constraints, transition constraints, witness generation
            // function) are defined on the step type-level

            // internal signals can only have constraints within the same step
            let c = ctx.internal("c");

            // in setup we define the constraints of the step
            ctx.setup(move |ctx| {
                // regular constraints are for internal signals only
                // constrain that a + b == c by calling `eq` function from constraint builder
                ctx.constr(eq(a + b, c));

                // transition constraints accepts forward signals as well
                // constrain that b is equal to the next instance of a, by calling `next` on forward
                // signal
                ctx.transition(eq(b, a.next()));
                // constrain that c is equal to the next instance of c, by calling `next` on forward
                // signal
                ctx.transition(eq(c, b.next()));
            });

            // witness generation (wg) function is Turing complete and allows arbitrary user defined
            // logics for assigning witness values wg function is defined here but no
            // witness value is assigned yet
            ctx.wg(move |ctx, (a_value, b_value): (u32, u32)| {
                // println!("fib line wg: {} {} {}", a_value, b_value, a_value + b_value);
                // assign arbitrary input values from witness generation function to witnesses
                ctx.assign(a, a_value.field());
                ctx.assign(b, b_value.field());
                ctx.assign(c, (a_value + b_value).field());
            })
        });

        ctx.pragma_num_steps(11);

        // trace function is responsible for adding step instantiations defined in step_type_def
        // function above trace function is Turing complete and allows arbitrary user
        // defined logics for assigning witness values
        ctx.trace(move |ctx: _, _| {
            // add function adds a step instantiation to the main circuit and calls witness
            // generation function defined in step_type_def input values for witness
            // generation function are (1, 1) in this step instance
            ctx.add(&fibo_step, (1, 1));
            let mut a = 1;
            let mut b = 2;

            for _i in 1..11 {
                ctx.add(&fibo_step, (a, b));

                let prev_a = a;
                a = b;
                b += prev_a;
            }
        })
    });

    compile(
        config(SingleRowCellManager {}, SimpleStepSelectorBuilder {}),
        &fibo,
    )
}