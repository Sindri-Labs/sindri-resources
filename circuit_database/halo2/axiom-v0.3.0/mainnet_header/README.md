# Axiom Mainnet Header

This circuit was adapted from the [axiom-eth repo](https://github.com/axiom-crypto/axiom-eth/tree/axiom-dev-0406).  It produces a ZKP verifying the validity of a sequence of Mainnet block headers of length up to $2**7$.

If you wish to use `goerli` instead of `mainnet`, change `sindri.json` to
```json
{
  "$schema": "https://forge.sindri.app/api/v1/sindri-manifest-schema.json",
  "name": "axiom-goerli-header",
  "circuitType": "halo2",
  "className": "axiom_eth::block_header::EthBlockHeaderChainCircuit",
  "degree": 14,
  "halo2Version": "axiom-v0.3.0",
  "packageName": "axiom-eth",
  "threadBuilder": "RlcThreadBuilder"
}
```