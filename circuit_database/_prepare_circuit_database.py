#!/usr/bin/python3
import json
import os
import pathlib
import sys
import tarfile
import zipfile
from glob import glob

global QUIET
QUIET = False

RELPATH_OF_THIS_DIRECTORY = os.path.relpath(pathlib.Path(__file__).parent.resolve())

CIRCUIT_PARENT_DIRS = [
    "circom/",
    "gnark/v0.8.1/",
    "gnark/v0.9.0/",
    "halo2/axiom-v0.2.2/",
    "halo2/axiom-v0.3.0/",
    "noir/",
]


def compress_dirs(source_dirs: list[str] | list[pathlib.Path], dry_run: bool = False) -> None:
    """
    For every `source_dir` in `source_dirs`, create/overwrite
    `<source_dir>.tar.gz` and `<source_dir>.zip`.
    ```txt
    tar -zcvf <source_dir>.tar.gz source_dir
    zip -r    <source_dir>.zip    source_dir
    ```
    """
    print(f"\nBEGIN: (Re)compressing {len(source_dirs)} circuit(s)\n")
    for source_dir in source_dirs:
        if not QUIET:
            print(f"\nSource {str(source_dir)}")

        targz_file, zip_file = get_compressed_file_paths(source_dir)

        if not dry_run:
            with tarfile.open(targz_file, "w:gz") as tar:
                tar.add(source_dir, arcname=targz_file.name)
        if not QUIET:
            print(f"       {targz_file}")

        if not dry_run:
            with zipfile.ZipFile(zip_file, mode="w") as archive:
                for file_path in pathlib.Path(source_dir).iterdir():
                    archive.write(file_path, arcname=file_path.name)
        if not QUIET:
            print(f"       {zip_file}")
    print(f"\nSUCCESS. Compressed {len(source_dirs)} circuit(s).")


def get_child_dirs_from_parent_dirs(circuit_parent_dirs: list[str]) -> list[str]:
    """
    Return all child dirs of every parent dir in `circuit_parent_dirs`.
    Transform them to relative paths based on the current working directory.
    """
    circuit_dirs = []
    for parent_dir in circuit_parent_dirs:
        for circuit_dir in glob(os.path.join(RELPATH_OF_THIS_DIRECTORY, parent_dir, "*", "")):
            circuit_dirs.append(os.path.relpath(circuit_dir))
    circuit_dirs.sort()
    return circuit_dirs


def get_compressed_file_paths(source_dir: str | pathlib.Path) -> list[pathlib.Path]:
    """
    Return the relative file paths (based on current working directory)
    to the compressed files for `source_dir`.
    ```
    # Input
    /path_to/source_dir/
    # Output
    /path_to/source_dir.tar.gz
    /path_to/source_dir.zip
    ```
    """
    source_dir = pathlib.Path(os.path.relpath(source_dir))
    if not source_dir.is_dir():
        raise ValueError("source_dir must be a directory")
    return [source_dir.with_suffix(".tar.gz"), source_dir.with_suffix(".zip")]


def remove_compressed_artifacts(
    source_dirs: list[str] | list[pathlib.Path], dry_run: bool = False
) -> None:
    """
    For every `source_dir` in `source_dirs`, remove `<source_dir>.tar.gz` and `<source_dir>.zip`.
    ```txt
    rm -f <source_dir>.tar.gz
    rm -f <source_dir>.zip
    ```
    """
    print(f"\nBEGIN: Removing {len(source_dirs)} circuit(s) artifacts\n")
    num_removed = 0
    for source_dir in source_dirs:
        file_paths = get_compressed_file_paths(source_dir)
        for file_path in file_paths:
            if os.path.exists(file_path):
                if not dry_run:
                    os.remove(file_path)
                num_removed += 1
                if not QUIET:
                    print(f"Removed {file_path}")
    print(f"\nSUCCESS. Removed {num_removed} artifacts for {len(source_dirs)} circuit(s).")


def validate_circuit_dirs(circuit_dirs: list[str] | list[pathlib.Path]) -> None:
    """
    Ensure that every circuit_dir in `circuit_dirs` is a valid circuit.

    Exit if `circuit_dir`
    - is missing a `README.md`
    - is missing an example input file
    - is missing a `Sindri.json` or `sindri.json` file
    - the `Sindri.json` file is not a dictionary
    - the `Sindri.json` file has no duplicate keys (case-insensitive)
    - the `Sindri.json` field is missing the `circuit_type` key (case-insensitive).
    - the `Sindri.json` field is missing the `name` key (case-insensitive).

    TODO: Use sindri_manifest JSON schema from Sindri API for validation for proper validation.
    """
    for circuit_dir in circuit_dirs:
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

        # Check required keys for all circuit types
        if "circuit_type" not in sindri_json_keys_lower:
            sys.exit(f"\nERROR: {sindri_json_path} is missing the `circuit_type` key.")
        if "name" not in sindri_json_keys_lower:
            sys.exit(f"\nERROR: {sindri_json_path} is missing the `name` key.")


def _print_help_message() -> None:
    print("\nTo run compression/removal, use the --compress OR the --remove flags:")
    print("  python3 _prepare_circuit_database.py --compress")
    print("  python3 _prepare_circuit_database.py --remove")
    print("\nAdd the --dry-run flag to perform a dry run:")
    print("  python3 _prepare_circuit_database.py --compress --dry-run")
    print("  python3 _prepare_circuit_database.py --remove --dry-run")
    print("\nAdd the --quiet flag to suppress verbose stdout:")
    print("  python3 _prepare_circuit_database.py --compress --quiet")
    print()


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument("-c", "--compress", action="store_true", help="Run compression")
    parser.add_argument("-r", "--remove", action="store_true", help="Remove compressed objects")
    parser.add_argument("-d", "--dry-run", action="store_true", help="Dry run")
    parser.add_argument("-q", "--quiet", action="store_true", help="Suppress verbose stdout")
    try:
        args = parser.parse_args()
    except:  # NOTE: This does not work as intended without bare except.
        parser.print_help()
        _print_help_message()
        sys.exit(0)

    if args.quiet:
        QUIET = True

    dry_run = args.dry_run
    compress = args.compress
    remove = args.remove
    if dry_run:
        print("\n***DRY RUN BEGIN***\n")
    if not compress and not remove:
        print("No command specified.")
        prompt = input("\nContinue and run compression? (--compress)\n[y/n]\n")
        if prompt.lower() == "y":
            compress = True
        else:
            sys.exit("Goodbye.")

    if not QUIET:
        print(
            "\nThis script compresses all circuit folders using"
            " multiple methods (.tar.gz, .zip) and places the compressed"
            " artifacts in the parent folder of the circuit folder.\n"
        )

    # Obtain circuit_dirs from CIRCUIT_PARENT_DIRS
    circuit_dirs = get_child_dirs_from_parent_dirs(CIRCUIT_PARENT_DIRS)

    # Validate all circuit_dirs
    validate_circuit_dirs(circuit_dirs)

    if remove:
        # Remove compressed artifacts for given circuit_dirs
        remove_compressed_artifacts(circuit_dirs, dry_run=dry_run)
    elif compress:
        # Compress all circuit_dirs
        compress_dirs(circuit_dirs, dry_run=dry_run)
    if dry_run:
        print("\n***DRY RUN SUCCESS***")
        if not QUIET:
            _print_help_message()
