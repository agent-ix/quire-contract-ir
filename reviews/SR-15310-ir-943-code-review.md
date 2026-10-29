---
id: SR-15310
title: "IR-943 PR 502 dependency signature reader code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/dependency_references.rs; tests/it/checked_package_v2_dependency_reference.rs; tests/it/checked_package_v2_reader.rs; QSpec FR-322-AC-84..87"
review_set: subset
---

## Summary

Ticket: IR-943. Reviewed the three-file PR diff at the reviewed head with the Rust review lane. The signature walk takes the function result and the semantic types of the parameter nodes in the first `parameters` binding, then traverses only type dependencies. The lock-order, refusal-locus, and signature tests exercise the intended reader behavior. FR-322-AC-87's compiler derivation is outside this reader code review and is addressed in the separate gap review.

## Verdict

**PASS** — no Rust implementation or code-test alignment defect found in the changed reader slice.

## Coverage

- Examined QSpec FR-322-AC-84: first failing dependency selection in lock order, presence before binding.
- Examined QSpec FR-322-AC-85: missing-selection, missing-name, and operator-ineligible refusal loci.
- Examined QSpec FR-322-AC-86: only parameter and result semantic types enter the signature closure.
- Context only: QSpec FR-322-AC-87; the changed IR test covers reader preservation of supplied occurrence data.
- Rust review: repository idioms, checked decoding, work-meter charging, panic/unsafe surface, bounded iteration, fixture semantics, and `#[trace]` bindings. No changed production path introduces `unwrap`, `expect`, unsafe, recursion, or an unbounded allocation beyond existing graph-sized worklists.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Follow-up review

**PASS.** The production reader is byte-identical to the prior reviewed head. The added commit only changes test trace tags from external QSpec IDs to IR's FR-038-AC-4/37/38 and updates the companion FR-038 description. The FR-038-AC-4 test asserts package-id stability under a changed source-map region, the AC-37 tests assert the specified missing/stale dependency and refusal loci, and the AC-38 tests assert its named adverse and positive signatures. No new Rust or code-test finding.
