---
id: SR-546
title: "Code and Rust review of the output-mapping foundation"
type: SpecReview
analysis: code-review
scope: "quire-contract-ir#95; PLAN-008; FR-032 through FR-034; TC-043; target-neutral output-mapping foundation"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-008
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-043
    type: reviews
---

## Summary

The review evaluated the complete issue #95 diff against accepted QSpec FS06
authority, PLAN-008, FR-032 through FR-034, TC-043, the repository's cycle-free
model boundary and Rust conventions. The candidate supplies only the common
request, mapper/record and atomic-package substrate; it adds no target-specific
correspondence, parser, evaluator, runtime or preservation default.

## Verdict

**PASS** — all Rust, identity-preimage, validation, traceability and public-API
findings were repaired; no actionable scoped finding remains.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-9501 | high | **FIXED:** the initial package value and JCS preimage redundantly included request-only native/model/semantic selections outside FR-299's closed member/preimage list. Those fields were removed; a real source-package mutation now qualifies the source-reference identity axis. | `crates/quire-contract-model/src/output_mapping.rs`; FR-034; QSpec FR-299 | implementation-bug-despite-evidence |
| FND-9502 | medium | **FIXED:** generator versions were bounded strings but were not required to satisfy semantic-version grammar. Generator construction now validates the bounded SemVer core/prerelease/build form locally and rejects invalid versions before assembly without adding a model dependency. | `OutputGeneratorIdentity::new`; FR-034 | implementation-bug-despite-evidence |
| FND-9503 | low | **FIXED:** deterministic allocation failures reported one generic path instead of the affected request/mapping/package field. Every allocation point now maps exhaustively to its stable field path. | `MappingAllocationPoint::path`; FR-032 | implementation-bug-despite-evidence |
| FND-9504 | low | **FIXED:** newly introduced wide constructors carried unexplained Clippy exceptions, and downstream observer accessors were not all exercised. The exceptions now state their indivisible requirement axes and TC-043 asserts the complete observer/package read-only surface. | `MappingCandidate::new`; `StructuralObserverIdentity::new`; TC-043 | implementation-bug-despite-evidence |

## Rust Review

- The public seam is target-neutral and cycle-free: strict request admission,
  one bounded `OutputMapper` call per obligation, constructor-private completed
  mappings, atomic package assembly and downstream structural evidence.
- Stable error codes, separately typed identity/digest domains and exhaustive
  closed-enum matches prevent message parsing, cross-domain substitution and
  implicit target fallback. Operational mapper failure is distinct from
  semantic unrepresented/refused records.
- All request/node/depth/work/record/byte arithmetic is checked or bounded
  before exposure. Allocation and monotonic cancellation are checked at request,
  mapping and assembly boundaries, and no partial records or package escape.
- Production changes contain no `unsafe`, panic/unwrap/expect path, unchecked
  integer cast, recursive untrusted traversal, TODO/stub, async/blocking bridge,
  filesystem, clock, locale, process, network or foreign-runtime dependency.
- TC-043 uses the public API, real strict `BoundPackage` admission and an injected
  mapper trait seam. It binds all 14 FR-032 through FR-034 acceptance criteria
  with repository-native `#[trace]` attributes.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| focused locked TC-043 and model-overflow tests | pass |
| locked workspace/all-target tests | pass |
| warning-denied workspace/all-target Clippy | pass |
| model-only dependency graph inspection | pass; no QSL, observation, protocol, TL, target mapper or runtime owner dependency |
| `cargo deny check --disable-fetch` | pass; advisories, bans, licenses and sources clean |
| unsafe-comment audit and changed-source panic/stub scan | pass |
| Quire 0.32.0 strict coverage | pass for scope; FR-032 4/4, FR-033 5/5, FR-034 5/5 |
| repository matrix-status census | pass; every implemented row resolves to a declared test symbol |

Full strict document validation continues to report only the repository's
pre-existing TestMatrix `Status` versus installed archetype `Coverage Status`
header contradiction recorded in SR-544; no issue #95 document has a grammar
finding.
