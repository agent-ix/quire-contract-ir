---
id: SR-1002
title: "criterion strength review of PR 264 (16 new acceptance criteria)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@184e6668bb8722f03c68a6b9924586593233118d; FR-013-AC-5, FR-016-AC-5 to AC-8, FR-020-AC-3, FR-032-AC-6, FR-033-AC-6, FR-034-AC-6, FR-034-AC-7, FR-038-AC-89 to AC-94 (git diff origin/main...HEAD)"
review_set: subset
---
# SR-1002: criterion strength review of PR 264

## Summary

Ticket: IR-274. For each AC, a mutation that it would catch:

- FR-013-AC-5: a number accepted in one of the eight members; a non-minimal
  string accepted; an out-of-range string mapped to the wrong code.
- FR-016-AC-5: one member written as a number, or `maximum_items`/`revision`
  stringified.
- FR-016-AC-8: `sha256_with_domain` used, or a prefix byte dropped.
- FR-033-AC-6 and FR-034-AC-6: the ceiling left at `u64::MAX`, or a limit
  refusal mapped to `allocation_failed`.
- FR-034-AC-7: a limit or region written as a number (refused past 2^53), or
  two near-`u64::MAX` limits collapsing to one identity.
- FR-038-AC-89: any change in node-key, `ir_id` or lowered-package bytes.
- FR-038-AC-90: the limit argument ignored.
- FR-038-AC-92: a preimage on the wrong encode path.
- FR-038-AC-93: the check skipped, the order after `byte-digest-mismatch`, or
  the wrong pointer.
- FR-038-AC-94: the ceiling off by one.

Each of these can fail. Weaknesses follow.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-016-AC-7's source-reading test covers "the canonical types" but never says which types those are. A test may pick a set that leaves out a type holding an `Option` serialized as `null` or a `Value`, and still pass. Name the types, or define them as every type reachable from the five envelopes | spec/model/functional/FR-016-canonicalization-digests.md:142 |
| FND-002 | low | FR-016-AC-6 ("bytes returned equal `quire_canonical::to_vec` over the kind's typed envelope"), FR-034-AC-7's first clause and FR-038-AC-89's last clause use the implementation's own call as the oracle. They fail only if the code bypasses `to_vec` for that envelope, not if the envelope is wrong. Spelling is pinned by FR-016-AC-5 and FR-034-AC-7's strings. FR-016-AC-8's "differs from `sha256_with_domain`" is implied by its first clause | spec/model/functional/FR-016-canonicalization-digests.md:141 |
| FND-003 | low | FR-032 states that the limit "bounds the work of the encode and is not compared with its result afterwards". FR-032-AC-6 cannot tell the two apart: a full encode followed by a length compare passes every clause. "Very large" and "far below" are unquantified | spec/output_mapping/functional/FR-032-admit-output-mapping-request.md:91 |
| FND-004 | low | FR-038-AC-91's "no function that walks a `Value` to write canonical bytes" gives a source scan nothing to match. Name the symbols to be absent (for example `encode.rs` `value_to_vec`), so the check can fail | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1412 |

## Verdict

Mostly strong. FND-001 needs a defined type population. The lows sharpen
oracles. FR-038-AC-93's missing vectors are SR-999 FND-003.

## Dispositions

Reviewed at agent-ix/quire-contract-ir@553736cfce62fe6949915559c471d4a7f5349d21 (main still 7d7716d; not rebased onto #263, whose FR-038, TC-048 and checked_package tests.md hunks conflict textually, expected rebase work). Proof re-run: no schemas/, corpus or code file in `git diff origin/main --stat`; `make corpus` exit 0; workspace tests all pass (204 + 101, 0 failed); validate passes, grammar 1 (FR-014 baseline), 215 ACs, strict 23 unbacked, coverage rows 237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 553736c |
| FND-002 | fixed | 553736c |
| FND-003 | fixed | 553736c |
| FND-004 | fixed | 553736c |
