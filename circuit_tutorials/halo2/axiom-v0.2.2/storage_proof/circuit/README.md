# Storage proof

You can run a single storage proof with the command 
```
cargo run --bin single_storage_proof
```
Initial compile for me takes 1 minute, circuit setup is around 5 minutes, and the proving step is ~20 minutes.  Verification is almost instant.

The big changes required from our old prover.rs can be seen on lines 38, 49, and 62 of `src/bin/single_storage_proof.rs`.  The axiom storage circuit has public parameters, which are passed into the variable 'instances' which is then required by both prove and verify.  Next on my todo list is to figure out how we can serialize instances so that we can save it.

The input file `full_block_proof.json` is the result of an ethereum account query for a certain block, account, and slot.  You can change any hex character of the final key ("account"->"storageProof"->"value") and you should find that the prove step fails - because you're claiming an incorrect value for the storage which is then hashed up along the merkle branch and doesn't agree with the block merkle tree root hash.