---
id: ADR-0055
title: "Rust 1.98.1 qualification and compatibility baseline"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: depends_on
---
# ADR-0055: Rust 1.98.1 compiler baseline

## Status

Accepted.

## Decision

Rust 1.98.1 is both the supported minimum and the compiler the repository
builds and tests with. `Cargo.toml` (`rust-version`), `rust-toolchain.toml` and
`clippy.toml` name it, and the `supported-rust` and `qualification-rust` Make
targets compile and test the workspace with it.

Rustfmt output from 1.98.1 is the repository's formatting result. A newly
reported Clippy diagnostic or formatting change is remediated, not treated as a
compiler incompatibility.

## Consequences

The inherited 1.75 lane is removed. Direct consumers make their own
compatibility decision; Contract IR does not promise an older floor for them.

## Rejected alternatives

- **Keep 1.75 because it is already declared.** No decision evidence exists.
- **Use 1.85 because Filament declares it.** That value is also stale and its
  existence is not a Contract IR requirement.
- **Use floating `stable`.** It is not reproducible.
- **Treat rustfmt or Clippy drift as incompatibility.** Both are expected inputs
  to migration and remediation.
