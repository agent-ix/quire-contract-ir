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
[shared-assurance reconciliation](spec/program/STD-002-shared-assurance-governance.md) records
the campaign issue and legacy-prototype dispositions.

Domain tools and project-native systems execute verification and own their
structured results. Quire exports static definitions, Quoin consumes explicit
results for retention/audit/reporting, and ix-flow records attributed human
decisions. Quire and Quoin are non-executing, and neither is a runtime
dependency of this crate.

## Shared assurance

```bash
make assurance-inputs   # the native producers run here, and only here
make assurance          # pins, legacy compatibility, and the Quoin chain
```

`make assurance-inputs` runs the contract conformance runner and `quire
coverage`. Everything after it consumes files that already exist. The shared
components are reached at the accepted released pins, classified against
Engineering Assurance's own compatibility matrix rather than against a second
copy of it kept here; `assurance/pins.json` records which release is adopted and
the digests of the artifacts read from it.

The ten immutable records under `evidence/` are history. They are read through
Engineering Assurance's read-only compatibility mapping, which reports `lossy`,
`incompatible`, or `unreadable` rather than synthesizing a field a legacy record
never carried, and they are never written to again. `schemas/README.md` explains
which schemas are live, which are historical, and which are frozen.

The assurance lane installs into its own interpreter (`make assurance-env`,
`.venv-assurance`). It is separate from the PGM-01 Draft 7 governance lane on
purpose: that lane pins `jsonschema==3.2.0` and Engineering Assurance declares
4.x, and neither is bent to fit the other.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
