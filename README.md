# Forge Sample Data
Sample data for Sindri Labs' Forge API

# Instructions for compressing your code
- Forge code uploads must be archived in a gzip tarfile (`.tar.gz`).
- Forge expects a tarfile of a directory.
- Even if you have 1 file, place the file a directory and archive that entire directory, not just the single file.

# Circom
## Requirements
- The main component file of your circom circuit must be named `main.circom`
- All code imports must use relative paths

## Structure
```
my_repo/
    main.circom
    supplementary.circom
```
## How to compress properly
To prepare your repo, `my_repo`, for Forge upload, run the following command ***from the parent directory*** of `my_repo`:
```
tar -zcvf my_repo.tar.gz my_repo/
```
*Note: The `my_repo` portion of `my_repo.tar.gz` may be called anything.*

## Invalid compression example
Your repo may only have 1 circom file. Do not compress only the `main.cirom` file. You must still compress the entire repo directory.

Example structure (only 1 circom file):
```
my_repo/
    main.circom
```

Invalid compression:
```bash
# INVALID
cd my_repo/
tar -zcvf my_repo.tar.gz main.circom
```


# Halo2