# Reference Code
This directory contains quick-start code in multiple languages for interfacing with the Sindri API.

# Setup
You will need a Sindri API key in order to run the scripts.
The Sindri Docs explain [how to obtain your API key](https://sindri.app/docs/topic-guides/access-management/#api-authentication).

You can then set the `SINDRI_API_KEY` environment variable with your API key.

# Verifiers

This directory contains scripts and demonstrations generating, deploying, and interacting with smart contracts that verify proofs from Sindri's API.

# SDK Quickstart

[`sdk_quickstart.py`](./sdk_quickstart.py) is for users that are just getting started using Sindri's API.
It utilizes the [Sindri Python SDK](https://pypi.org/project/sindri-labs/), which abstracts Sindri API calls into a simple interface.

Refer to the Sindri documentation for quickly [getting started with the Python SDK](https://sindri.app/docs/getting-started/api-sdk/).

# Quick-Start Scripts

For users that want less abstracted API calls that they can adapt within their own pipeline, we provide quick-start scripts in JavaScript ([`quickstart.js`](./quickstart.js)) and Python ([`quickstart.py`](./quickstart.py)) as well as compilable quick-start rust code ([`quickstart_rust/src/main.rs`](./quickstart_rust/src/main.rs)).
These scripts will create a Circom circuit in Sindri, create a gzipped upload from the sample circuit folder located at [`../circuit_database/circom/multiplier2/`](../circuit_database/circom/multiplier2/), and compile it.
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