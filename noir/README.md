# Valid Noir sample data

Every directory is a valid circuit that contains
- `README.md`
- `Sindri.json` with `"circuit_type": "noir"`

The suffix at the end of any circuit directory (e.g. `v0_10_5`) communicates which Noir compiler version the circuit specifies within the `Nargo.toml` configuration file.  This version is added to the suffix rather than the prefix (which is the style for Gnark and Halo2) because it should not outwardly influence our Nargo prover.  However, as you will note from the examples in `sampledata/qa_invalid/noir`, compiler versions below v0.7.0 and above v0.10.5 seem more likely to encounter runtime bugs with our Noir proving backend.