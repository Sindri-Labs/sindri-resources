# Sindri API Sample Data
This repo contains sample circuit data and utility scripts for interacting with the Sindri Labs' API. The readme within each framework directory will contain information specific to that circuit type.


# Sample Circuits
Each sample circuit will include:
- A `README.md` 
- A `sindri.json` file (required)
- An example input file (`input.json`) for a proving the circuit
- Files specific to the circuit

Sample circuits are stored as subdirectories in the following directories
```
circom/
gnark/
halo2/
noir/
```

## Preparing Circuits for Upload
Circuit uploads must be archived in a **gzip** tarfile (`.tar.gz`). `.zip`` compression is not currently supported, but is in development.

### Utility script: `compress_sampledata.py`

The `compress_sampledata.py` script is a utility for locally validating sample circuits and compressing them. 
- It assumes every sample circuit directory in one of the framework subdirectories is a valid circuit. 
- The script can be run from any directory. It uses relative paths from the file location (you do not have to worry about the paths being messed up because of your shell's current working directory).
- All compressed circuit objects that are created by the `compress_sampledata.py` script for a circuit are stored inside the circuit's directory.
- The script has no requirements other than `python3`.
- The resulting compressed circuits are gitignored (see `.gitignore`).

**What happens if I run the script multiple times?**
- The script will remove existing compressed objects before recompressing the directory, ***provided that the circuit directory was not renamed***!

#### Dry Run
`./compress_sampledata.py --dry-run` option

The **Dry Run** will perform a validity check on all of the supplied circuits directories. 
- If a valid circuit directory does not have a `README.md` or a valid `sindri.json` file, it will exit with an error.

#### Usage
```
$ python3 compress_sampledata.py --help
usage: compress_sampledata.py [-h] [-c] [-r] [-d] [-q]

options:
  -h, --help         show this help message and exit
  -c, --compress     Run compression (omit for dry run) (default: False)
  -r, --remove-only  Only remove compressed objects (requires -c) (default: False)
  -d, --dry-run      Run dry run (validate files) (default: False)
  -q, --quiet        Quiet: Suppress stdout (default: False)
```

If `--compress` or `--dry-run` are not supplied, the script will prompt for `y/n` to continue with running compression.
- If `n` is supplied or `--dry-run` is specified, the script will only perform a **dry run**.

