#!/usr/bin/python3
import json
import os
import pathlib
import shutil
import sys
import tarfile
import zipfile
from glob import glob

"""
This script is a utility for locally validating sample circuits and compressing them. 
- It assumes every sample circuit directory in one of the framework subdirectories is a valid circuit. 
- The script can be run from any directory. It uses relative paths from the file location (you do not have to worry about the paths being messed up because of your shell's current working directory).
- All compressed circuit objects that are created by the `_prepare_circuit_database.py` script for a circuit are stored inside the circuit's directory.
- The script has no requirements other than `python3`.
- The resulting compressed circuits are gitignored (see `.gitignore`).

**What happens if I run the script multiple times?**
- The script will remove existing compressed objects before recompressing the directory, ***provided that the circuit directory was not renamed***!

#### Dry Run
`./_prepare_circuit_database.py --dry-run` option

The **Dry Run** will perform a validity check on all of the supplied circuits directories. 
- If a valid circuit directory does not have a `README.md` or a valid `sindri.json` file, it will exit with an error.

#### Usage
```
$ python3 _prepare_circuit_database.py --help
```

If `--compress` or `--dry-run` are not supplied, the script will prompt for `y/n` to continue with running compression.
- If `n` is supplied or `--dry-run` is specified, the script will only perform a **dry run**.
"""


RELPATH_OF_THIS_DIRECTORY = os.path.relpath(pathlib.Path(__file__).parent.resolve())

CIRCUIT_PARENT_DIRS_TO_BE_COMPRESSED = [
    "../circom/",
    "../gnark/",
    "../halo2/",
]


MANUAL_DIRS_TO_BE_COMPRESSED = []
# Deduplicate just in case
MANUAL_DIRS_TO_BE_COMPRESSED = list(set(MANUAL_DIRS_TO_BE_COMPRESSED))
MANUAL_DIRS_TO_BE_COMPRESSED.sort()



def compress_circuit_dir(output_filename, source_dir, remove_only: bool = False):
    """
    1. Remove source_dir/output_filename.tar.gz, source_dir/output_filename.zip
    2. Create output_filename.tar.gz, output_filename.zip from source_dir
    3. Move output_filename.tar.gz, output_filename.zip into source_dir
    """
    if not QUIET:
        if not remove_only:
            print(f"(Re)compressing circuit directory: {source_dir}")
        else:
            print(f"Removing circuit compressed objects: {source_dir}")

    final_tarfile_name = output_filename + ".tar.gz"
    final_tarfile_destination = os.path.join(source_dir, final_tarfile_name)
    final_zipfile_name = output_filename + ".zip"
    final_zipfile_destination = os.path.join(source_dir, final_zipfile_name)

    try:
        os.remove(final_tarfile_destination)
    except FileNotFoundError:
        pass
    try:
        os.remove(final_zipfile_destination)
    except FileNotFoundError:
        pass

    if remove_only:
        return

    with tarfile.open(final_tarfile_name, "w:gz") as tar:
        tar.add(source_dir, arcname=output_filename)

    with zipfile.ZipFile(final_zipfile_name, mode="w") as archive:
        for file_path in pathlib.Path(source_dir).iterdir():
            archive.write(file_path, arcname=file_path.name)

    shutil.move(final_tarfile_name, final_tarfile_destination)
    shutil.move(final_zipfile_name, final_zipfile_destination)


def validate_circuit_directory(circuit_dir) -> None:
    """
    Exit if `circuit_dir`
    - is missing a `README.md`
    - is missing an `input.json` file
    - is missing a `Sindri.json` or `sindri.json` file
    - the `Sindri.json` file is a dictionary
    - the `Sindri.json` file has no duplicate keys (case-insensitive)
    - the `Sindri.json` field is missing the `circuit_type` key (case-insensitive).
    """
    if not QUIET:
        print(f"VALIDATING: {circuit_dir}")

    # Ensure the directory exists
    if not os.path.exists(circuit_dir):
        sys.exit(f"\nERROR: {circuit_dir} does not exist.")

    # Ensure circuit_dir contains a README.md
    readme_path = os.path.join(circuit_dir, "README.md")
    if not os.path.exists(readme_path):
        sys.exit(f"\nERROR: {circuit_dir} is missing a README.md file.")

    # Ensure circuit_dir contains an input.json
    # or Prover.toml (Noir circuits only)
    input_path = os.path.join(circuit_dir, "input.json")
    noir_input_path = os.path.join(circuit_dir, "Prover.toml")
    if not os.path.exists(input_path) and not os.path.exists(noir_input_path):
        sys.exit(f"\nERROR: {circuit_dir} is missing an input.json or Prover.toml file.")

    # Ensure circuit_dir contains a Sindri.json or sindri.json
    sindri_json_path = os.path.join(circuit_dir, "Sindri.json")
    if not os.path.exists(sindri_json_path):
        sindri_json_path = os.path.join(circuit_dir, "sindri.json")
        if not os.path.exists(str(sindri_json_path).lower()):
            sys.exit(f"\nERROR: {circuit_dir} is missing a Sindri.json file.")

    # Load the contents of the Sindri.json
    sindri_json = {}
    with open(sindri_json_path, "r") as f:
        sindri_json = json.load(f)
    if not isinstance(sindri_json, dict):
        sys.exit(f"\nERROR: {circuit_dir} Sindri.json is not a dictionary.")

    sindri_json_keys_lower = [key.lower() for key in sindri_json.keys()]

    # Ensure there are no duplicate Sindri.json keys (case-insensitive)
    if len(sindri_json_keys_lower) != len(set(sindri_json_keys_lower)):
        sys.exit(f"\nERROR: {sindri_json_path} has duplicate keys (case-insensitive).")

    if "circuit_type" not in sindri_json_keys_lower:
        sys.exit(f"\nERROR: {sindri_json_path} is missing the `circuit_type` key.")


def get_valid_circuit_dirs_from_parent_dir(parent_dir) -> list[str]:
    """
    Look at all child dirs of parent_dir. If the child dir is a valid circuit,
    add it to the list of returned dirs that should be compressed.
    """
    valid_circuit_dirs = []

    child_dirs = glob(os.path.join(parent_dir, "*", ""))
    child_dirs.sort()
    for child_dir in child_dirs:
        validate_circuit_directory(child_dir)
        valid_circuit_dirs.append(child_dir)
    valid_circuit_dirs.sort()
    return valid_circuit_dirs


def get_circuit_dirs_from_parent_dir(parent_dir) -> list[str]:
    """
    Return all child dirs of parent_dir.
    """
    circuit_dirs = []
    for circuit_dir in glob(os.path.join(parent_dir, "*", "")):
        circuit_dirs.append(os.path.relpath(circuit_dir))
    circuit_dirs.sort()
    return circuit_dirs


global QUIET
QUIET = False

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "-c", "--compress", action="store_true", help="Run compression (omit for dry run)"
    )
    parser.add_argument(
        "-r",
        "--remove-only",
        action="store_true",
        help="Only remove compressed objects (requires -c)",
    )
    parser.add_argument("-d", "--dry-run", action="store_true", help="Run dry run (validate files)")
    parser.add_argument("-q", "--quiet", action="store_true", help="Quiet: Suppress stdout")
    args = parser.parse_args()
    if args.quiet:
        QUIET = True

    remove_only = args.remove_only
    compress = args.compress
    if not args.dry_run and not compress:
        if remove_only:
            prompt = input("Run removal? [y/n]")
        else:
            prompt = input("Run compression? [y/n]")
        if prompt.lower() == "y":
            compress = True

    if not compress:
        print("PERFORMING DRY RUN. Run with '-c' or '--compress' to run compression.")

    if not QUIET:
        print("\nThis script compresses all circuit folders using multiple methods (.tar.gz, .zip)")
        print("and places the compressed results inside their respective circuit folder.\n")

    circuit_dirs_to_compress = []

    # Detect and add circuit dirs from CIRCUIT_PARENT_DIRS_TO_BE_COMPRESSED
    for parent_dir in CIRCUIT_PARENT_DIRS_TO_BE_COMPRESSED:
        parent_dir = os.path.join(RELPATH_OF_THIS_DIRECTORY, parent_dir)
        circuit_dirs_to_compress += get_circuit_dirs_from_parent_dir(parent_dir)

    # Add circuit dirs from MANUAL_DIRS_TO_BE_COMPRESSED
    for circuit_dir in MANUAL_DIRS_TO_BE_COMPRESSED:
        circuit_dir = os.path.join(RELPATH_OF_THIS_DIRECTORY, circuit_dir)
        circuit_dirs_to_compress.append(circuit_dir)

    # Deduplicate and sort
    circuit_dirs_to_compress = list(set(circuit_dirs_to_compress))
    circuit_dirs_to_compress.sort()

    num_circuits = len(circuit_dirs_to_compress)

    # Validate all circuit_dirs_to_compress
    for circuit_dir in circuit_dirs_to_compress:
        validate_circuit_directory(circuit_dir)

    if compress:
        # Compress all circuit_dirs_to_compress
        if not QUIET:
            print(f"\nBEGIN: {num_circuits} circuit(s)\n")
        for circuit_dir in circuit_dirs_to_compress:
            last_dir_name = pathlib.PurePath(circuit_dir).name
            compress_circuit_dir(last_dir_name, circuit_dir, remove_only=remove_only)

        if not QUIET:
            print(f"\nSUCCESS. {num_circuits} valid circuit(s)")
        else:
            print(f"SUCCESS. {num_circuits} valid circuit(s)")
    else:
        if not QUIET:
            print(f"\nDRY RUN SUCCESS. {num_circuits} valid circuit(s)")
            print("Run with '-c' or '--compress' to run compression.\n")
        else:
            print(f"DRY RUN SUCCESS. {num_circuits} valid circuit(s)")
