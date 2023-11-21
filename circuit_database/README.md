# Sindri Circuit Database
This directory contains sample circuits for interacting with the Sindri Labs' API. The `README.md` within each framework directory will contain information specific to that circuit type.

Each sample circuit will include:
- A `README.md` 
- A `sindri.json` file (required)
- An example input file (`input.json`) for a proving the circuit
- Files specific to the circuit

Sample circuits are stored as subdirectories in the following directories:
```
📦circuit_database
 ┣ 📂circom
 ┣ 📂gnark
 ┗ 📂halo2
```
# Uploading Circuits via Sindri's Interactive Website
Circuit uploads to the website must be archived in a **gzip** tarfile (`.tar.gz`). *`.zip` compression is not currently supported, but is in development.*

Within each circuit directory, we include a `.tar.gz` and a `.zip` of the circuit directory so downloading a circuit from GitHub and uploading it to Sindri's interactive website is simple.

