## Compression Circuit

This circuit is built with Gnark frontend v0.9.0 and currently accepts two inputs similar to 
```
{
    "original": "aaabbcccccc", 
    "compressed": "a3b2c6"
}
```

Where the compressed string collapses each symbol with multiplicity 1-9.  See the docs for a more thorough explanation of the functionality.

Within `circuit.go` we have the circuit definition
```
func (circuit *Circuit) Define(api frontend.API) error { }
```
which iterates through each symbol in the original string and ensures that the corresponding pointer in the compressed list matches.

Note that the inputs to the circuit are not the strings represented in this JSON input, but preprocessed versions of these strings which are turned into integer lists.  From the input example above, we would actually input:
```
"X": [97,97,97,98,98,99,99,99,99,99,99,0,0,0,...],
"Y": [97,3,98,2,99,6,0,0,...]
```
to the circuit and that is what `FromJson` does.

### TODO LIST

- [ ] Compute the compressed representation within `FromJson` so that the proof input is only one string
- [ ] Incorporate a hash of the original string which is another public variable returned to the user
- [ ] Give circuit internal & external signals better names
- [ ] Explore how large we can make the strings before the circuit becomes huge (currently max length of x is 100 and we have 80K constraints)
