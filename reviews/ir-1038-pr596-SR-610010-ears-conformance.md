---
id: SR-610010
title: "IR-1038 spec-review/ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-066-retain-identity-bound-execution-points.md; spec/spec.md"
review_set: subset
---

## Summary

Ticket IR-1038. Frozen PR #596. Reviewed FR-066-AC-1 through AC-5 and the master index against current QSpec, FR-040 and the public model.

## Examined units

- FR-066-AC-1 (examined): The four published QSpec `node-identity-vectors.json` execution-point application preimages, read directly from the selected current QSpec checkout, each recompute to its recorded SHA-256 and agree with the IR application candidate for the same body. Separately, complete reader-admissible clause and anchor fixtures retain each exact point variant and name in `CheckedSemanticNodeV2`; an omitted point remains absent. No QSpec schema or vector is copied into IR.
- FR-066-AC-2 (examined): For otherwise equal valid state-clause bodies, absent, `initialization{startup}` and `handler{tick}` points produce distinct application preimages and keys. For otherwise equal valid operation-anchor bodies, absent, `pre{adjust}` and `post{adjust}` points produce distinct structural preimages and keys. Independent canonical derivation agrees with QSpec's four published clause point vectors and the published anchor schema; rekeyed packages have distinct package ids, while retaining a pre-edit key refuses `invalid_package`/`stale-node-key` at the original node's `node_id` before state checks.
- FR-066-AC-3 (examined): The public typed encoder writes a clause point only at `body.operation.member.execution_point` and an anchor point only as the bare fourth `body.members` binding; an absent point adds neither member. A read/write/read round trip retains equal typed bodies, points, node keys and package id. An external consumer constructs these nodes with the public model and encoder without an untyped envelope or private QSL type.
- FR-066-AC-4 (examined): A duplicate `execution_point` JSON key, unknown point kind, missing or extra point member, empty name, wrong fourth binding position, point on another operation-member kind, inapplicable point, mismatched operation name, and unequal or one-sided clause/anchor points each return FR-040's specified first code, cause and locus with no admitted package. Paired earlier wire, graph or stale-key defects win according to FR-040-AC-35.
- FR-066-AC-5 (examined): Inspection finds one identity-bearing body carrier at each QSpec location in `CheckedSemanticNodeV2`, its raw and projection paths, and the writer. No additional top-level point, occurrence-keyed point list, QSpec schema/vector copy, alternate preimage calculation, or fallback inference exists.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

No defect found in this review lens.
