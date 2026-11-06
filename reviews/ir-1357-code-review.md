---
id: SR-2026110602354607
title: "IR-1357 PR 924 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 924; FR-358 and native selected-identity joins"
review_set: subset
---

# IR-1357 PR 924 code-review review

## Summary

Reviewed frozen PR #924 head 12286329 against FR-358, FR-019, TC-467 and changed source and tests. Ticket: IR-1357.

## Verdict

FAIL: 2 finding(s) require a fix round.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Stage and union joins compare caller-controlled document, original-identity, authority, declaration and member strings with only constant or element-count work charges. Long text can be fully inspected after a budget admits one work unit, violating the byte-before-scan rule and making the work limit nonbinding for these paths. | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:1127-1153 |
| FND-002 | medium | Enum validation clones the full CheckedNodeId and occurrence role to construct a source-index lookup key after only a work charge. No storage charge precedes these dynamic copies, so repeated occurrences allocate outside the caller budget. | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:808-811 |

## Examined Criteria

- role: examined; id: FR-358-AC-1; excerpt: A public `NativeJoinBudget` with finite work and storage limits is passed by mutable reference to stage-row acquisition and each union, record, tuple, enum and collection join. Two individually affordable operations whose combined work or storage exceeds one limit stop at the first unpaid step of the second operation; no operation silently resets either counter. The public item list and call graph contain one mandatory budgeted acquisition path and no compatibility wrapper or parallel one-shot join entry.
- role: examined; id: FR-358-AC-2; excerpt: For each of stage-row, union, record, tuple, enum and collection acquisition, a real admitted fixture at both exact work and exact storage limits yields the complete same selected value as an ample allowance. One unit below either measured requirement refuses with the typed exhausted kind, first native locus and attempted charge, without a partially authoritative result.
- role: examined; id: FR-358-AC-3; excerpt: Every IR-owned row, key, handle, container and map/set entry retained or cloned by the budgeted path, and each byte of copied dynamic payload, is charged by the defined logical-storage rule before the corresponding reserve, clone or insert. The tests make a prior nonempty map require an `Arc::make_mut` copy and show that a limit one below that copy's charge refuses before the new row appears. A borrowed inspection incurs no storage charge.
- role: examined; id: FR-358-AC-4; excerpt: Cumulative counters equal the sum of admitted charges across a successful stage acquisition and all five joins, while each success delta equals that operation's charge. A refusal clears the current success delta, leaves earlier admitted consumption visible, and records the first unpaid kind and amount without incrementing the denied charge; overflow cannot wrap either counter. Repeating the ordered inputs and limits, including against an admitted package clone, gives equal counters, values or first stops.
- role: examined; id: FR-358-AC-5; excerpt: Each budgeted join preserves FR-352's selected-document, structural node, declaration/member identity, producer order and duplicate checks at sufficient allowance. Missing, foreign, conflicting, stale, wrong-form and incomplete rows retain their existing typed native causes and first loci, without a partial selected value. No caller-created opaque identity becomes authoritative through the budget API.
- role: examined; id: FR-358-AC-6; excerpt: The exported API is cycle-free and has no QSL or runtime dependency. In-repository callers use the mandatory budget parameter; the IR implementation introduces no compatibility path, pin, copied artifact or diagnostic-payload claim belonging to IR-1359.

## Review Evidence

Diff inspected at frozen head 12286329. The computed matrix in the coder log tags AC-1 through AC-5; AC-6 is inspection-only. Focused tests, scoped Clippy and compile were supplied by the coder; this reviewer ran no Cargo gate. Plan completion: not assessed.
