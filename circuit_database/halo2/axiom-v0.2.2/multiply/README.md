# Multiply

This directory contains a circuit written in Axiom's Halo2-lib `v0.2.2` format:  the circuit definition file 'mult_circ.rs' relies only on the 'halo2_base' library.  While there are no public outputs to from this circuit, internally the circuit is multiplying two numbers 120 times.  This circuit can easily be altered for benchmarking purposes.


Source: https://github.com/axiom-crypto/halo2-lib/blob/v0.2.2/halo2-base/benches/mul.rs