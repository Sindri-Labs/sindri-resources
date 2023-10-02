# Sample Gnark Circuits

In this directory, we provide a handful of Gnark circuits which have already been prepared to use with Forge. You may find them useful as a reference when writing your own circuits; or you may use these as circuit uploads if you are just beginning to try out Forge.

All of the tar files are compressed versions of the corresponding folder. Rather than uploading the entire directory to Forge, you will upload it's compressed .tar.gz file. In order to request a proof for one of these circuits, consult the input.json file in the corresponding folder.

It is allowed (but unneccesary) to include extraneous files in your compressed directory such as readme's, example inputs, and unused code. However, you should not include any executables.

See Sindri's GitBook for more complete instructions.

## Available Circuits

### Gnark v0.8.1 (Gnark-Crypto v0.9.1)

| Name | Size (DEGREE) | Original Source | Functionality | 
| ---- | ---- | --------------- | ------------- | 
| cubic | 3 | [Gnark Github Examples](https://github.com/Consensys/gnark/blob/master/examples/cubic/cubic.go) | Checks $x^3 + x + 5 == y$ for public $y$ and private $x$|
| poseidon | 214 | [Vocdoni's Gnark Primitives](https://github.com/vocdoni/gnark-crypto-primitives/tree/main) | Computes the Poseidon hash of a preimage  |


## Circuits Requirements
- Upload all your go language source code for a circuit
  - include your module definition file `go.mod`
  - our current prover configuration uses gnark `v0.8.1` and gnark-crypto `v0.9.1`. We cannot guarantee compatibility with circuits relying on newer features.
- Your main circuit struct should be public
- Indicate the main circuit upon which we build our prover via the `Sindri.json` file
- Your main circuit should be able to instantiate an assignment for your main circuit from a json path
```
func FromJson(pathInput string) witness.Witness {}
```

### Supporting Files
The file `Sindri.json` specifies what you have named your package and how to reference the circuit.  In this file, you will also indicate the Gnark version your circuit was written with, as well as the proving scheme and curve.
```
{
    "PACKAGE_NAME": "PackageName",
    "CIRCUIT_NAME": "MainCircuitDef",
    "VERSION": "8.1",
    "CURVE_NAME": "bn254",
    "PROVING_SCHEME": "Groth16"
}
```

### Current Support

| Type        | Field       | Status       |
| ----------- | ----------- | --- |
| `PROVING_SCHEME`      | Groth16       |  ✅   | 
| `PROVING_SCHEME`      | Plonk       |  Coming Soon   | 
| `CURVE_NAME`      | bn254       |  ✅   | 
| `CURVE_NAME`      | BLS12-381       |  Coming Soon   | 
| `CURVE_NAME`      | BLS12-377       |  Coming Soon   | 
| `CURVE_NAME`      | BLS24-315       |  Coming Soon   | 
| `CURVE_NAME`      | BW6-633      |  Coming Soon   | 
| `CURVE_NAME`      | BW6-761       |  Coming Soon   | 
| `VERSION`      | (Gnark) 8.1       |  ✅   | 
| `VERSION`      | (Gnark) 9.0-alpha       |  Coming Soon   | 

### How to compress properly
To prepare your repo, `my_repo`, for Forge upload, run the following command ***from the parent directory*** of `my_repo`:
```
tar -zcvf my_repo.tar.gz my_repo/
```
*Note: The `my_repo` portion of `my_repo.tar.gz` may be called anything.*

