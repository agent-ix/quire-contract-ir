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
| SUITE-002 | Strict specification validation | `quire validate --scope . 'spec/**/*.md' 'planning/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary` | quire 0.31.0 / quire-rs 0.46.0 | Analysis |
| SUITE-003 | Static specification and coverage export | `quire coverage --scope . --json` | quire 0.31.0 / quire-rs 0.46.0 | Static |
| SUITE-004 | Shared assurance intake chain | `python3 scripts/assurance_chain.py --candidate-revision <sha>` | quoin 0.23.1 change-assurance and evidence surfaces | Integration |

## Notes

SUITE-001 is the native structured producer transcribed through Quoin's
`contract-conformance` adapter. Each result now carries the sorted Test Case ids declared by its
manifest fixture. Quire owns the Test Case-to-acceptance-criterion relationship; the corpus does not
copy that graph or turn a matching fixture into a release decision.

SUITE-002 and SUITE-003 are static Quire reads. SUITE-004 consumes already produced bytes and never
executes SUITE-001. `make ci` is native orchestration, not an evidence suite or trust root.
