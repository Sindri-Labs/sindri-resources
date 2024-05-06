## merkle_tree_proof_verifier Circuit

This circuit project contains all the necessary ingredients to upload to Sindri's API and start requesting proofs.

The circuit language is written in gnark and will produce a proof that a user has uploaded a valid merkle tree path that matches a publicly available root hash.

The Gnark standard library does provide a MTP gadget, but that abstracts away the circuit code.
So we started from [this implementation](https://github.com/hashcloak/merkle_trees_gnark/blob/master/merkle_tree.go) from [Hashclock](https://hashcloak.com/), a blockchain R&D lab.
While it makes use of the mimc hash gadget, abstracting some of the low-level details, we can still see the operations taking place following a MT path.