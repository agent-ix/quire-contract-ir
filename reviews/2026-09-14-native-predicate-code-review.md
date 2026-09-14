---
id: SR-061
title: "Code and Rust review of native predicate TL projection"
type: SpecReview
analysis: code-review
scope: "quire-contract-ir#70; FR-025; TC-038; bridge and predicate subsystems; immutable owner dependencies; strict readers and owner-view fixtures"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-038
    type: reviews
  - target: ix://agent-ix/tl-syntax/Task-006
    type: reviews
---

## Summary

The review evaluated implementation candidate
`db6bbf551d5e321c8051011d63221e03e9bfaece` against cycle-free Contract IR base
`53cc03c639e2e26528132d34d96dc56449df78e8`. The candidate implements the
complete FR-025 predicate subsystem over real constructor-private QSL, Quire
Observation and Quire Protocol views and real tl-syntax signal-catalog and
proposition-map constructors/readers. It adds no source parser, evaluator,
Contract-IR predicate vocabulary, Boolean coercion or alternate semantic
authority.

## Verdict

**PASS** — all code, Rust, dependency, traceability, identity, resource and
strict-reading findings discovered during review were repaired. No actionable
scoped finding remains.

## Assurance Context

`AP-001` (`spec/assurance/AP-001-contract-ir-v01.md`) applies because this
identified v0.1 candidate can affect semantic drift, false coverage and stable
canonical identity. The review evaluated base
`53cc03c639e2e26528132d34d96dc56449df78e8`, candidate
`db6bbf551d5e321c8051011d63221e03e9bfaece`, every changed production/test/spec
path, the exact Cargo-resolved owner graph, and FR-025/TC-038 traceability.
Available context includes AP-001 and the repository's declared architecture,
canonicalization, diagnostic and test-matrix controls. Hosted cross-platform
execution, a human source-release decision, and pre-stable retained evidence
are unavailable and are not inferred. AP-001 declares no active exception;
the candidate remains unpublished and this review does not make an
accreditation or release decision.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6481 | high | **FIXED:** target admission compared a generic target pair and could accept signal-catalog and proposition-map selections in the wrong slots. Each slot is now compared to its exact owner contract and a traced cross-wire test refuses the swap. | `src/predicate/admission.rs`; TC-038 |
| FND-6482 | high | **FIXED:** initial bounds checked individual documents but did not charge the full input, derived tuple, sibling artifact and decision traversal. Projection and valuation now apply saturating cumulative work checks before and after construction. | `src/predicate/admission.rs`; `src/predicate/valuation.rs`; FR-025-AC-6 |
| FND-6483 | high | **FIXED:** completeness outside the exact deciding set could erase an otherwise final Boolean. Outside-set incomplete/contradicted facts are now retained only as typed gaps; deciding-fact loss remains fail-closed. | `src/predicate/valuation.rs`; FR-025-AC-5 |
| FND-6484 | high | **FIXED:** the existing dependency policy rejected the required immutable AGPL owner graph. The allow-list now names the actual licenses and exact approved Git sources while retaining unknown-source denial; `cargo deny check` is clean. | `deny.toml`; `Cargo.toml` |
| FND-6485 | medium | **FIXED:** public digest parsing returned an unstructured string error. It now uses a typed `BridgeDigestParseError` and `FromStr` without exposing an untrusted-input panic path. | `src/bridge/identity.rs` |
| FND-6486 | medium | **FIXED:** manifest pins and runtime contract-selection strings could drift independently. TC-038/TC-041 now assert exact Cargo-resolved revisions, runtime revisions and owner schema bytes for all four owner repositories. | `tests/predicate_projection.rs`; `tests/predicate_valuation.rs`; `tests/cycle_free_model.rs` |
| FND-6487 | medium | **FIXED:** contract-selection failure classification did not cleanly separate malformed/unavailable, same-identity schema conflict and well-formed unsupported selections. Admission now emits distinct closed causes and withholds authenticated downstream fields. | `src/predicate/admission.rs`; FR-025-AC-4 |
| FND-6488 | low | **FIXED:** warning-denied public documentation linked to a private module constant. The public description now states the owner-maxima rule without a broken intra-doc link; warning-denied rustdoc passes. | `src/bridge/limits.rs` |
| FND-6489 | low | **FIXED:** two inherited TestMatrix documents used the obsolete `Status` header and failed the installed Quire archetype assertion. All four asserted tables now use `Coverage Status`; no row meaning changed. | `spec/contract-test-matrix.md`; `spec/test-matrix.md` |

## Rust review

- The public API is a small facade: `predicate::{project, read_projection,
  value, read_valuation}` plus typed selections, decisions, identities and
  limits. Implementation modules remain private.
- All untrusted bytes enter bounded strict readers with unknown/duplicate-field,
  trailing-data, canonical-byte, depth, string, byte, population, work and
  allocation checks. Operational failures expose no partial artifact or
  Boolean value.
- Production source contains no `unsafe`, panic/unwrap/expect path, unchecked
  integer cast, TODO/stub, async/blocking bridge, mutex or external process,
  network or ambient-state dependency.
- `BTreeMap`/`BTreeSet` and explicit cause ranking provide deterministic order.
  Wire identities use domain-separated SHA-256 and strict lowercase parsing.
- Real owner fixture construction exercises constructor privacy and public
  readers. Tests use bare `#[trace("TC-038", ...)]` tags and cover both Boolean
  values, every owner non-value family, correction binding, cross-wires,
  hostile bytes, deterministic goldens, join disagreement and resource
  failpoints.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| workspace/all-target Clippy with `-D warnings` | pass |
| locked workspace/all-target Rust suite | pass; 67 tests |
| release workspace build | pass |
| warning-denied public documentation | pass |
| `cargo deny check` | pass; advisories, bans, licenses and sources clean |
| `cargo audit` | pass; 134 dependencies scanned |
| unsafe-comment audit | pass |
| Quire 0.32.0 FR-025 coverage | pass; 8/8 criteria, 16 TC-038 symbols, no scoped unbacked row/status lie/untracked symbol |

Repository-wide `make spec` intentionally remains nonzero on the 16 explicitly
planned FR-026/TC-039 and FR-027/TC-040 rows. The installed process module also
reports its known contradiction between the archetype-required `Coverage
Status` header and its classifier-configured `Status` header. The independent
matrix census passes and every implemented row resolves to a completed test;
neither diagnostic is converted into a scoped FR-025 pass.
