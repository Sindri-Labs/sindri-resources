# Setup
You will need a Sindri API key in order to run the scripts.
The Sindri Docs explain [how to obtain your API key](https://sindri-labs.github.io/docs/forge/using-forge/access-management/#api-authentication).

You can then either set the `SINDRI_API_KEY` environment variable with your API key. Alternatively, you may modify the value of the `API_KEY` global variable in the below scripts before running them.


# Quick-Start Scripts

We provide quick-start scripts in JavaScript ([`quickstart.js`](./quickstart.js)) and Python ([`quickstart.py`](./quickstart.py)) as well as compilable quick-start rust code ([`quickstart_rust/src/main.rs`](./quickstart_rust/src/main.rs)).
These scripts will create a Circom circuit object in Sindri, upload a gzipped sample circuit file (located at [`../circom/multiplier2/multiplier2.tar.gz`](../circom/multiplier2/multiplier2.tar.gz)), and compile it.
Once a proof has finished executing, the code will then print the public outputs from the circuit.

### JavaScript ([`quickstart.js`](./quickstart.js))

- Make sure you have Node.js installed.
- Install the necessary dependencies by running `npm install axios form-data tar`.
- Inside this directory (`scripts`), run the script using `SINDRI_API_KEY=<your-api-key> node quickstart.js`.

### Python ([`quickstart.py`](./quickstart.py))

- Install Python 3.
- Install the python [requests library](https://pypi.org/project/requests/).
- Inside of this directory (`scripts`), invoke the python script in the command line via `SINDRI_API_KEY=<your-api-key> python3 quickstart.py`.

### Rust ([`quickstart_rust/src/main.rs`](./quickstart_rust/src/main.rs))

- Install [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Inside the `quickstart_rust` directory, build and run the main source file in the command line via `SINDRI_API_KEY=<your-api-key> cargo run`.

### Expected Outcome

For all three scripts, you should see the following printed to `stdout`:

```
1. Creating circuit...
Circuit poll exited after 9 seconds with status: Ready
Circuit compilation succeeded!
2. Proving circuit...
Proof poll exited after 0 seconds with status: Ready
Circuit proof output signal: 294
```

Note that the circuit computes the product of two inputs `a=7` and `b=42`, so the output signal should change accordingly when you alter `proof_input`/`proofInput` in the input scripts.

# Sindri SDK

### Python ([`sdk.py`](./sdk.py))
The `SindriSdk` class in `skd.py` abstracts the api calls into a simple interface.
- Install Python 3.
- Install the python [requests library](https://pypi.org/project/requests/).
- Inside of this directory (`scripts`), invoke the python script in the command line via `SINDRI_API_KEY=<your-api-key> python3 sdk_quickstart.py`.