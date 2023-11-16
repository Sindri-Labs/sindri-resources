# Forge Sample Data
This repo contains sample data for Sindri Labs' Forge API.  The readme within each framework directory will contain information specific to that circuit type.


## Common Conventions

The following information applies to all circuit types.

- Each example will include a `.tar.gz` file outside the code directory which may be directly uploaded as a circuit.
- Example input for a proof has been included within the source code folder for any example circuit.

#### Compressing Circuit Code
- Forge code uploads must be archived in a **gzip** tarfile (`.tar.gz`), zip compression is not currently supported.
- Forge expects a tarfile of a directory.
- Even if you have 1 file, place the file a directory and archive that entire directory, not just the single file.
