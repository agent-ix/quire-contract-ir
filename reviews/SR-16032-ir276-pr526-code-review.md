---
id: SR-16032
title: "Code and Rust review — IR-276 counted-walk report"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #526 at c7f7316f2697eb4650b8e2a8998c9614b44be9f3; crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs"
review_set: subset
---

## Summary

Ticket: IR-276. Reviewed the 15 added reporting lines against the existing fixture, counter site, lowering path, and nine BEFORE/AFTER raw test-output cells reproduced in IR-276 comment 7ef9f105. The attached archive itself was unavailable from this host; the inline original stdout, invocation scripts, exit codes, and memory caps were readable. The report measures the fixture's direct retained-fact scan and the subsequent lower call; it does not measure public reader admission.

## Verdict

**CONDITIONAL** — one low-severity output-label finding. The counter and timing code is otherwise consistent with the stated narrow measurement. The historical outputs establish K(R+1) body validations before versus R+K fact-scan validations and zero additional lower validations after, across K=2,4,8 and R=4,8,16. They do not establish total lowering O(K+R); per-request charge and output work remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Rename `admission_body_validations`: this private fixture calls `scan_lowering_facts` directly and never runs public reader admission, so the JSON field falsely attributes its counter to admission when reused without the ticket's explanatory prose. | crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:347 |

## Rust review

The added `Instant` surrounds only `package.lower`; it excludes fixture construction and retained-fact scanning, so the two historical elapsed-time columns are not a total-work comparison. The report executes before the existing batch/single-request parity assertions, which still run for every cell. The `BODY_WALKS` counter is incremented by the real `validate_body` function in test builds, and subtraction is safe in this single-thread-local fixture because the counter is reset before the scan and only increments. No production code, dependency, lock, unsafe, panic surface, or lints changed. `git diff --check` passed. No aggregate gate was run by this review.

## Scope

Examined FR-038-AC-308 and FR-038-AC-309, with IR-276 AC-1, AC-2 and AC-6 as ticket context. The changed test file is the entire PR diff. The raw nine-cell stdout is present in IR-276 comment 7ef9f105; the attached archive was not independently accessible.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At exact PR head `2fb983ad8c8e95a7f21d6f81a9a7e6f4add67b72`, the JSON key is `"lowering_fact_scan_body_validations": admission_walks,` and accurately names the direct fixture scan. The other reporting fields and existing assertions are unchanged. |
