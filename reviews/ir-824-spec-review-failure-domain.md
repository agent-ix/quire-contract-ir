---
id: SR-11832
title: "Failure-domain review of IR-824 reaches refusal order (PR #420)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (reaches paragraph, FR-038-AC-252), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (item 11)"
review_set: subset
---

## Summary

Ticket: IR-824. Combined-defect inputs and refusal precedence for
`quire.op.model.reaches` over a relationship end, against QSpec FR-322-AC-57
(origin/main ) and the current reader. The member-level order
(direction, multiplicity, type) matches QSpec and code, and each RN-23..RN-27
code, cause and locus is pinned in AC-252 and TC-048 item 11. The multiplicity
refusals come from the single FR-152 derivation the FR-038 binding already
requires ("not maintain a second direction or multiplicity table"), which is
`RelationshipDecl::navigation_type` (`model_members.rs:252`). One combined
input is left open.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | medium   | AC-252 and the FR-038 paragraph order direction, multiplicity and type, but do not place the operand-1 conformance check (`arguments/1`) in that order. QSpec FR-322-AC-57 checks the receiver effective endpoint for `arguments/0` and, for `reaches`, `arguments/1` before direction. The reader checks `arguments/1` last, after the type check (`operations.rs:2257-2268` after `2242`, `2263` and `2249-2256`). For an unrelated operand 1 combined with a wrong-direction, unbounded/ordered or different-type end, QSpec refuses at `arguments/1` and the reader at `member.name`. No RN vector combines these, so the conformance vectors will not catch it. State the operand-1 position (per QSpec, before direction) and add one combined case to TC-048 item 11, or record the divergence explicitly. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4424; crates/quire-contract-model/src/checked_package/v2/operations.rs:2242-2268 | missing-requirement |

## New findings (disposition pass 1)

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-002 | low      | The normative text is exact: FR-038 now checks operand 1 against the receiver endpoint type, as QSpec FR-322-AC-57 states (RN-21 subtype admits, RN-22 unrelated refuses at `arguments/1`). But the implementation note ("The implementation slice moves the operand-1 check ahead of direction ... leaves every single-defect outcome unchanged"; TC-048 item 11 "reorders the reader's operand-1 check") describes only a reorder. The reader compares operand 1 with `destination.type_ref` (`operations.rs:2265`). Moved as-is ahead of the type check, RN-25 (`reaches invoiced` on M10, operands M1, M1: receiver Order, destination Invoice) would refuse at `arguments/1` instead of `member.name`, contradicting AC-256 and the note's own "single-defect unchanged" claim. The note should also say the check retargets from the destination type to the receiver endpoint type. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1167-1169; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1853; crates/quire-contract-model/src/checked_package/v2/operations.rs:2257-2268 | wrong-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 1840592 states the QSpec FR-322-AC-57 order in FR-038 (lines 1156-1170) and in the new FR-038-AC-253: receiver-endpoint checks of `arguments/0` and `arguments/1` come first, then direction, multiplicity and type. TC-048 item 11 adds the combined cases (an unrelated operand 1 with each of wrong-direction, unbounded, ordered and other-type ends refuses at arguments1). |
| FND-002 | fixed | Fix: FR-038 (lines 1167-1169) now says the implementation slice "compares operand 1 with the receiver endpoint type instead of the destination type", and TC-048 item 11 (line 1853) adds "so a well-formed other-type end still refuses at member.name"; RN-25 stays at member.name. |
