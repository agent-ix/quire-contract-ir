---
id: SUR-001
title: "Contract IR v0.1 evidence suite registry"
type: SuiteRegistry
---

# Contract IR v0.1 evidence suite registry

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-001 | Versioned contract conformance corpus | `cargo run --quiet --bin quire-contract-conformance -- run --manifest corpus/contract-v0.1/manifest.json` | quire-contract-ir 0.1.0 | Integration |
| SUITE-002 | Strict specification validation | `quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary` | quire 0.31.0 / quire-rs 0.46.0 | Analysis |
| SUITE-003 | Static specification and coverage export | `quire coverage --scope . --json` | quire 0.31.0 / quire-rs 0.46.0 | Static |
| SUITE-004 | Shared assurance intake chain | `python3 scripts/assurance_chain.py --candidate-revision <sha> --conformance target/assurance/conformance.jsonl --quire-export target/assurance/quire-static-export.json` | quoin 0.23.1 change-assurance and evidence surfaces | Integration |

## Notes

SUITE-001 is the native structured producer transcribed through Quoin's
`contract-conformance` adapter. Each result carries the sorted criterion IDs
validated against the fixture's observed coverage tokens (FR-018). Quire owns
criterion identity and static relationships; a matching fixture is neither
complete criterion verification nor a release decision.

SUITE-002 and SUITE-003 are static Quire reads. SUITE-004 consumes already produced bytes and never
executes SUITE-001. `make ci` is native orchestration, not an evidence suite or trust root.
