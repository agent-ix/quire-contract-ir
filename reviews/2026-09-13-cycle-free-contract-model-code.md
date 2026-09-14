---
id: SR-534
title: "Code and Rust review of the cycle-free Contract Model"
type: SpecReview
analysis: code-review
scope: "quire-contract-ir#73; FR-028; TC-041; Cargo workspace, model source ownership, compatibility bridge, locked QSL composition fixture, CI/Make gates"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-041
    type: reviews
---

## Summary

The review evaluated the complete cycle-free package split and compatibility
boundary at `a07ed49a6ecac5275fe223ae8aa18de40c4d23b3` against
`58f834be44927e31755b3abf797a35fb9cace507`. The final implementation retains
one semantic source, keeps owners outside the model and production bridge
closure, and proves the exact real-QSL composition with a clean-run-reproducible
locked fixture.

## Verdict

**PASS** — all code, Rust, architecture, traceability and dependency-policy
findings discovered in review were repaired; no actionable scoped finding
remains.

## Assurance Context

`AP-001` (`spec/assurance/AP-001-contract-ir-v01.md`) applies to this identified
v0.1 source candidate because the split can affect semantic-drift,
false-coverage and canonical-identity controls. The review evaluated candidate
`a07ed49a6ecac5275fe223ae8aa18de40c4d23b3`, comparison base
`58f834be44927e31755b3abf797a35fb9cace507`, every changed Cargo/Make/CI path,
the moved model sources, compatibility re-export, TC-041 fixtures and the
FR-028 matrix rows. `AD-001`, `CAC-001`, `MP-001` and active `AA-001` were
available, but the first three remain proposed and `AA-001` retains an open
human source-release claim against an earlier baseline. Pre-stable retained
evidence, hosted CI, cross-platform comparison and a human sufficiency decision
are unavailable by declared policy; none is inferred. The installed Quire
0.32.0 is newer than the separate shared-assurance matrix pin of 0.31.0, so
that qualification classification remains `unknown` and was not changed by
this implementation review. AP-001 declares no active exception; AA-001's
downstream-independence and toolchain-reproduction assumptions remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings remain after remediation. | - |

## Remediation completed

- Replaced the incomplete QSL-alias-only proof with a locked build importing
  the real checked-predicate and temporal-subject owner modules beside the
  compatibility bridge.
- Proved QSL is dev-only and absent from the root package's transitive normal
  dependency closure; the model remains owner/TL-free.
- Aligned the standalone fixture lock to workspace transitive versions so a
  clean runner's workspace fetch is sufficient for its offline build.
- Added exact source-policy allow-list entries and an explicit QSL package
  version without weakening unknown-source denial.
- Corrected stale matrix states/headers and the Python orchestration assertion
  that still expected root-only Cargo gates.

## Rust review

- The model package contains the single existing semantic implementation; the
  compatibility crate's re-export-only shape is required by FR-028 rather than
  a placeholder.
- No owner vocabulary, parser, evaluator, Boolean coercion, callback, trust
  flag, owner wire mirror, unsafe block, async path or new public panic surface
  was introduced.
- The new tests carry TC-041/FR-028 traces, exercise real Cargo resolution and
  compilation, use immutable revisions, and fail on missing packages, source
  drift, production-edge leakage or build failure.
- The CI and Makefile changes widen all Cargo gates to the workspace; no lane,
  target, warning policy or safety check was removed.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| workspace/all-target/all-feature Clippy with `-D warnings` | pass |
| workspace/all-target Rust suite | pass; 51 tests |
| TC-041 locked offline composition suite | pass; 4 tests |
| release workspace build | pass |
| native conformance corpus | pass |
| corpus regeneration/reproducibility | pass |
| `cargo deny check` | pass; only inherited unmatched-allow warnings |
| `cargo audit` | pass; 131 locked dependencies scanned |
| unsafe-comment audit | pass |
| native orchestration controls | pass; 3 tests |
| changed FR-028/TestMatrix structural validation | pass with Quire 0.32.0 |

The combined `make test` qualification wrapper additionally ran 20 passing
Python tests and stopped only on its inherited Quire 0.31.0 compatibility pin;
changing that qualification matrix is outside this implementation and was not
used to convert an unknown classification into a pass.
