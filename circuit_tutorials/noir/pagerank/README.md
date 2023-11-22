# Pagerank Circuit

# TODO: should package.json and package-lock.json be gitignored here?

This circuit accepts a list of edges in a sparse graph with 20 nodes.
It will compute the pagerank of those 20 nodes and return the sorted list of nodes
The most "influential" node in the network is last in the sorted list of `Verifier.toml`.

### Circuit Info

```
+----------+------------------------+--------------+----------------------+
| Package  | Language               | ACIR Opcodes | Backend Circuit Size |
+----------+------------------------+--------------+----------------------+
| pagerank | PLONKCSat { width: 3 } | 344141       | 349595               |
+----------+------------------------+--------------+----------------------+
```

### Instructions
Refer to the [Sindri's Noir tutorial](https://sindri-labs.github.io/docs/forge/api-tutorials/noir/) for more information on this circuit and instructions for running the `compile.mjs` script.