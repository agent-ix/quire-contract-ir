---
id: SR-610051
title: "code-review — IR-1052 Boolean transition V2 reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/, tests/conformance_qspec/transition.rs; IR-1052"
review_set: subset
---

## Summary

Independent code-review review of PR #607 against QSpec. Examined FR-067-AC-1 through AC-7.

## Verdict

**CONDITIONAL** — 1 finding(s); QSL-authored AC-7 remains open on QSL-1163.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The operation step checks the StateTransition member operator but not the reciprocal ProtocolControl case. With the published protocol-control application edited to member operator transition, member_is_well_formed(ProfileOperator) returns true and the new conditional does not run. The isolated operation step therefore misses QSpec XT-20 operation-member-mismatch; add the reciprocal check and a direct operation-step test. Complete-reader XT-07 still correctly fails at the schema stage. | crates/quire-contract-model/src/checked_package/v2/operations.rs:858-867 |

## Coverage

Plan completion: not assessed. The computed Quire matrix marks FR-067-AC-1 through AC-6 tagged and AC-7 untagged. No specification file changed in this PR; spec-review was not applicable. Formal aggregate gates are reserved for the final reviewed head by the code lead.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | KnownOperation::ProtocolControl now refuses transition, alongside StateTransition refusing other members; isolated operation-step test asserts XT-19 and XT-20 with code, cause and locus. |
