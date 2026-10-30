---
id: FR-058
title: "Expose the admitted execution point of a V2 clause claim"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-341
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-342
    type: depends_on
  - target: ix://agent-ix/quire-contract-codegen/FR-011
    type: references
---
# FR-058: Expose the admitted execution point of a V2 clause claim

## Description

When a consumer selects a clause node and owner-bound authentic `claim`
occurrence from one admitted `CheckedPackageV2`, Contract IR SHALL expose
the exact optional execution point admitted from that clause node's QSpec FR-341 application
member as the public `quire_contract_model::ExecutionPoint` value. The node
body is the carrier; its point is shared by all authentic claim occurrences of
that node. Contract IR SHALL NOT infer the point from clause kind, an operation
anchor, source text or a caller-supplied fallback. IR-1040 defines and admits
the wire carrier; the accessor implements the public IR-1013 reader seam.

`ExecutionPoint` is the sole public four-variant point type for authored IR,
checked-package V2 and QSL callers. Its names are nonempty strings, including
QSpec-admitted V2 names that are not valid authored `AnchorName` identifiers.
Authored constructors and wire decoders SHALL retain FR-011's `AnchorName`
validation and refusal before admitting an authored point. The V2 accessor SHALL
NOT convert a point name through `AnchorName` or reject an otherwise
admitted package because authored validation would fail. There SHALL be no
public `CheckedExecutionPoint` type, conversion bridge or second binding
authority.

## Inputs

- One admitted `CheckedPackageV2`, the `CheckedNodeId` of a selected
  `state`/`state_clause`, and one owner-bound claim key represented by a pair
  of `&CheckedNodeId` (claim owner) and `&CheckedOccurrence` (claim role and
  ordinal). The public accessor takes the pair as one third argument,
  `claim: (&CheckedNodeId, &CheckedOccurrence)`, after the package receiver
  and selected clause id. It SHALL NOT expose a second signature accepting
  an unbound occurrence. The package's admission is the authority for the selected node and its
  occurrences; no QSL checked object or source bytes are a new accessor
  input. The owner is required because role and ordinal alone can have the
  same values on different nodes.
- The optional, typed `operation.member.execution_point` retained from that
  admitted clause node by FR-040 and QSpec FR-341/FR-322. There is no
  occurrence-keyed sidecar, point-entry list or second binding authority.

## Outputs

- On success, the owned public `ExecutionPoint` model value:
  `Initialization { name }`, `Handler { name }`, `Pre { operation }`, or
  `Post { operation }`, retaining the exact nonempty name and variant.
- On failure, `CheckedExecutionPointError` with a stable, closed
  `CheckedExecutionPointRefusal` code: `UnknownClause`, `NotStateClause`,
  `MissingClaimOccurrence`, or `MissingExecutionPoint`. The error carries
  the supplied clause id, claim owner and claim occurrence. There is no
  successful default, partial value or prose-only error.
- Reader admission of a malformed or inapplicable body point follows
  FR-040-AC-31 through FR-040-AC-35 and QSpec FR-341/FR-342: schema-level
  `invalid_semantic_graph` or semantic `invalid_model_binding`/
  `malformed-declaration` at their specified loci. The earlier planned
  `InvalidExecutionPointCarrier`/`DuplicatePoint`/`NonClaimPoint`/
  `IncompatiblePoint` names described a separate occurrence-keyed carrier
  that QSpec #449 did not adopt; they SHALL NOT be added to the reader's
  closed refusal catalog for the node-body carrier.

## Admission and retention

Contract IR SHALL decode the closed QSpec FR-322 `ExecutionPoint` from the
optional clause application member during `CheckedPackageV2::read`, check
FR-040's applicability and anchor agreement, and retain the typed point with
the admitted clause node. The point contributes to that node's rederived
identity and package identity. Two equal or conflicting `execution_point`
JSON members in one object are a general duplicate-key wire defect, not two
carrier entries. A present point on a non-state-clause operation member fails
the closed operation-member shape; a non-claim occurrence fails the existing
claim-role admission or accessor authenticity check. A precondition and a
postcondition each admit either `pre` or `post` when QSpec's operation-name
and anchor-point agreement hold; a narrower consumer policy may reject one
after admission, but does not change wire admission. Invariant and constraint
points admit `initialization` or `handler`. The reader returns no package on
its first applicable body/schema, identity, state or operation refusal.

## Behavior

1. If the clause id is absent from the admitted package, then Contract IR SHALL
   refuse `UnknownClause`.
2. If the resolved node is not `state`/`state_clause`, then Contract IR SHALL
   refuse `NotStateClause`.
3. If the supplied claim owner differs from the resolved clause node id, or
   the supplied occurrence is not an authentic `claim` occurrence of that
   resolved node, then Contract IR SHALL refuse `MissingClaimOccurrence`.
   An occurrence with the same role and ordinal on another node does not
   satisfy this lookup when paired with that other node's id. The owner
   comparison occurs before the occurrence membership check; neither may
   alter the `UnknownClause` or `NotStateClause` precedence.
4. If the admitted clause node carries no accepted execution point, then Contract IR SHALL refuse
   `MissingExecutionPoint`.
5. When that node carries an accepted point, Contract IR SHALL return its
   exact V2 model variant and name, including the distinction between
   `Initialization` and `Handler` on distinct point-qualified invariant or
   constraint clause nodes. All authentic claims of one node return its one
   retained point. The accessor SHALL NOT derive that distinction from the
   clause kind or the node's anchor.
6. When resolving a point, Contract IR SHALL read only the
   admission-retained typed clause-node member.
7. When resolving a point, Contract IR SHALL NOT decode V2 body JSON,
   recheck QSL, export bindings, mutate the package, or construct a parallel
   binding authority.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-058-AC-1 | For one QSL-produced, IR-admitted V2 package with accepted bindings at `Initialization { name: startup }`, `Handler { name: tick }`, `Pre { operation: adjust }`, and `Post { operation: adjust }`, the public accessor returns each exact `ExecutionPoint` variant and nonempty QSpec point name for the selected clause id and authentic claim occurrence. A fresh producer result and the authoritative QSpec carrier are required; a synthetic package alone gives no producer-origin credit. | Test |
| FR-058-AC-2 | SUPERSEDED by FR-058-AC-9 under QSpec #449 / IR-1040. Original planned text: For each of an invariant and a constraint node, two authentic claim occurrences share one node and body anchor but bind respectively to initialization `startup` and handler `tick`; the accessor returns distinct exact points by claim. Replacing `tick` with another handler name changes only that claim's result. No clause-kind or anchor-derived default returns either value. The invariant and constraint have different node ids, as QSpec FR-341-AC-13 requires. The original same-node/different-point case is infeasible because the point now contributes to node identity; no implementation credit is claimed. | Test |
| FR-058-AC-3 | An absent clause id returns `UnknownClause`; a present non-state-clause id returns `NotStateClause`; a wrong-role, wrong-ordinal or other-node claim returns `MissingClaimOccurrence`. The first applicable refusal is returned in that order, with the supplied clause id and occurrence and no point. | Test |
| FR-058-AC-4 | A selected state clause with an authentic claim but no accepted producer point returns `MissingExecutionPoint` and no point. A point on another package or clause node cannot satisfy this lookup; the selected unpointed node returns `MissingExecutionPoint` for every authentic claim. A different occurrence of the same pointed node reads the same point. | Test |
| FR-058-AC-5 | An external CG-style Rust consumer calls the public accessor using only `&CheckedPackageV2`, `&CheckedNodeId` and `&CheckedOccurrence`, exhaustively distinguishes the typed refusal codes and receives `quire_contract_model::ExecutionPoint` without importing QSL internals, inspecting V2 body JSON, or calling `export_bindings`. Repeated calls and a clone return equal results without changing package equality or identity. | Test |
| FR-058-AC-6 | A public-consumer compile control accepts only the admitted package, clause id and claim occurrence and cannot supply a caller point or QSL binding. An IR reader control consumes the authoritative QSpec node-body member from a fresh QSL package, retains the point after source bytes are dropped, and a source check finds no IR-owned QSL binding copy or alternate compatibility path. | Test |
| FR-058-AC-7 | SUPERSEDED by FR-058-AC-10 under QSpec #449 / IR-1040. Original planned text: Two equal or conflicting point entries for one claim refuse reader admission with `InvalidExecutionPointCarrier`/`DuplicatePoint`; an entry for a non-claim or another node's claim refuses with `InvalidExecutionPointCarrier`/`NonClaimPoint`. Each result names the offending clause/occurrence locus, returns no admitted package and creates no first-wins point. QSpec #449 has no point-entry list or occurrence-keyed carrier, so these exact cases are infeasible; no implementation credit is claimed. | Test |
| FR-058-AC-8 | For an invariant or constraint, a well-formed `Pre` or `Post` point refuses reader admission as `invalid_model_binding`/`malformed-declaration` at the clause node under FR-040. Valid `Initialization` and `Handler` points for both invariant and constraint admit. A precondition or postcondition admits either `Pre` or `Post` if the operation name and referenced anchor point agree, as QSpec FR-341-AC-14 requires; an `Initialization` or `Handler` point refuses `invalid_model_binding`/`malformed-declaration` at the clause node. An accessor returns the exact admitted variant without imposing a narrower consumer policy. | Test |
| FR-058-AC-9 | PLANNED/UNRUN (IR-1141). Separate invariant or constraint clause nodes with equal condition and anchor but different admitted initialization or handler points retain their exact distinct point values in their complete bodies. Authentic claims return the point of the selected node; changing only the point changes the accessor result after read/write/read. No kind- or anchor-derived default supplies either value, and a reader-admitted nonempty name outside AnchorName grammar returns unchanged. | Test |
| FR-058-AC-10 | PLANNED (QSpec #449 / IR-1040). A duplicate `execution_point` JSON member refuses `duplicate_member` with no cause at the later member in strict wire admission; malformed or extra point members and a point on the wrong operation member fail the closed QSpec/FR-040 shape with no admitted package or first-wins value. A non-claim or other-node occurrence cannot retrieve a point and returns `MissingClaimOccurrence`; two authentic claims of one node return its one retained point. Exact wire and state refusal code/cause/locus follow FR-040, preserving AC-7's uniqueness and wrong-target assurances on the actual member carrier. | Test |
| FR-058-AC-11 | The crate root exports exactly one public four-variant point type, `ExecutionPoint`. An authored clause rejects `tick now` and an empty point name with FR-011's `invalid_identifier`, including when the caller builds the canonical point directly; a V2 reader admits `tick now` and the accessor returns that exact variant and name to a CG-style external consumer without inspecting JSON. QSL authored construction uses the same validated name rule and migrates to the canonical type after taking the IR dependency; no public `CheckedExecutionPoint`, conversion bridge, fallback or copied QSL point type remains. | Test |

## Dependencies

QSpec #449 (FR-322-AC-131, FR-341-AC-14 and FR-342-AC-10) fixes the
optional node-body carriers and their identities. IR-1040/FR-040 owns the
reader's typed admission and retention; IR-1013 implements this accessor
against the admitted typed clause node. QSL owns the checked source binding
and its producer conformance. Contract IR does not copy QSL bindings, add an
occurrence-keyed sidecar, or reconstruct a point from source text. The driver
alone invokes `export_bindings` under ADR-017 AR-4; CG receives no new point
input. Existing authored-contract callers of `ExecutionPoint` and `AnchorName`
keep their FR-011 validation; a CG V2 consumer reads the same canonical
`ExecutionPoint` from this accessor without a conversion. QSL migrates its
authored construction and checking call sites after taking the IR dependency,
including the additional authored route introduced by QSL-1183; that
producer-origin work remains necessary for AC-1 and AC-6.
CG's V2 replay selection already carries node and occurrence separately;
the CG consumer passes its selected clause id and the claim's owner-bound
pair directly. QSL's V2 emitter already groups occurrences under their
node id; it supplies the same pair if it calls this accessor. Neither
consumer currently calls the planned accessor, so this input change requires
no compatibility overload or migration path.
