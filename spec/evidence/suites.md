---
id: SUR-001
title: "Contract IR v0.1 evidence suite registry"
type: SuiteRegistry
---

# Contract IR v0.1 evidence suite registry

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-001 | Versioned contract conformance corpus | `cargo run --quiet --bin quire-contract-conformance -- run --manifest corpus/contract-v0.1/manifest.json` | quire-contract-ir | Integration |
| SUITE-002 | Strict specification validation | `quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --strict --summary` | quire | Analysis |
| SUITE-003 | Static specification and coverage export | `quire coverage --scope . --json` | quire | Static |
| SUITE-005 | Native predicate/TL projection properties | `cargo test --test native_predicate_projection --all-features` | planned Rust property and real tl-syntax contract harness | Property |
| SUITE-006 | Native predicate canonical identity goldens | `cargo test --test native_predicate_projection canonical_identity_goldens --all-features` | planned independently authored preimage/digest oracle | Snapshot |
| SUITE-007 | Native predicate real-reader join | `cargo test --test native_predicate_projection real_tl_syntax_reader_join --all-features` | planned integration against exact accepted public tl-syntax readers | Integration |
| SUITE-008 | Native predicate deterministic allocation faults | `cargo test --test native_predicate_projection allocation_faults --all-features` | planned failpoint-backed no-partial-output checks | Integration |

## Notes

SUITE-001 is the native structured producer. Each result carries the sorted criterion IDs
validated against the fixture's observed coverage tokens (FR-018). Quire owns
criterion identity and static relationships; a matching fixture is neither
complete criterion verification nor a release decision.

SUITE-002 and SUITE-003 are static Quire reads. `make ci` is native orchestration, not an evidence suite or trust root.
SUITE-005 through SUITE-008 are planned identities only and are not current
evidence. They separate generative semantic properties, an independently
written canonical-byte oracle, public-reader integration, and deterministic
resource-failure injection. They remain
blocked until accepted native checked-leaf, source-result/mapping and
result-availability authority contracts plus a compatible released tl-syntax
catalog schema/reader are selected; a mock or copied schema cannot make it
available.
