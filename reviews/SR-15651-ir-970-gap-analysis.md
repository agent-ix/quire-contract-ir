---
id: SR-15651
title: "Gap analysis — IR-970 typed operation and identity evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #516; FR-349-AC-1/2/3/4, FR-038-AC-35/88, changed operations tests and existing integration controls"
review_set: subset
---

## Summary

Compared the changed reader and preimage paths with the owning criteria and their tagged tests. Existing integration controls exercise admitted typed bodies, dependency-reference selection, first refusals, and identity bytes. One new test claims FR-349-AC-4 without exercising malformed-term admission.

## Verdict

**CONDITIONAL** — correct the new test's FR-349-AC-4 trace claim or add a malformed-term reader assertion that backs it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new retained-operation test claims malformed-term refusal coverage but never reads a malformed term | `crates/quire-contract-model/src/checked_package/v2/operations.rs:4677` |

## Coverage

Plan completion: not assessed. Examined FR-349-AC-1/2/3/4 and FR-038-AC-35/88. The new `admitted_operation_identity_comes_from_the_retained_term` test decodes a conforming application body, mutates only its raw `operation.identity`, and calls the private `operation_defect` with the prior retained term. It proves operation dispatch uses retained data, but no malformed or unknown `term` reaches `CheckedPackageV2::read`, and it asserts no first code, cause or JSON pointer. Existing tagged tests in `checked_package_v2_frame_bodies.rs`, `checked_package_v2_abstraction_relation.rs`, and `tests/conformance_qspec/extension.rs` cover FR-349-AC-4 independently; the new trace claim is inaccurate, not a missing criterion. A full computed matrix was attempted but exceeded the 40-second focused review window; the cited bindings and assertions were inspected directly. No optional semantic audit or plan was selected.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, the test carries only `#[trace("FR-349-AC-2")]`; the unsupported FR-349-AC-4 claim is removed. |

## Current verdict

**PASS** at exact. The parent is the reviewed head and the delta is one trace line, with no code or assertion change. `git diff --check` passed. SR-15650's code/Rust PASS carries unchanged; no aggregate gate was run in this reviewer lane.
