# Quire Contract IR

Versioned semantic contract model and canonical representation for assurance tooling.

## Build

```bash
make test
```

## Development status

This crate is being developed spec-first. Its public API is not stable yet, and
registry publication is disabled until the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

The canonical cross-repository governance contract is
[`PGM-01`](spec/program/PGM-01-governance.md). It defines compatibility,
provenance, evidence identity, release ordering, and the qualification boundary
for the eight repositories in the contract-derived verification program.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
