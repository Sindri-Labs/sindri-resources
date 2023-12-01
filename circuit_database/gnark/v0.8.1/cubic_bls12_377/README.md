# Cubic circuit

This circuit is the standard for [Gnark beginners](https://github.com/Consensys/gnark/blob/master/examples/cubic/cubic.go). It checks $x^3 + x + 5 == y$ for public $y$ and private $x$.

Notice from the `sindri.json` file that when you upload this circuit, you are requesting Groth16 proofs over `bls12-377`.