---
id: SR-049
title: "Integrity review of native predicate to TL projection"
type: SpecReview
analysis: integrity
scope: "PredicateRef, result_projection_ref, sibling artifact refs, projection_set_ref, source-result authority and supersession at cf4beaf15e35dfe276749637dbfb16c230070514"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-049: Integrity review of native predicate to TL projection

## Summary

Every bridge identity has an exact domain, canonical tuple and authority
boundary; local derived identities do not replace native producer authority.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6316 | high | **FIXED:** Digest tuples lacked exact members, nested shapes and reproducible preimages. Domains and three synthetic goldens are now exact. | FR-025 canonical profiles and vectors | wrong-requirement |
| FND-6317 | high | **FIXED:** Real source-result bytes could be paired with invented projected fields. The selected reader and deterministic mapping now derive every field before hashing. | FR-025 producer-input profile; Behavior | wrong-requirement |
| FND-6318 | high | **FIXED:** A local hash risked becoming a second result authority. `source_result_ref` remains producer-owned and `result_projection_ref` is explicitly derived. | FR-025 result projection identity | wrong-requirement |
| FND-6319 | high | **FIXED:** The pure bridge overclaimed global supersession validation from one predecessor. Only the supplied direct predecessor is verified; graph checks remain producer-owned. | FR-025 correction rules | wrong-requirement |

## Verdict

Every identity axis has a one-axis mutation, prior bytes remain immutable, and
the bridge creates no competing semantic authority. **PASS.**
