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

## Build

```bash
make test
```

## Development status

This crate is being developed spec-first. Its public API is not stable yet.

## License

Licensed under the GNU Affero General Public License, version 3 or (at your
option) any later version (`AGPL-3.0-or-later`). See [LICENSE](LICENSE).
