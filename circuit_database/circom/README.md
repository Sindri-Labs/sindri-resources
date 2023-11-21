# Circom Circuits

Every directory is a circuit that contains
- `README.md`
- `Sindri.json`
- Circuit code


## Available Circuits

| Name | Size (# Constraints) | Original Source | Functionality | 
| - | - | - | - | 
| multiplier2 | 1 | [Circom2 Docs](https://docs.circom.io/getting-started/writing-circuits/)| Computes the product of two private inputs| 
| sha256      | 152313 | [Celer Network's Benchmarking](https://github.com/celer-network/zk-benchmark/tree/main) | Verifies that the SHA256 hash of a private preimage equals the claimed public output |
