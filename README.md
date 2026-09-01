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
domain-result provenance, shared assurance ownership, release ordering, and the
qualification boundary for the eight repositories in the contract-derived
verification program. The reviewed
[shared-assurance reconciliation](docs/shared-assurance-governance.md) records
the campaign issue and legacy-prototype dispositions.

Domain tools and project-native systems execute verification and own their
structured results. Quire exports static definitions, Quoin consumes explicit
results for retention/audit/reporting, and ix-flow records attributed human
decisions. Quire and Quoin are non-executing, and neither is a runtime
dependency of this crate.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
