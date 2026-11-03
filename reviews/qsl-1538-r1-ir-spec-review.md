---
id: SR-20261103150647
title: "spec-review of QSL-1538 FR-038-AC-286 delivery interval (IR)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #855 at d4d4876a; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-286); QSpec PR agent-ix/quire-specification#610 at f7934212 (FR-465, read-only, context)"
review_set: subset
---

## Summary

Spec review of the single FR-038-AC-286 edit in quire-contract-ir PR #855. Ticket: QSL-1538. The edit inserts a parenthetical into the channel clause: delivery is one of the four QSpec FR-465 policy words or a canonical delivery count interval `[lo,hi]` with `lo <= hi`, and any other spelling, `lo > hi` included, refuses `malformed_wire` at the term.

Agreement with QSpec PR #610 was checked against the FR-465 text in that PR's worktree, read-only. FR-465 defines the interval as `[`, the decimal numeral of `lo`, `,`, the decimal numeral of `hi` and `]`, with no space and no other character. Each numeral is a natural number no greater than 18446744073709551615, written in digits `0` to `9` with no sign and no leading zero (`0` itself is a numeral), and `lo <= hi`. Any other text refuses `malformed_wire` at the term.

- **Grammar.** The IR grammar matches FR-465 on every point: no space, decimal, at most u64::MAX, no sign, no leading zero, `0` admitted, `lo <= hi`, and the same code and locus.
- **Test vectors.** The IR tests' admitted and refused sets are a superset of FR-465-AC-10 and of TC-947 CH13a and CH14a.
- **Body distinctness.** The CH13b requirement that `[1,1]` and `exactly-once-premise` give distinct bodies holds by construction: the wire text differs, and the unit test asserts the parsed values differ.
- **Duplicate and lose steps.** FR-465 says an interval enables neither `duplicate` nor `lose` (FR-426), which matches `protocol_member_classes`.

## Examined scope

- FR-038-AC-286 (examined): "A role admits the FR-465 static and replicated shapes only, ...; a channel admits only same-clause role endpoints, an eligible message type, explicit ordering, explicit delivery (one of the four QSpec FR-465 policy words or a canonical delivery count interval `[lo,hi]` with `lo <= hi`, any other spelling, `lo > hi` included, refusing `malformed_wire` at the term), and a FIFO key with equality-stable type; ..."
- QSpec FR-465 "Delivery count interval" and "Delivery spellings" bullets (context_only, PR #610): "The text `[lo,hi]` is `[`, the decimal numeral of `lo`, `,`, the decimal numeral of `hi` and `]`, with no space and no other character. Each numeral is a natural number no greater than 18446744073709551615 ... with no sign and no leading zero (`0` itself is a numeral), and `lo <= hi`; so one interval has one text."
- QSpec FR-465-AC-10 (context_only, PR #610): admitted `[0,3]`, `[0,0]`, `[2,2]`, `[0,18446744073709551615]`; refused `[4,3]`, `[0, 3]`, `[00,3]`, `[+0,3]`, `[-1,3]`, `[0.5,3]`, `[0,18446744073709551616]`, `[3]`, `[0,3`, `0,3`.
- TC-048 step 4 (context_only): "channels with each delivery and ordering premise ... the local adverse forms ... of TC-947" — covers the new vectors by reference; no matrix edit needed.

## Verdict

PASS. The AC-286 edit is atomic, testable and consistent with QSpec FR-465 as amended by PR #610. It defers the exact meaning of "canonical" to FR-465 by name, which is the right owner, and it states the refusal code and locus. No contradiction was found with other FR-038 ACs. The class-reconstruction consequence (no `duplicate` or `lose` for an interval) follows from QSpec FR-426 and FR-432 through FR-038-AC-325, which needs no edit. The misplaced trace tag on its unit test is recorded in the gap-analysis SR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
