---
id: SR-202610310155
title: "Gap analysis of IR-1230 temporal clock binding reader PR 788"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1230; PR 788 head c86bccab; QSpec FR-322-AC-294; tests/conformance_qspec/temporal_clock_bindings.rs; crates/quire-contract-model/src/checked_package/v2/temporal_binding.rs"
review_set: subset
---

## Summary

The computed repository Test Matrix and the external QSpec criterion were reconciled with the PR's by-reference TCB test and reader implementation. The current test covers the three clock kinds and published refusal cases, but its two-clause leg does not assert the criterion's exact per-clause declaration keys or structural row order.

## Verdict

CONDITIONAL. Strengthen the TCB-11 assertion and adverse coverage before claiming the exact multi-clause identity portion of FR-322-AC-294.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The TCB-11 two-clause test checks only row count and distinct declarations; it does not verify both returned declaration keys, structural order, or that removing the second row refuses at the second clause body. | tests/conformance_qspec/temporal_clock_bindings.rs:127 |

## Coverage

Plan completion: not assessed.

`quire matrix --scope . --format json` completed, but it covers this repository's `spec/` tree and therefore does not mint external QSpec `FR-322-AC-294`; the external trace was checked manually. The new test carries `#[trace("TC-134", "TC-233", "FR-322-AC-294")]`. It reads QSpec `temporal-clock-binding-vectors.json` by reference at current QSpec `6a021ef8` and passed against the frozen PR head. Positive TCB-01..03, schema TCB-04..06, semantic TCB-07..10 and TCB-12..13, and exact-period controls execute. TCB-11 carries two distinct formula references (QSpec node indices 28 and 29) with the same model, profile, and clock name. The test constructs two rows but `admitted` checks only `rows.len() == 2` and `rows[0].declaration != rows[1].declaration`, so a reader keyed by row position or clock name could satisfy the assertions while violating exact declaration identity and per-clause coverage. QSpec #571 resolved the configured positive fixture gap by adding required `temporal_bindings` rows.

## Scope and bindings

Examined QSpec FR-322-AC-294 and its TCB-01..13 vector data; FR-322 temporal clock binding prose was context. Examined the changed conformance test, its fixture construction and assertions, and the reader's key/coverage checks. No hand-written matrix was used. No unspecified production surface was found beyond the writer and work-bound findings recorded in the companion code review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TCB-11 now checks each returned declaration against the exact constructed clause ID, both published formula references and clock names, structural row order, and the second clause body locus when its row is removed. |

Confined recheck: PR head `adfba19d`; `temporal_clock_` focused tests passed 2/2 against QSpec `6a021ef8`. The original finding text remains unchanged.
