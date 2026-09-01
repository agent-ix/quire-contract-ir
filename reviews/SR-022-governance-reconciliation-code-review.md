---
id: SR-022
title: "Code review — PR #41 shared assurance governance reconciliation"
type: SpecReview
analysis: code-review
scope: "tests/governance_reconciliation.rs, tests/fixtures/historical-pgm01-files.sha256, docs/shared-assurance-governance.md, spec/program/PGM-01-governance.md, README.md, CONTRIBUTING.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/38
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-021
    type: references
---
# SR-022: Code review — PR #41 shared assurance governance reconciliation

## Summary

Review of `issue/38-governance` at `14960ae03e3886d72983485ee0422542087b49a7`
(989 additions, 57 deletions, 23 files). Gates were run, not assumed: `make ci`
exits 0 with 5/5 new governance tests passing; `make spec` exits 1 on two
unbacked rows. The reconciliation itself is sound — FR-021-AC-5 was checked
independently of its own test and holds, and the 46-file byte lock covers every
tracked historical PGM-01 record, checksum, correction and v1 schema. The
findings below are where the new gates assert less than they appear to.

## Verdict

**FAIL** — `make spec` is red, and PGM-01-R11 asserts external issue state that
is not yet true. Both are the merge conditions the PR body already names; this
review confirms them rather than disputing them.

## Assurance Context

AP-001 at `spec/assurance/AP-001-contract-ir-v01.md` applies because the
governance delta affects the v0.1 candidate's semantic-drift and false-coverage
controls. The evaluated baseline is `bb5d30c..14960ae`; changed paths are the
scope named in frontmatter. AD-001, CAC-001, MP-001, and AA-001 were available
and were checked for semantic-library isolation, deterministic gate use, and
the still-open human source-release claim. No semantic Rust library behavior
changed.

Quoin machine-readable assurance context, a current candidate evidence record,
cross-platform results, and an independent human sufficiency decision were
unavailable. Hosted CI remained intentionally undispatched. Active exceptions:
none. AP-001 and its linked assurance artifacts remain `proposed`; AA-001's
source-release claim remains open.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-217 | high | `make spec` exits 1 (`TC-026` and `FR-021-AC-3` unbacked under `--strict`), so `make release-check` is red on this branch. `make ci` does not include `make spec`, so the green CI run does not cover traceability. | Makefile:69 |
| FND-218 | high | PGM-01-R11 states as normative policy that issue #20 "is superseded" and #7 "is re-scoped", but #1, #7 and #20 are all still OPEN and unedited (#7 last updated 2026-08-30). Merging before the dispositions land publishes a policy claim about external state that is false. | spec/program/PGM-01-governance.md:245 |
| FND-219 | medium | Two of the three obsolete-text guards in `tc_028` assert strings that never existed in `README.md` or `CONTRIBUTING.md` on `main` — `git grep -F` against `origin/main` finds "Quoin and Quire may integrate later" and "This program does not refactor Quoin, Quire-rs" in neither file. Those assertions cannot go red in either direction; only the third guard corresponds to a real removal. | tests/governance_reconciliation.rs:205 |
| FND-220 | medium | `tc_028` scans only `README.md` and `CONTRIBUTING.md`, but FR-021-AC-5 says "no campaign document". `spec/`, `docs/`, `plan/` and `reviews/` are unscanned. The criterion does hold today — verified independently — but nothing gates it. | tests/governance_reconciliation.rs:198 |
| FND-221 | medium | The runtime-independence guard in `tc_025` matches the literal `package = "quoin"` with exact spacing. Cargo accepts `package="quoin"`, so `assurance = { version = "1", package="quoin" }` passes the guard with a renamed Quoin runtime dependency in `[dependencies]`. | tests/governance_reconciliation.rs:140 |
| FND-222 | medium | `tc_025` inspects only the first `[dependencies]` block of the root manifest. `[build-dependencies]`, `[dependencies.quoin]` table syntax appearing before the `[dependencies]` header, and any future workspace member are outside the guard. | tests/governance_reconciliation.rs:128 |
| FND-223 | medium | `tc_024` is an allow-list plus a count. It digests the 46 enumerated paths and asserts `locked.len() == 46`, but never walks `evidence/` to confirm the enumeration is complete, so a newly added file under `evidence/` is invisible to the lock. | tests/governance_reconciliation.rs:105 |
| FND-224 | medium | The reconciliation record claims the fixture "locks every historical record, checksum, correction, and schema byte present when this reconciliation was accepted". Four tracked schema files — `contract-conformance-manifest-v1.schema.json`, `contract-package-reference-v1.schema.json` and their `.sha256` companions — are outside the lock. Narrow the claim to the historical PGM-01 schemas or extend the lock. | docs/shared-assurance-governance.md:31 |
| FND-225 | medium | `docs/shared-assurance-governance.md` is untyped and outside every `quire validate` glob (`spec/**`, `plan/**`, `reviews/**`), yet FR-021 names it as an Output and three tests assert its exact contents. It carries normative-adjacent content with no schema, no frontmatter and no validation. | docs/shared-assurance-governance.md:1 |
| FND-226 | medium | FR-008-AC-4 and FR-009-AC-6 declare verification method `Test`, but their oracle is a substring assertion over the same document that states the property — `tc_025` establishes "Quoin remains non-executing" by finding the sentence "Quire and Quoin are explicitly non-executing" in PGM-01. The repository's own `verification_catalog` separates `inspection` from the test classes; these rows are inspection, and TC-023/TC-025 are already typed `Static` in the matrix. | spec/functional/FR-009-evidence-retention.md:63 |
| FND-227 | low | Each of the eight dimension reviews (SR-014…SR-021) records exactly two findings, every one prefixed "Resolved:" and already fixed in the same commit, and all eight verdicts are PASS. They document design rationale well but record no open risk, so they read as rationale rather than as a gate. | reviews/SR-018-governance-reconciliation-failure-domain.md:24 |
| FND-228 | low | `spec/functional/FR-008-evidence-envelope.md` keeps a filename asserting the concept the change removes; its title is now "Validate domain derivation provenance" and its body states the record "is not a common evidence envelope". | spec/functional/FR-008-evidence-envelope.md:1 |

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass |
| `make test` (governance + `cargo test -- --include-ignored`) | pass |
| `tests/governance_reconciliation.rs` | 5/5 pass |
| `make corpus`, `make corpus-repro` | pass |
| `cargo deny check licenses` | pass |
| `scripts/check_unsafe_comments.sh` | pass |
| `make ci` | exit 0 |
| `quire validate` | 81/81 docs grammar-clean |
| `make spec` | **exit 1** — 2 unbacked rows |

## What was checked independently

FR-021-AC-5 was verified without its test: a tree-wide search for "common
evidence envelope", "universal runner" and "evidence store" across `spec/`,
`plan/`, `docs/`, `reviews/`, `README.md` and `CONTRIBUTING.md` returns only
prohibitions, no prescriptions. The criterion holds.

The byte lock was reconciled against `git ls-files evidence schemas`: 50 tracked
files, 46 locked, and the four unlocked files are the Wave 1 contract
conformance schemas rather than historical PGM-01 inputs. The 46-file claim is
accurate.

TC-026's closure path is sound. `quire coverage --strict` reports that
`Inspection` mints no source symbol, but the repository already backs an
Inspection row the same way TC-026 will need to be backed — `TC-004` is typed
`Inspection` and carries `/// Tracing: TC-004` on a Rust test in
`tests/governance.rs:148`.
