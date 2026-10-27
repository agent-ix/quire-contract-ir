---
id: SR-610009
title: "IR-1038 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-066-retain-identity-bound-execution-points.md; spec/spec.md"
review_set: subset
---

## Summary

Ticket IR-1038. Frozen PR #596 at 6bef91d62a0279565097cc53dca00a15e06cae48. Reviewed FR-066-AC-1 through AC-5 and the master index against current QSpec, FR-040 and the public model.

## Examined units

- FR-066-AC-1 (examined): The four published QSpec `node-identity-vectors.json` execution-point application preimages, read directly from the selected current QSpec checkout, each recompute to its recorded SHA-256 and agree with the IR application candidate for the same body. Separately, complete reader-admissible clause and anchor fixtures retain each exact point variant and name in `CheckedSemanticNodeV2`; an omitted point remains absent. No QSpec schema or vector is copied into IR.
- FR-066-AC-2 (examined): For otherwise equal valid state-clause bodies, absent, `initialization{startup}` and `handler{tick}` points produce distinct application preimages and keys. For otherwise equal valid operation-anchor bodies, absent, `pre{adjust}` and `post{adjust}` points produce distinct structural preimages and keys. Independent canonical derivation agrees with QSpec's four published clause point vectors and the published anchor schema; rekeyed packages have distinct package ids, while retaining a pre-edit key refuses `invalid_package`/`stale-node-key` at the original node's `node_id` before state checks.
- FR-066-AC-3 (examined): The public typed encoder writes a clause point only at `body.operation.member.execution_point` and an anchor point only as the bare fourth `body.members` binding; an absent point adds neither member. A read/write/read round trip retains equal typed bodies, points, node keys and package id. An external consumer constructs these nodes with the public model and encoder without an untyped envelope or private QSL type.
- FR-066-AC-4 (examined): A duplicate `execution_point` JSON key, unknown point kind, missing or extra point member, empty name, wrong fourth binding position, point on another operation-member kind, inapplicable point, mismatched operation name, and unequal or one-sided clause/anchor points each return FR-040's specified first code, cause and locus with no admitted package. Paired earlier wire, graph or stale-key defects win according to FR-040-AC-35.
- FR-066-AC-5 (examined): Inspection finds one identity-bearing body carrier at each QSpec location in `CheckedSemanticNodeV2`, its raw and projection paths, and the writer. No additional top-level point, occurrence-keyed point list, QSpec schema/vector copy, alternate preimage calculation, or fallback inference exists.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public clause point may remain an untyped operation Value — The public model currently represents CheckedTerm::Application.operation as serde_json::Value. AC-3 requires construction without an untyped envelope but does not require an Option<ExecutionPoint> or equivalent typed member within the application. An implementation can leave the clause point in Value while adding a typed bare-anchor carrier, passing the stated round trip yet failing the requested typed public representation for both locations. Require typed point access for both clause and anchor bodies through public model constructors or fields. | spec/checked_package/functional/FR-066-retain-identity-bound-execution-points.md:89 |

## Verdict

Findings require a fix before handoff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-066-AC-3 at head 97aaf3ae37915e12e10b46b0293bd6d9e94a79e7 now states the typed public or strict reader oracle verified in the fix delta. |
