# Halo2 Circuits

### Axiom v0.2.2

| Name | Size (DEGREE) | Original Source | Functionality | 
| ---- | ---- | --------------- | ------------- | 
|mult-example| 11 | [halo2-lib's benchmarking](https://github.com/axiom-crypto/halo2-lib/blob/v0.2.2/halo2-base/benches/mul.rs) | Performs $b*c$ 120 times with no public output (120 is a tunable parameter to time proofs for circuits of various sizes)|

### Axiom v0.3.0

| Name | Size (DEGREE) | Original Source | Functionality | 
| ---- | ---- | --------------- | ------------- |
| quadratic_circuit | 10 | Adapted from [halo2-scaffold](https://github.com/axiom-crypto/halo2-scaffold) | Computes $x^2+72$ with $x$ a public input | 
| axiom_header_goerli | 14 | [axiom-eth repo](https://github.com/axiom-crypto/axiom-eth/tree/axiom-dev-0406) | Produces a ZKP verifying the validity of a sequence of Goerli block headers