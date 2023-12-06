# Sindri Circuit Database
This directory contains sample circuits for interacting with the Sindri Labs' API.
The `README.md` within each framework directory contains information specific to that circuit type.

Each sample circuit will include:
- A `README.md` 
- A `sindri.json` file (required)
- An example input file for proving the circuit
- Files specific to the circuit

Sample circuits are stored as subdirectories in the following directories:
```
📦circuit_database
 ┣ 📂circom
 ┣ 📂gnark
 ┣ 📂halo2
 ┗ 📂noir
```
# Uploading Circuits via Sindri's Interactive Website
Circuit uploads to the website must be archived in a **gzip** tarfile (`.tar.gz`). *`.zip` compression is not currently supported, but is in development.*

We include a `.tar.gz` and a `.zip` of each circuit directory so downloading a circuit from GitHub and uploading it to Sindri's interactive website is simple.

# Other
**`_prepare_circuit_database.py` is a utility script for maintainers of this repository** for compressing all sample circuits in `circuit_database/` to be easily used as file uploads in the interactive website.
- It ensures every sample circuit directory in one of the framework subdirectories is a valid circuit - it contains the necessary files for an upload to the Sindri API.
- The script can be run from any directory. It uses relative paths from the file location (you do not have to worry about the paths being messed up because of your shell's current working directory).
- The script has no requirements other than `python3`.
- Running the compression multiple times will overwrite existing compressed objects.

### Usage
```bash
$ python3 _prepare_circuit_database.py --help
usage: _prepare_circuit_database.py [-h] (-c | -r) [-d] [-q]

options:
  -h, --help      show this help message and exit
  -c, --compress  Run compression (default: False)
  -r, --remove    Remove compressed objects (default: False)
  -d, --dry-run   Dry run (default: False)
  -q, --quiet     Suppress verbose stdout (default: False)

To run compression/removal, you must use the --compress OR the --remove flags:
  python3 prepare_circuit_database.py --compress
  python3 prepare_circuit_database.py --remove

Add the --dry-run flag to perform a dry run:
  python3 prepare_circuit_database.py --compress --dry-run
  python3 prepare_circuit_database.py --remove --dry-run

Add the --quiet flag to suppress verbose stdout:
  python3 prepare_circuit_database.py --compress --quiet
```

