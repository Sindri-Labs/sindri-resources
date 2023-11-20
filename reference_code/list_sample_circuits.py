#!/usr/bin/python3
import os
import pathlib
from glob import glob

RELPATH_OF_THIS_DIRECTORY = os.path.relpath(pathlib.Path(__file__).parent.resolve())

CIRCUIT_PARENT_DIRS = ["../circom/", "../gnark/", "../halo2/", "../noir/"]


def get_circuit_dirs_from_parent_dir(parent_dir) -> list[str]:
    """
    Return all child dirs of parent_dir.
    """
    circuit_dirs = []
    for circuit_dir in glob(os.path.join(parent_dir, "*", "")):
        circuit_dirs.append(os.path.relpath(circuit_dir))
    circuit_dirs.sort()
    return circuit_dirs


def get_sample_circuits() -> list[str]:
    """
    Return relative paths (from current working directory) to all sample circuits.
    """
    circuit_dirs = []
    for parent_dir in CIRCUIT_PARENT_DIRS:
        parent_dir = os.path.join(RELPATH_OF_THIS_DIRECTORY, parent_dir)
        circuit_dirs += get_circuit_dirs_from_parent_dir(parent_dir)
    circuit_dirs.sort()
    return circuit_dirs


if __name__ == "__main__":
    # Print to stdout
    for circuit_dir in get_sample_circuits():
        print(circuit_dir)
