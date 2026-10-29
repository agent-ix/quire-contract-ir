---
id: SR-609105
title: "IR-1032 PR #609 ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-063-dimension-equivalence-node.md; PR #609 "
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-063
    type: references
---

## Summary

Ticket IR-1032. The changed SHALL statements are conditional and testable against the named malformed-wire and graph oracles. Reviewed exact PR against QSpec.

## Verdict

**PASS** for this method. This is a spec-only PR; no Cargo or Make gate was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

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

## Current verdict

**PASS** for this method. The fix changes only AC-5 source attribution and its dependency explanation; the clean review remains valid.
