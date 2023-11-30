#!/usr/bin/python3
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
    print(f"\nBEGIN: (Re)compressing {len(source_dirs)} circuit(s)")
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


def _print_help_message() -> None:
    print("\nTo run compression/removal, you must use the --compress OR the --remove flags:")
    print("  python3 prepare_circuit_database.py --compress")
    print("  python3 prepare_circuit_database.py --remove")
    print("\nAdd the --dry-run flag to perform a dry run:")
    print("  python3 prepare_circuit_database.py --compress --dry-run")
    print("  python3 prepare_circuit_database.py --remove --dry-run")
    print("\nAdd the --quiet flag to suppress verbose stdout:")
    print("  python3 prepare_circuit_database.py --compress --quiet")
    print()


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("-c", "--compress", action="store_true", help="Run compression")
    group.add_argument("-r", "--remove", action="store_true", help="Remove compressed objects")
    parser.add_argument("-d", "--dry-run", action="store_true", help="Dry run")
    parser.add_argument("-q", "--quiet", action="store_true", help="Suppress verbose stdout")
    try:
        args = parser.parse_args()
    except SystemExit:
        # Catch argparse error so we can add additional commentary for the user
        _print_help_message()
        sys.exit()

    if args.quiet:
        QUIET = True

    dry_run = args.dry_run
    compress = args.compress
    remove = args.remove

    if dry_run:
        print("\n***DRY RUN BEGIN***\n")

    if not QUIET:
        print(
            "This script compresses all circuit folders using"
            " multiple methods (.tar.gz, .zip) and places the compressed"
            " artifacts in the parent folder of the circuit folder."
        )

    # Obtain circuit_dirs from CIRCUIT_PARENT_DIRS
    circuit_dirs = get_child_dirs_from_parent_dirs(CIRCUIT_PARENT_DIRS)

    if remove:
        # Remove compressed artifacts for given circuit_dirs
        remove_compressed_artifacts(circuit_dirs, dry_run=dry_run)

    if compress:
        # Compress all circuit_dirs
        compress_dirs(circuit_dirs, dry_run=dry_run)
    
    if dry_run:
        print("\n***DRY RUN SUCCESS***")
