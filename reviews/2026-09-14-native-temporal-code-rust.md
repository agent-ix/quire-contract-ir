---
id: SR-536
title: "Code and Rust review of native temporal TL correspondence"
type: SpecReview
analysis: code-review
scope: "quire-contract-ir#71; PLAN-006; FR-026; TC-039; temporal projection, sibling request, result join and strict-reader subsystems"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-039
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/PLAN-006
    type: reviews
---

## Summary

The review evaluated implementation candidate `321b283` against temporal spec
authority `980fcc0` and predicate bridge base `202210c`. The candidate
implements the complete FR-026 owner bridge: occurrence-preserving temporal
formula construction, complete predicate valuations, independently constructed
QSL-native and tl-mltl requests, structural result agreement, correction
lineage, and bounded strict readers. Contract IR adds no parser, evaluator,
Boolean coercion, mirrored owner wire vocabulary, callback or qualification
framework.

## Verdict

**PASS** — every code, Rust, identity, lineage, resource, architecture and test
finding found during review was repaired. No actionable scoped finding remains.

## Assurance Context

`AP-001` (`spec/assurance/AP-001-contract-ir-v01.md`) applies because this v0.1
candidate can affect semantic drift, false coverage and canonical identity. The
review used exact base `202210c`, spec authority `980fcc0`, candidate `321b283`,
the complete diff, resolved Cargo graph, AD-001, FR-026, TM-002 and TC-039.
Available context also includes CAC-001, MP-001 and AA-001. Hosted
cross-platform execution, a human source-release decision, pre-stable retained
evidence and a selected release candidate are unavailable and are not inferred.
AP-001 declares no active exception; the package remains unpublished and this
review makes no accreditation or release decision.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6490 | high | **FIXED:** correspondence identity omitted the activation-guard decision, permitting distinct per-activation semantics to share one identity. A typed absent/present guard identity is now canonicalized, and true/false guards produce distinct identities and owner outcomes. | `src/temporal/correspondence.rs`; TC-039 |
| FND-6491 | high | **FIXED:** corrected/invalidating joins checked owner predecessors but did not bind the prior join to the current correspondence. Direct lineage now also requires the exact current correspondence identity. | `src/temporal/join.rs`; FR-026-AC-5 |
| FND-6492 | medium | **FIXED:** valuation cells entered correspondence identity in input order even though each row is a set. Cells are sorted by position and predicate identity, with a reversed-order differential proving stable identity. | `src/temporal/correspondence.rs`; TC-039 |
| FND-6493 | medium | **FIXED:** formula-row resource evidence counted every projected predicate, including the separate activation guard, instead of the temporal subject's formula leaves. Population accounting now uses exact subject leaves. | `src/temporal/request.rs`; FR-026-AC-8 |
| FND-6494 | medium | **FIXED:** initial TC-039 coverage did not execute every supported Boolean, future and past operator through both independent owners. The differential now covers not/and/or/implies, eventually/always/until/release, and once/historically/since/triggered. | `tests/temporal_projection.rs`; FR-026-AC-3/5 |
| FND-6495 | low | **FIXED:** guard refusal coverage did not distinguish a missing guard from illegally reusing a formula leaf as the guard. Both invalid owner shapes now fail closed. | `tests/temporal_projection.rs`; FR-026-AC-1/8 |
| FND-6496 | low | **FIXED:** the correspondence constructor carried an over-wide argument list behind a Clippy exception. The inputs are now grouped in a private `ProjectionParts` value and the production exception is removed. | `src/temporal/correspondence.rs` |
| FND-6497 | low | **FIXED:** architecture and matrix prose still described the temporal bridge as planned/model-only after implementation. AD-001, both matrices and the spec index now describe the implemented root owner bridge while preserving the cycle-free model boundary. | `spec/assurance/AD-001-contract-ir-architecture.md`; `spec/test-matrix.md` |

## Rust Review

- The public surface is a bounded facade: `temporal::{project,
  read_projection, join, read_join}` plus typed selections, inputs, decisions,
  identities and accessors; construction details remain private modules.
- Every untrusted byte reader rejects unknown/duplicate fields, trailing data,
  noncanonical bytes, wrong profiles/identities and configured byte, depth,
  population and work excess before exposing a validated view.
- Production temporal source has no `unsafe`, panic/unwrap/expect path,
  unchecked integer cast, TODO/stub, async/blocking bridge, lock, process,
  network or ambient-state dependency.
- Deterministic collections and explicit sorting govern set semantics. Stable
  domain-separated digests cover the full correspondence, result and lineage
  inputs without treating distinct owner identity domains as interchangeable.
- Tests use real constructor-private owner APIs and bare TC-039 trace tags.
  QSL-native and tl-mltl requests are built, strict-read and evaluated
  independently; Contract IR receives only validated owner result views.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| locked workspace/all-target tests | pass; 77 Rust tests |
| all-feature workspace/all-target Clippy with `-D warnings` | pass |
| release workspace build | pass |
| warning-denied public documentation | pass |
| `cargo deny check` | pass; advisories, bans, licenses and sources clean |
| `cargo audit` | pass; 135 dependencies scanned |
| unsafe/stub/panic-path source audit | pass |
| Quire 0.32.0 FR-026 coverage | pass; 8/8 criteria, 10 TC-039 symbols |

Repository-wide coverage is 130/144. Its seven reported unbacked rows are the
explicitly planned FR-027/TC-040 model-export track; there is no FR-026 status
lie, unbacked criterion or untracked TC-039 symbol.
