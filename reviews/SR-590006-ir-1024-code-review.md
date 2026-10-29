---
id: SR-590006
title: "IR-1024 PR 577 Rust code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/conformance_qspec/hybrid.rs; tests/conformance_qspec/timed_model.rs; IR-1024"
review_set: subset
---

## Summary

Reviewed the frozen Rust test diff in PR 577 against the repository's Rust conventions and QSpec fixture contract. The changed assertions exercise the public reader and distinguish exact code, cause and locus. Rust-review is included in this code-review method.

## Verdict

**PASS** for the changed Rust test code. The focused conformance target, rustfmt, Clippy and diff check were reported green at the frozen head; the reviewer did not repeat or run an aggregate gate. Acceptance coverage is recorded separately in SR-590007.

## Scope

- IR-1024 AC 2 (examined): Repair owned mismatches in the authoritative repository, preserving strict IR conformance assertions and reading QSpec fixtures from the QSpec checkout only; no fixture copies or weaker expectations.
- IR-1024 AC 3 (examined): Run the focused QSpec conformance target against the corrected pair and record all results.
- FR-038-AC-316 (context only): Hybrid and protocol refusal order.
- FR-038-AC-318 (context only): Timed-model published fixture admission and complete body round-trip.
- Files examined: `tests/conformance_qspec/hybrid.rs`, `tests/conformance_qspec/timed_model.rs`; QSpec `positive-all-families.json` at the selected fixture revision.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

- The early protocol-reference refusal remains exact and loses its incorrect FR-038-AC-316 trace.
- The new protocol-law case refuses with exact `InvalidPackage/OperationLawMismatch` and locus; the paired hybrid model defect refuses with exact `InvalidPackage/InvalidValue` and locus. Both assertions can fail against changed reader behavior.
- Timed-model fixture count changes 54 to 56 while admission, retained bodies, writer round-trip and package identity assertions remain.
- Focused author evidence: 68 passed, 0 failed, 1 ignored; rustfmt, Clippy and diff check passed.
