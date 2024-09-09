# Merkle Tree Circuit

This Rust script uploads and proves a Plonky2 circuit that verifies a Merkle inclusion proof for a 1024 leaf Merkle tree

To run the code, change the `sample.env` to `.env` and enter your Sindri API Key. From the root of the `merkle_tree` directory, run:
```bash
cargo run --release
```