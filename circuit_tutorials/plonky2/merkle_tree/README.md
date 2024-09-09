# Merkle Tree Circuit

This Rust script uploads and proves a plonky2 circuit that verifies a merkle inclusion proof for a 1024 leaf merkle tree

To run the code, change the `sample.env` to `.env` and enter your Sindri API Key. From the `merkle_tree`, run:
```bash
cargo run --release
```