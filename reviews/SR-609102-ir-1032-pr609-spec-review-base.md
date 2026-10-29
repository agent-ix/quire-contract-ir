---
id: SR-609102
title: "IR-1032 PR #609 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-063-dimension-equivalence-node.md; PR #609 "
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-063
    type: references
---

## Summary

Ticket IR-1032. Wire-first refusal and preserved graph cases match QSpec FR-322-AC-118/139. The attribution of unrelated vector mutations to TC-280 is false. Reviewed exact PR against QSpec.

## Verdict

**FAIL** for this method. This is a spec-only PR; no Cargo or Make gate was run.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-063-AC-5 calls the negative-zero dimension/unit and model-owner-version mutations “TC-280’s,” but QSpec TC-280 contains none of them. They are in `node-identity-vectors.json` and FR-322-AC-139. This misdirects the Test oracle; name the vectors, or add those cases to TC-280 without dropping any refusal. | spec/checked_package/functional/FR-063-dimension-equivalence-node.md:94; QSpec TC-280; QSpec FR-322-AC-139 | wrong-requirement |

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-063-AC-5 | examined | The QSpec TC-280 equivalence mutations with absent `root_scale`, zero or negative `numerator` or `denominator`, a one-entry `dimension_node_ids` list, or a present `declaration` refuse `malformed_wire` at the node before graph checks. A schema-valid but unreduced `root_scale`, one missing dependency, non-self `semantic_type`, owner outside the selected source lock, or a stale key after pair or scale mutation refuses `invalid_semantic_graph` at the equivalence node, with FR-322's stated check order when an earlier independent defect is also present. The same wire-first rule makes TC-280's negative-zero spelling in a dimension exponent, unit scale or unit offset, and a `version` member on a model owner refuse `malformed_wire` at the offending node; schema-valid semantic, stale-key and owner-join mutations retain their `invalid_semantic_graph` refusal. |
| FR-063-AC-4 | context_only | Each QSpec TC-280 adverse pair or graph is refused `invalid_semantic_graph` at its equivalence node. |
| FR-322-AC-118 | context_only | Wire-invalid equivalence members refuse `malformed_wire` before graph checks; schema-valid semantic defects refuse `invalid_semantic_graph`. |
| FR-322-AC-139 | context_only | Dimension-equivalence wire precedes graph, and vector mutations use `refused_by` to select `malformed_wire` or `invalid_semantic_graph`. |

## Evidence

- The PR changes one FR document and adds a QSpec FR-322-AC-139 reference; all earlier criteria remain.
- `quire validate --scope. spec/checked_package/functional/FR-063-dimension-equivalence-node.md` passed.
- Focused `quire properties --criteria` classified 6 criteria; `git diff --check` passed.
- QSpec TC-280 MU-23 and MU-24 cover dimension-equivalence graph and wire mutations. The other four schema mutations are listed only in QSpec `node-identity-vectors.json` and FR-322-AC-139.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | AC-5 still calls a negative `denominator` a QSpec TC-280 equivalence mutation, but TC-280 MU-24 covers negative numerator and zero denominator only. QSpec FR-322-AC-139 requires negative denominator as a wire refusal. Cite that criterion for the full component boundary, or name the TC-280 cases precisely; retain negative denominator in the IR criterion. | spec/checked_package/functional/FR-063-dimension-equivalence-node.md:94; QSpec TC-280 MU-24; QSpec FR-322-AC-139 | wrong-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |, AC-5 names QSpec FR-322-AC-139 node-identity-vectors.json for the four cross-form mutations and the dependency paragraph separates those vectors from TC-280. All four refusal cases remain. |
| FND-002 | still-open | The first AC-5 sentence still attributes negative denominator to TC-280, where MU-24 has no such mutation. The requirement is in QSpec FR-322-AC-139. |
| FND-002 | fixed | Round 2: AC-5 now cites governing QSpec FR-322-AC-139 for the full absent/zero/negative numerator-or-denominator wire refusal set. The negative-denominator case and every other refusal remain. |

## Current verdict

**PASS**. FND-001 and FND-002 are fixed; no open finding.
