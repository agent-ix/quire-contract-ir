---
id: SR-16625
title: "Code review — IR-645 Kani outcome constructor and wire reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-645; src/kani/outcome.rs, tests/it/kani_*.rs"
review_set: subset
---

## Summary

Diff-scoped Rust review of PR #554, including the constructor, deserializer, accessors, tests, and adjacent IR consumers. The kind/cause/count invariants are enforced in the implementation; the privacy compile-fail probe can pass for the wrong compiler error.

## Verdict

**CONDITIONAL** — one medium test defect requires a same-PR repair before merge.

## Scope

- FR-030-AC-4 (examined): a three-check proof retains count three; zero-check classification is inconclusive without count or Boolean; a zero-count direct proof request fails.
- FR-030-AC-5 (examined): fixed unavailable/inconclusive causes, Boolean rejection through broad constructor, and private fields.
- FR-030-AC-6 (context only): typed code and zero-count error code.
- TC-223 (examined): positive, negative, wire, exhaustive-kind, and compile-fail controls.
- Rust implementation (examined): src/kani/outcome.rs and changed kani tests; no unchecked struct construction was found in adjacent IR source.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Struct-literal compile-fail probe succeeds for an unrelated type error, so it does not prove field privacy | src/kani/outcome.rs:155 |

## Evidence

The probe assigns `success_checks: Some(0)` but the field type is `Option<NonZeroUsize>`, so an E0308 mismatch remains even if all fields become public. The `compile_fail,E0451` fence does not require E0451: a minimal rustdoc control with only E0308 passed under that annotation. Use a well-typed nonzero count in the probe so field visibility is the cause of failure.

Focused checks reported on the PR: 7 kani_outcome, 12 kani_shared, 4 TC-223 tests and 33 doctests passed; Clippy, fmt and diff check passed. I did not rerun the aggregate gate; the author holds it for the reviewed final head.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, the probe uses `Some(NonZeroUsize::MIN)` and every other field value is type-correct; a struct literal from outside `kani` now fails on field visibility rather than the former E0308 mismatch. |

## Disposition Evidence

The changed probe imports `std::num::NonZeroUsize` and sets `success_checks: Some(NonZeroUsize::MIN)`. The only remaining compile barrier in this exact example is the private fields; the source diff changes no production body.
