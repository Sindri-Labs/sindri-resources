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

## `_prepare_circuit_database.py`
**TL;DR: `python3 _prepare_circuit_database.py --compress`**

**The `_prepare_circuit_database.py` script is a utility for compressing all sample circuits in `circuit_database/` to be easily used as file uploads in the interactive website.**
- It ensures every sample circuit directory in one of the framework subdirectories is a valid circuit - it contains the necessary files for an upload to the Sindri API.
- The script can be run from any directory. It uses relative paths from the file location (you do not have to worry about the paths being messed up because of your shell's current working directory).
- All compressed objects for a circuit are stored in the circuit's parent directory and are gitignored (see `.gitignore`).
- The script has no requirements other than `python3`.

### Usage
```bash
usage: _prepare_circuit_database.py [-h] [-c] [-r] [-d] [-q]

options:
  -h, --help      show this help message and exit
  -c, --compress  Run compression (default: False)
  -r, --remove    Remove compressed objects (default: False)
  -d, --dry-run   Dry run (default: False)
  -q, --quiet     Suppress verbose stdout (default: False)

To run compression/removal, use the --compress OR the --remove flags:
  python3 _prepare_circuit_database.py --compress
  python3 _prepare_circuit_database.py --remove

Add the --dry-run flag to perform a dry run:
  python3 _prepare_circuit_database.py --compress --dry-run
  python3 _prepare_circuit_database.py --remove --dry-run

Use just the --dry-run flag to only perform validation:
  python3 _prepare_circuit_database.py --dry-run

Add the --quiet flag to suppress verbose stdout:
  python3 _prepare_circuit_database.py --compress --quiet
```

What happens if I run the compression script multiple times?
- The script will overwrite existing compressed objects!

What happens if I do not include either the `--compress` or `--remove` flags?
- The script will prompt for `[y/n]` to continue with running compression (`--compress`).
