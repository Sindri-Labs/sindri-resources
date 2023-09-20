# Example Code

This directory contains examples of code interacting with Forge.

## Quick-Start Scripts

We provide quick-start scripts in both JavaScript ([`quickstart.js`](./quickstart.js)) and Python ([`quickstart.py`](./quickstart.py)).
These scripts will create a Circom circuit object in Forge, upload a gzipped sample circuit file (located at [`../circom/multiplier2.tar.gz`](../circom/multiplier2.tar.gz)), and compile it.
Once a proof has finished executing, the code will then verify that proof.

You will need a Forge API key in order to run the scripts.
The Forge GitBook explains how to obtain your API key [here](https://sindri-labs.gitbook.io/forge/ZpTt7gQVuHU2jgnnKBQl/forge/using-forge/access-management#api-authentication).
You can then either set a `FORGE_API_KEY` environment variable with your API key, or modify the value of the `API_KEY` global variable in the scripts before running them.

### JavaScript ([`quickstart.js`](./quickstart.js))

* Make sure you have Node.js installed.
* Install the necessary dependencies by running `npm install axios form-data`.
* Inside this directory (`example-code`), run the script using `FORGE_API_KEY=<your-api-key> node quickstart.js`.

### Python ([`quickstart.py`](./quickstart.py))

* Install Python 3.
* Install the python [requests library](https://pypi.org/project/requests/).
* Inside of this directory (`example-code`), invoke the python script in the command line via `FORGEAPI_KEY=<your-api-key python3 quickstart.py`.

### Expected Outcome

For both scripts, you should see the following printed to `stdout`:

```
1. Creating circuit...
Circuit poll exited after 9 seconds with status: Ready
Circuit compilation succeeded!
2. Proving circuit...
Proof poll exited after 0 seconds with status: Ready
3. Verifing proof...
Proof was valid
Circuit proof output signal: 294
```

Note that the circuit computes the product of two inputs `a=7` and `b=42`, so the output signal should change accordingly when you alter `proof_input`/`proofInput` in the input scripts.
