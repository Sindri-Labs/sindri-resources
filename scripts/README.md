# EZ Scripts
The `ez_*.py` scripts use the `sindri_sdk.py`.

1. Paste your Sindri API Key into the `../API_KEY` file or set the `FORGE_API_KEY` environment variable.
2. Prepare the sample circuits if you have not already: `python3 compress_sample_circuits.py`
3. Run an EZ script to create a circuit and prove it! `python3 ez_circom.py`


# Quick-Start Scripts

We provide quick-start scripts in JavaScript ([`quickstart.js`](./quickstart.js)) and Python ([`quickstart.py`](./quickstart.py)) as well as compilable quick-start rust code ([`rust_quickstart/src/main.rs`](./rust_quickstart/src/main.rs)).
These scripts will create a Circom circuit object in Forge, upload a gzipped sample circuit file (located at [`../circom/multiplier2/multiplier2.tar.gz`](../circom/multiplier2/multiplier2.tar.gz)), and compile it.
Once a proof has finished executing, the code will then print the public outputs from the circuit.

You will need a Forge API key in order to run the scripts.
The Forge GitBook explains how to obtain your API key [here](https://sindri-labs.gitbook.io/forge/ZpTt7gQVuHU2jgnnKBQl/forge/using-forge/access-management#api-authentication).
You can then either set a `FORGE_API_KEY` environment variable with your API key, or modify the value of the `API_KEY` global variable in the scripts before running them.

Before running the scripts, make sure you prepared the sample data: `python3 compress_sample_circuits.py`

### JavaScript ([`quickstart.js`](./quickstart.js))

* Make sure you have Node.js installed.
* Install the necessary dependencies by running `npm install axios form-data`.
* Inside this directory (`scripts`), run the script using `FORGE_API_KEY=<your-api-key> node quickstart.js`.

### Python ([`quickstart.py`](./quickstart.py))

* Install Python 3.
* Install the python [requests library](https://pypi.org/project/requests/).
* Inside of this directory (`scripts`), invoke the python script in the command line via `FORGE_API_KEY=<your-api-key> python3 quickstart.py`.

### Rust ([`rust_quickstart/src/main.rs`](./rust_quickstart/src/main.rs))

* Install [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
* Inside the `rust_quickstart` directory, build and run the main source file in the command line via `FORGE_API_KEY=<your-api-key> cargo run`.

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
