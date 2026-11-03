---
id: SR-20261103150652
title: "IR-1278 IR-1279 PR 862 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #862; IR-1278, IR-1279; temporal_binding.rs; temporal_clause_names.rs; intake.rs; mod.rs; temporal_clock_bindings.rs; temporal_binding_public.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-019", type: references }
---

## Summary

Compared targeted criteria to tagged tests and implementation; no plan selected. Plan completion: not assessed. Ticket: IR-1278 and IR-1279.

## Verdict

**CONDITIONAL** — the published-vector test omits required node-locus and work-limit assertions.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | BO and CN refusal assertions check code, cause and primary pointer but never structural node locus, leaving FR-038-AC-441 and AC-447 unverified for every adverse vector. | tests/conformance_qspec/temporal_clock_bindings.rs:190 |
| FND-002 | medium | No focused boundary test sets the work limit at the first temporal-binding or name-row charge and proves `Incomplete` has the required setting, pointer, consumed count and no partial admitted view. | tests/conformance_qspec/temporal_clock_bindings.rs:104 |

## Coverage

- Plan completion: not assessed.
- FR-038-AC-437 through FR-038-AC-440 and AC-442 through AC-446, examined: changed tests trace each and positive/adverse vector assertions exercise core joins and code/cause/pointer.
- FR-038-AC-441 and AC-447, examined: structural node loci and first exhausted work-charge controls are missing.
- QSpec FR-322-AC-342 through FR-322-AC-370, examined: published CN and BO case sets are consumed by reference with expected case counts; no vectors are copied.
- FR-019-AC-25, context only: external public API tested separately.
- Computed matrix unavailable: `quire matrix --scope . --format json` emitted module diagnostics and no usable JSON; trace tags and assertions were inspected directly.
- Optional repository-wide semantic review: not selected; targeted PR-diff intent-to-test review performed.

## Evidence

Both adverse branches stop after `refusal.loci[0].path` equality. The QSpec vector `expected` objects supply code, cause and locus but no structural node oracle, so the test must establish node-locus expectations from the patched graph or an independent case map. The production name validator charges one work visit at each row before inspection; the binding validator charges each row and occurrence traversal. Existing focused tests do not put the work budget at either first charge. No aggregate gates were run by this reviewer.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Both BO and CN adverse branches now assert refusal.locus against the patched graph or row declaration; the duplicate binding refusal now retains the declaration locus. |
| FND-002 | fixed | The focused test finds each first refusal work threshold and checks the preceding Incomplete work kind, limit, consumed count and exact binding/name row pointer. |

## Round 1 evidence

The disposition compares commit ed0a7a10 against frozen head 70a0de01. The revised tests exercise the published QSpec cases by reference. No new finding arose from the focused fix diff.
