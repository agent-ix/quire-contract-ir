---
id: SR-038
title: "Dependency review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: dependency
scope: "StR-001, FR-012, FR-023, FR-025, issues 52/57/63/64"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-038: Dependency review of issue 64

## Summary

The dependency lens separates semantic enablement from the bridge and its
downstream exporter. The graph is acyclic; FR-025 remains correctly parked
until the predicate and reviewed-profile authorities exist.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-381 | medium | **Closed:** the draft did not distinguish parent coordination (#52) from downstream FRETish consumption (#57). Both are now explicit references, and #57 is expressly not a prerequisite. | FR-025 relationships and Dependencies | missing-requirement |
| FND-382 | high | **Closed:** unfinished issue #63 could have been treated as a soft dependency with a temporary Boolean mapping. FR-025 now makes it a hard prerequisite and defines unsupported-without-fallback behavior until it is reviewed. | FR-025 Behavior/Dependencies; issue #63 | missing-requirement |

## Classification

| Requirement or authority | Class | Rationale |
|---|---|---|
| StR-001 | feature need | Requires one identity-preserving semantic contract for downstream lowerings. |
| FR-012 | enablement | Supplies stable clause, anchor, and source-span identity. |
| FR-023 | enablement | Supplies the complete immutable BoundClause population. |
| issue #63 | enablement | Must supply total-Boolean predicate and proposition-map identity. |
| reviewed native temporal profile | enablement | Owns the source temporal meaning and clock/history/closure rules. |
| versioned TL profiles and evaluator | enablement | Own the candidate target meaning evaluated by correspondence tests. |
| FR-025 | cross-component enablement | Owns the public correspondence decision and join identity. |
| issue #57 | downstream feature | Generates FRETish output only after FR-025 is available. |

## Dependency Graph

`FR-012 -> FR-023 -> FR-025`, `issue #63 -> FR-025`, `reviewed native
temporal profile -> FR-025`, and `versioned TL profiles/evaluator -> FR-025 ->
issue #57`. Issue #52 coordinates the chain but does not replace a semantic
prerequisite.

## Topological Order

1. Retain FR-012 and FR-023 as completed Contract IR foundations.
2. Review and accept issue #63 and the coherent native temporal profile; pin
   exact TL profile/evaluator revisions.
3. Implement FR-025 and TC-038 against those exact inputs.
4. Review and implement the issue #57 output mapping, if still selected.

## Cycles

None. The native profile declares the meaning consumed by FR-025; it does not
depend on the Contract IR implementation to define that meaning.

## Result

**PASS with implementation parked at step 2.** Ticket existence, a merged draft,
or matching formula text does not satisfy a prerequisite.
