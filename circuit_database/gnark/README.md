# Gnark Circuits

### Gnark v0.8.1 (Gnark-Crypto v0.9.1)

| Name | Size (DEGREE) | Original Source | Functionality | 
| - | - | - | - | 
| cubic bls12-377 | 3 | [Gnark Github Examples](https://github.com/Consensys/gnark/blob/master/examples/cubic/cubic.go) | Checks $x^3 + x + 5 == y$ for public $y$ and private $x$|
| exponentiate | ? | [Gnark Github Examples](https://github.com/Consensys/gnark/blob/master/examples/exponentiate/exponentiate.go) | Checks that $y == x**e$ where $e$ is private |

### Gnark v0.9.0 (Gnark-Crypto v0.11.2)
| Name | Size (DEGREE) | Original Source | Functionality | 
| - | - | - | - | 
| cubic bn254 | 3 | [Gnark Github Examples](https://github.com/Consensys/gnark/blob/master/examples/cubic/cubic.go) | Checks $x^3 + x + 5 == y$ for public $y$ and private $x$ |
| poseidon | 214 | [Vocdoni's Gnark Primitives](https://github.com/vocdoni/gnark-crypto-primitives/tree/main) | Computes the Poseidon hash of a preimage  |
