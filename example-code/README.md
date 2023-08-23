# Example Code

This directory contains examples of code interacting with Forge.

## A Quick Start Script

The python file `quickstart.py` will create a Circom circuit object in Forge.  Then it will upload a gzipped sample circuit file (cf. `circom/multiplier2` in this repo) and compile.  Once a proof has finished executing, the code will then verify that proof.

### To Run
* Install the python [requests library](https://pypi.org/project/requests/)
* Replace the string in line 6 of `quickstart.py` with your own API key.  (The Forge GitBook explains how to obtain your API key [here](https://sindri-labs.gitbook.io/forge/ZpTt7gQVuHU2jgnnKBQl/forge/using-forge/access-management#api-authentication))
* Inside of this directory (`example-code`), invoke the python script in the command line via `python3 quickstart.py`

### Expected Outcome

You should see the following printed to stdout:
```
Circom compilation succeeded!
Proof poll exited after 8 seconds with status: Ready
Circuit output signal: 294
Proof was valid
```
Note that the circuit computes the product of two inputs `a=7` and `b=42`, so the output signal should change accordingly when you alter line 73 of `quickstart.py`