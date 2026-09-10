---
id: SR-053
title: "Failure-domain review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: failure-domain
scope: "FR-026, STD-001, and TC-039"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: reviews
---
# SR-053: Failure-domain review of issue 64

## Summary

The failure-domain review challenged identity authority, missing valuations,
partial traces, progress/closure confusion, contradictory results, and late
supersession at exact snapshot `558c4dc`. Four material gaps were fixed without
moving evaluation or observation ownership into Contract IR.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6411 | high | **Fixed:** a caller-claimed `trace_ref` was not derived from an authority-complete per-position valuation population. FR-026 now requires a rectangular set of admitted FR-025 valued decisions and content-addresses the exact generated trace and request. | FR-026 Inputs; Valuation, trace and evaluator request; FR-026-AC-1/2/8 | missing-requirement |
| FND-6412 | high | **Fixed:** the old join flattened execution, decision scope, closure, truth, settlement, support, and completeness into one progress value. The public record and exact join table now preserve and validate each independent axis. | FR-026 Public v1 records; Progress, closure and result joining; FR-026-AC-5/6 | missing-requirement |
| FND-6413 | high | **Fixed:** supersession named only a prior reference and could not prove immutable same-producer ancestry. The join now requires and validates the direct predecessor bytes, identity, revision, digest, subject, and producer while expressly declining global graph authority. | FR-026 Inputs; Progress, closure and result joining | missing-requirement |
| FND-6414 | high | **Fixed:** all three formula golden digests had been computed with two literal backslash-zero bytes. The corrected vectors use actual zero-byte domain separators and reproduce independently. | FR-026 Formula construction and identity | wrong-requirement |

## Failure-Domain Result

- A missing, duplicate, foreign, stale, non-Boolean, or non-valued proposition
  cell cannot become an omitted false signal.
- Contract admission precedes interpretation of subject, observation,
  availability, and result bytes; failed admission exposes only base fields.
- Closed-incomplete observations produce no evaluator request. Closed-complete
  pending results and impossible execution/truth/settlement combinations are
  refused before producer comparison.
- Open early-final truth requires a continuation-stable decisive basis and
  complete exact decision support.
- Late data creates new immutable revisions and a validated direct-predecessor
  edge; no prior object is rewritten.

## Result

**PASS after remediation.** No open failure-domain finding permits fallback,
fabricated values, or implementation before the dependency gates in SR-055.
