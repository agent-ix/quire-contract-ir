# Quire Contract IR

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Cycle-free semantic contract model and compatibility bridge for assurance tooling.

## Workspace architecture

This repository contains two Rust packages with one-way production dependencies:

- `quire-contract-model` owns the stable semantic model, canonicalization,
  diagnostics, wire checks, limits, bindings, and conformance behavior. Owner
  crates depend on this package without depending on the Contract IR bridge.
- `quire-contract-ir` preserves the existing crate name and publicly re-exports
  the complete model API. Native owner integrations are added only to this
  downstream bridge package.

Cargo dependency aliases let existing `quire_contract_ir` source imports select
the `quire-contract-model` package where an owner crate needs the cycle-free
substrate. The package split changes no schema, canonical byte, identity,
diagnostic, or conformance outcome.

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
release ordering, and the qualification boundary for the repositories in the
contract-derived verification program.

Domain tools and project-native systems execute verification and own their
structured results. Quire exports static definitions, Quoin consumes explicit
results for retention/audit/reporting, and ix-flow records attributed human
decisions. Quire and Quoin are non-executing, and neither is a runtime
dependency of this crate.

## Retained evidence

This repository held ten immutable PGM-01 records under `evidence/`, read
through Engineering Assurance's read-only compatibility mapping. It was the only
repository in the eight-repository campaign for which that mapping worked: all
ten mapped `lossy` with their source digests preserved and no byte moved. The
repository owner released the evidence-preservation constraint for the
pre-stable phase on 2026-09-02
([engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7)),
and the records, their reader, and the schemas frozen only for their sake are
deleted. Nothing was rewritten on the way out and no claim here rests on them.
The same decision withdraws PGM-01-R08, so the derivation-evidence envelope
schema, its Draft 7 validator, its fixture corpus and its pinned Python lane are
deleted too. `schemas/README.md` explains which schemas are live and why.
The constraint re-applies at the move toward stable releases.

## License

Licensed under the GNU Affero General Public License, version 3 or (at your
option) any later version (`AGPL-3.0-or-later`). See [LICENSE](LICENSE).
