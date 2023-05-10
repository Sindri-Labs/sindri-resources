This directory contains a circuit written in Axiom format:  the circuit definition file 'mult_circ.rs' relies only on the 'halo2_base' library.  If you try
```
cargo run --bin axiom_prover
```
everything should work.  This prover uses methods imported from halo2_base (the axiom lib).  However if you try to run a similar prover with methods from halo2_proofs (the pse lib) like this
```
cargo run --bin pse_prover
```
you'll see three errors, all coming from the same place.  There is apparantly a difference in field dependencies when implementing the circuit trait, and I don't know how to solve it.