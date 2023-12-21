# Axiom Mainnet Header

This circuit was adapted from the [axiom-eth repository](https://github.com/axiom-crypto/axiom-eth/tree/axiom-dev-0406).  It produces a ZKP verifying the validity of a sequence of Mainnet block headers of length up to $2**7$.

If you wish to use `goerli` instead of `mainnet`, change `sindri.json` to
```json
{
  "$schema": "https://sindri.app/api/v1/sindri-manifest-schema.json",
  "name": "axiom-goerli-header",
  "circuitType": "halo2",
  "className": "axiom_eth::block_header::EthBlockHeaderChainCircuit",
  "degree": 14,
  "halo2Version": "axiom-v0.3.0",
  "packageName": "axiom-eth",
  "threadBuilder": "RlcThreadBuilder"
}
```
and `config.json` to
```json
{
    "ETH_CONFIG_PARAMS": "{\"degree\":14,\"num_rlc_columns\":3,\"num_range_advice\":[46,16,0],\"num_lookup_advice\":[1,1,0],\"num_fixed\":1,\"unusable_rows\":61,\"keccak_rows_per_round\":12,\"lookup_bits\":8}",
    "NETWORK": "GOERLI",
    "MAX_DEPTH": "3",
    "LOOKUP_BITS": "8",
    "KECCAK_ROWS": "12"
}
```