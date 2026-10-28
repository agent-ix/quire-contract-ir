---
id: SR-611026
title: "Code and Rust review of IR-874 protocol dependency exactness"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/structural.rs, protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Reviewed quire-contract-ir PR #629 at frozen head 7ac176525c667a13a44a3fdaea28b2e60e8eee26 against IR FR-038-AC-285, AC-324 and AC-327, QSpec FR-322-AC-96 and FR-464-AC-6/13. The graph stage now owns exact protocol-family dependencies. No applicable AssuranceProfile was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Clause slots repeat the exact dependency check after the graph stage has already completed the same comparison; remove the unreachable second implementation so the rule has one owner and no redundant body walk. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3251 |

## Verdict

CONDITIONAL. The reader's new stage order and target resolution match the reviewed obligations, but the clause-specific duplicate should be removed before merge. The changed tests assert exact refusal code, cause and path through the public reader; no copied fixture or gate narrowing appears in the diff. Coder-reported focused protocol module 139/139, Clippy and rustfmt passed; this reviewer did not run aggregate gates.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At fix head 34e3a57bb55ea70a0bc0ae33da4838da056cc46e, `check_clause_slots` begins with `ProtocolScope::new`; the duplicate `application_dependencies` comparison is gone. The structural graph-stage check remains the sole exact dependency comparison for the clause. |
