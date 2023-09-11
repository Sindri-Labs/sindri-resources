# Poseidon Circuit

This circuit has two variables, `data` representing some private preimage and `hash` which is made public.  Notice that the input file only contains one field `PreImage`.  That's because the user defined `FromJson` function (see line 50 of `poseidon_ext.go`) can transform the preimage into a hash before constructing the circuit assignment.

Source: The poseidon circuit comes from Vocdoni's gnark crypto primitives: https://github.com/vocdoni/gnark-crypto-primitives/tree/main.  The utility to read input from json is based on https://github.com/zkCollective/zk-Harness/blob/main/gnark/util/reader.go