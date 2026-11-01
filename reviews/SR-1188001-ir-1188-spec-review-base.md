---
id: SR-1188001
title: "IR-1188 FR-048 bulk node-id mapping base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-048-derive-structural-and-group-candidates.md; IR-1188"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-048
    type: reviews
---
# IR-1188 FR-048 bulk node-id mapping base review

## Summary

Reviewed the IR-1188 additions to FR-048 against the existing V2 node and term representations, FR-038, FR-349, FR-019 and the repository's spec rules. No applicable AssuranceProfile was found in the repository. The new criteria are authored for the implementation ticket and do not yet have test trace tags.

## Verdict

**PASS after disposition.** The initial mirrored-operation gap was fixed in the same draft; the final requirement specifies one callback result for each logical slot and its read mirror.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A stateful callback could give different replacements to the same operation identity stored in both the closed record and its typed read view, leaving serialization and typed reads inconsistent; the first draft treated them as independent occurrences. | FR-048 Behavior, AC-6, AC-7; `typed_terms.rs:28-32,341-357` |

## Dispositions

- FND-001: **fixed.** FR-048 now defines a logical identity slot, calls the callback once for that slot and applies the same result to the operation record and its typed view. AC-7 tests the mirror with a stateful callback.

## Coverage

- ID and link validation passed for changed FR-048; AC-6 through AC-9 are sequential.
- AC-6 covers callback absence, replacement, independent equal slots and non-cascading behavior.
- AC-7 covers nested typed slots, mirror coherence, preservation of unrelated data and deterministic order.
- AC-8 covers depth, one-walk resource use and invalid replacement handoff to the reader.
- AC-9 covers the public API and absence of a new admission path.
- The change is spec-only. Its four new criteria have no implementation trace tags yet; the IR-1188 code slice owns those tests.
