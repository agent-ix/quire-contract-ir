---
id: SR-2026110414560009
title: IR-1326 PR 896 code-review review
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-contract-ir; PR #896; IR-1326; FR-038-AC-461/462; code-review'
review_set: subset
---

## Summary

Independent review of PR #896 at 254da53f against QSpec main 860e0890.

## Verdict

PASS after disposition round 1 at 283af7a1; the historical findings below are fixed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Duration bounds use the first temporal binding for a clause and ignore later occurrence bindings. QSpec FR-322 permits one clause to have distinct claim occurrence clock configurations; an ms bound can therefore be admitted when another occurrence uses s. Check the bound against every applicable occurrence or refuse the incompatible clause. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:1218-1234 |
| FND-002 | medium | Hosted Rust Checks fails make lint: qsl-string-edge reports 35 unmarked new string edge occurrences in temporal.rs. The reviewed head cannot pass the required final gate until those edges are typed or explicitly marked. | crates/quire-contract-model/src/checked_package/v2/temporal.rs:1050-1370 |

## Coverage

FR-038-AC-461 and FR-038-AC-462 examined; QSpec FR-370-AC-17/18 and FR-322 temporal binding occurrence rule used as context. Plan completion: not assessed. The published QSpec vector is read by reference; no case count, digest, or source revision is pinned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The reader gathers occurrence clock units by clause and retains one only when every binding agrees; the mixed ms/s case now refuses ill_typed/unit-mismatch at the lower bound, while two ms occurrences admit. |
| FND-002 | fixed | make string-edge on pushed head 283af7a1 exits 0 with no unmarked or unlisted string comparisons. New aggregate kinds use typed enums and deliberate string-edge boundaries. |
