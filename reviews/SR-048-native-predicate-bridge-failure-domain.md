---
id: SR-048
title: "Failure-domain review of native predicate to TL projection"
type: SpecReview
analysis: failure-domain
scope: "projection/valuation decisions, strict decode, result availability, completeness, resource and correction cases at cf4beaf15e35dfe276749637dbfb16c230070514"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-048: Failure-domain review of native predicate to TL projection

## Summary

The result model keeps availability, execution, truth and completeness
orthogonal and assigns every non-value state to a closed decision vocabulary.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6312 | high | **FIXED:** Incomplete, unavailable, unsupported, failed and refused states were flattened. Availability, execution, truth and completeness are now independent with a total first-match mapping. | FR-025 Public v1 records; Behavior | wrong-requirement |
| FND-6313 | high | **FIXED:** Facts outside exact decision support could erase settled truth. Only an inside-set fact affects truth; outside-set gaps remain typed. | FR-025 completeness rules; AC-5 | wrong-requirement |
| FND-6314 | high | **FIXED:** Structural, semantic and allocation failures overlapped. Malformed inputs, semantic decisions and operation resource diagnostics are now disjoint. | FR-025 Outputs; Behavior | wrong-requirement |
| FND-6315 | medium | **FIXED:** A bare `not-yet-observed` claim could manufacture incompleteness. Exact assertion bytes and the selected authority reader now bind the classification. | FR-025 Inputs; Public v1 records | wrong-requirement |
| FND-6329 | high | **FIXED:** `not-yet-observed` covered every observation state while its cause was restricted to open observations. The cause now applies to every authority-verified not-yet classification. | STD-001; FR-025 valuation mapping | wrong-requirement |
| FND-6330 | medium | **FIXED:** Conflict named an undefined retained premise and non-literal dimension. It is now limited to contradicted deciding facts under exact dimension `deciding-facts`. | STD-001; FR-025 code allocation | wrong-requirement |
| FND-6331 | high | **FIXED:** The bridge lacked a constructible outcome when an availability, source-result or mapping contract failed before assertion/result validation. A base-only contract-admission shape now omits untrusted fields, uses six exact causes and three closed dimensions, precedes assertion parsing, and has independent TC-038 mutations. | STD-001; FR-025 decision shapes, cause allocation and staged valuation mapping | wrong-requirement |
| FND-6332 | high | **FIXED:** Universal observation/result binding statements contradicted the base-only contract-admission shape. Base, post-contract-admission and result-bearing obligations are now scoped separately in Outputs, Behavior and AC-2. | FR-025 Outputs; Behavior; FR-025-AC-2 | wrong-requirement |
| FND-6333 | medium | **FIXED:** `result_projection_ref` recomputation was required before every valuation decision, including the base-only pre-validation shape that cannot have a result. Recomputation is now required only for result-bearing post-contract-admission decisions. | FR-025 result projection identity | wrong-requirement |

## Verdict

The closed cause/code allocation and conformance mutations distinguish every
named non-success family without Boolean fallback. **PASS.**
