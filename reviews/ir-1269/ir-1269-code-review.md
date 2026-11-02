---
id: SR-2026110101298
title: "IR-1269 PR 829 schedulability prefilter code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #829; crates/quire-contract-model/src/checked_package/v2/schedulability.rs; tests/it/checked_package_v2_schedulability.rs"
review_set: subset
---

## Summary

Ticket: IR-1269. Reviewed frozen head ec9b2e1f against base 51971194. Applied code-review and its Rust-review lane, Rust style, repository guidance, the changed reader path, and the public reader test. No applicable AssuranceProfile was found under spec/.

## Verdict

**PASS** — filtering claim positions before the structural-key sort preserves the sorted order of every position that reaches body validation. The changed code does not add or remove a meter charge or alter typed refusal construction. The focused test exercises two malformed claim bodies with graph order opposite structural-key order and checks the exact code, cause and locus. No source copy, new API, wire change, or CI gate change appears in the diff. Aggregate gates remain with the code lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-398, role: examined. Excerpt: Wire and graph/aggregate validation precede carrier joins; declaration precedes occurrence, repeated key precedes order, coverage follows all rows, and carrier validation precedes FR-482 body validation. Combined defects distinguish each adjacent check. Each visited row charges one existing validation visit before inspection; an exact measured budget admits the positive package, and one below the measured charge refuses with the existing incomplete/resource-exhaustion result at the first uncharged row and no partial typed claim view.
- FR-038-AC-399, role: examined. Excerpt: The reader admits the full FR-482 schedulable/response bodies and validates its shape, literal vocabulary, form fit, strict task order, common time unit and policy-specific task-set premises in that order after the protocol body step.
- FR-038-AC-380 and FR-038-AC-381, role: context_only. Existing NI-13 long-index chain test is their performance regression control.

## Evidence

The old loop sorted all indexed positions and skipped nonclaims inside the loop. The new loop filters the same positions by the same claim predicate, then sorts by the same structural keys. Filtering before sorting cannot change the relative order of claim positions, and the body of the loop is unchanged. The existing admitted graph index and kinds arrays share node positions. The new test's two invalid claims have ids fixture/schedulability/a and fixture/schedulability/worker, respectively, while a is appended after worker in the graph. It checks InvalidPackage, InvalidValue, one locus, and the appended claim's body member 1. The parent-provided focused evidence reports that this test and the NI-13 100,001-node test pass; scoped Clippy and fmt pass. These test runs were not repeated in this review.
