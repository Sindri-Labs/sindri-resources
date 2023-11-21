# Valid Gnark sample data

Every directory is a valid circuit that contains
- `README.md`
- `Sindri.json` with `"circuit_type": "gnark"`

There are two versions of Gnark. They have shorthand versions in the items names.
- `v8`: `"gnark_version": "v0.8.1"`
- `v9`: `"gnark_version": "v0.9.0"`

If an item does not have a curve suffix, it is `bn254`.



## Available Circuits

### Gnark v0.8.1 (Gnark-Crypto v0.9.1)

| Name | Size (DEGREE) | Original Source | Functionality | 
| - | - | - | - | 
| cubic | 3 | [Gnark Github Examples](https://github.com/Consensys/gnark/blob/master/examples/cubic/cubic.go) | Checks $x^3 + x + 5 == y$ for public $y$ and private $x$|
| poseidon | 214 | [Vocdoni's Gnark Primitives](https://github.com/vocdoni/gnark-crypto-primitives/tree/main) | Computes the Poseidon hash of a preimage  |

### Gnark v0.9.0 (Gnark-Crypto v0.11.2)
| Name | Size (DEGREE) | Original Source | Functionality | 
| - | - | - | - | 
| cubic_v9 | 3 | [Gnark Github Examples](https://github.com/Consensys/gnark/blob/master/examples/cubic/cubic.go) | Checks $x^3 + x + 5 == y$ for public $y$ and private $x$ |
| poseidon_v9 | 214 | [Vocdoni's Gnark Primitives](https://github.com/vocdoni/gnark-crypto-primitives/tree/main) | Computes the Poseidon hash of a preimage  |
