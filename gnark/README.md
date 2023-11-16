# Valid Gnark sample data

Every directory is a valid circuit that contains
- `README.md`
- `Sindri.json` with `"circuit_type": "gnark"`

There are two versions of Gnark. They have shorthand versions in the items names.
- `v8`: `"gnark_version": "v0.8.1"`
- `v9`: `"gnark_version": "v0.9.0"`

If an item does not have a curve suffix, it is `bn254`.