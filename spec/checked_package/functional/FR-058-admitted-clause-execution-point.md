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

When a consumer selects a clause node and authentic `claim` occurrence from
one admitted `CheckedPackageV2`, Contract IR SHALL expose the execution point
that the QSL producer bound to that exact claim, as the existing
`quire_contract_model::ExecutionPoint` value. Contract IR SHALL NOT infer the
point from clause kind, a V2 operation anchor, source text, or caller-supplied
fallback. This accessor is **Planned** under IR-1013 and cannot be implemented
from the current V2 wire; see Dependencies.

## Inputs

- One admitted `CheckedPackageV2`, a `CheckedNodeId` of a selected
  `state`/`state_clause`, and one of that node's authentic `claim`
  `CheckedOccurrence` values. The package's admission is the authority for
  the clause, occurrence and their association; no QSL checked object or
  source bytes are a new accessor input.
- A producer-carried execution-point relation from the accepted QSL binding
  to that exact `(package_id, clause node id, claim occurrence)` tuple. Its
  authoritative V2 representation and admission rule are pending QSL-1082
  and the corresponding QSpec contract; this FR does not define a wire member.

## Outputs

- On success, the existing owned `ExecutionPoint` model value:
  `Initialization { name }`, `Handler { name }`, `Pre { operation }`, or
  `Post { operation }`, retaining the exact `AnchorName` value and variant.
- On failure, `CheckedExecutionPointError` with a stable, closed
  `CheckedExecutionPointRefusal` code: `UnknownClause`, `NotStateClause`,
  `MissingClaimOccurrence`, or `MissingExecutionPoint`. The error carries
  the supplied clause id and claim occurrence. There is no successful
  default, partial value or prose-only error.
- On reader admission failure for a present carrier, the typed
  `CheckedPackageRefusalCode::InvalidExecutionPointCarrier` with one of the
  typed causes `DuplicatePoint`, `NonClaimPoint`, or `IncompatiblePoint` and
  the offending clause/occurrence locus. The QSpec V2 carrier contract must
  assign these cases to its closed diagnostic pairing before implementation.

## Admission and retention

When the QSpec-governed point carrier is available, Contract IR SHALL verify
and retain the accepted point relation during `CheckedPackageV2::read`, keyed
by the admitted package identity, clause node id and claim occurrence. The
reader SHALL retain at most one point per claim. If two entries target one
claim, then the reader SHALL refuse `InvalidExecutionPointCarrier` with
`DuplicatePoint`, whether the points agree or conflict. If an entry targets
a non-claim or a claim of another node, then the reader SHALL refuse
`InvalidExecutionPointCarrier` with `NonClaimPoint`. If an entry's point
variant is incompatible with the clause kind, then the reader SHALL refuse
`InvalidExecutionPointCarrier` with `IncompatiblePoint`: an invariant or
constraint admits Initialization or Handler, a precondition admits Pre, and
a postcondition admits Post. IR SHALL revalidate these facts at admission
rather than trusting the producer assertion. The carrier's wire location,
identity preimage and diagnostic pairing remain QSpec-owned dependencies;
this section assigns the IR reader's independent checks and typed outcomes.

## Behavior

1. If the clause id is absent from the admitted package, then Contract IR SHALL
   refuse `UnknownClause`.
2. If the resolved node is not `state`/`state_clause`, then Contract IR SHALL
   refuse `NotStateClause`.
3. If the supplied occurrence is not an authentic `claim` occurrence of that
   resolved node, then Contract IR SHALL refuse `MissingClaimOccurrence`.
4. If the admitted producer relation carries no accepted execution point for
   that exact package, clause and claim, then Contract IR SHALL refuse
   `MissingExecutionPoint`.
5. When that relation carries an accepted point, Contract IR SHALL return its
   exact model variant and `AnchorName`, including the distinction between
   `Initialization` and `Handler` for claims over one invariant or constraint
   clause node. The accessor SHALL NOT derive that distinction from the
   clause kind or the node's anchor.
6. When resolving a point, Contract IR SHALL read only the
   admission-retained typed relation.
7. When resolving a point, Contract IR SHALL NOT decode V2 body JSON,
   recheck QSL, export bindings, mutate the package, or construct a parallel
   binding authority.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-058-AC-1 | For one QSL-produced, IR-admitted V2 package with accepted bindings at `Initialization { name: startup }`, `Handler { name: tick }`, `Pre { operation: adjust }`, and `Post { operation: adjust }`, the public accessor returns each exact `ExecutionPoint` variant and `AnchorName` for the selected clause id and authentic claim occurrence. A fresh producer result and the authoritative QSpec carrier are required; a synthetic package alone gives no producer-origin credit. | Test |
| FR-058-AC-2 | For each of an invariant and a constraint node, two authentic claim occurrences share one node and body anchor but bind respectively to initialization `startup` and handler `tick`; the accessor returns distinct exact points by claim. Replacing `tick` with another handler name changes only that claim's result. No clause-kind or anchor-derived default returns either value. The invariant and constraint have different node ids, as QSpec FR-341-AC-13 requires. | Test |
| FR-058-AC-3 | An absent clause id returns `UnknownClause`; a present non-state-clause id returns `NotStateClause`; a wrong-role, wrong-ordinal or other-node claim returns `MissingClaimOccurrence`. The first applicable refusal is returned in that order, with the supplied clause id and occurrence and no point. | Test |
| FR-058-AC-4 | A selected state clause with an authentic claim but no accepted producer point returns `MissingExecutionPoint` and no point. A point associated with another package id, clause id or claim occurrence cannot satisfy this lookup; it also returns `MissingExecutionPoint` when the selected tuple has no point. | Test |
| FR-058-AC-5 | An external CG-style Rust consumer calls the public accessor using only `&CheckedPackageV2`, `&CheckedNodeId` and `&CheckedOccurrence`, exhaustively distinguishes the typed refusal codes and receives `quire_contract_model::ExecutionPoint` without importing QSL internals, inspecting V2 body JSON, or calling `export_bindings`. Repeated calls and a clone return equal results without changing package equality or identity. | Test |
| FR-058-AC-6 | A public-consumer compile control accepts only the admitted package, clause id and claim occurrence and cannot supply a caller point or QSL binding. An IR reader control consumes the authoritative QSpec carrier from a fresh QSL package, retains the point after source bytes are dropped, and a source check finds no IR-owned QSL binding copy or alternate compatibility path. | Test |
| FR-058-AC-7 | Two equal or conflicting point entries for one claim refuse reader admission with `InvalidExecutionPointCarrier`/`DuplicatePoint`; an entry for a non-claim or another node's claim refuses with `InvalidExecutionPointCarrier`/`NonClaimPoint`. Each result names the offending clause/occurrence locus, returns no admitted package and creates no first-wins point. | Test |
| FR-058-AC-8 | For an invariant or constraint, `Pre` and `Post` point entries refuse reader admission with `InvalidExecutionPointCarrier`/`IncompatiblePoint`; for a precondition or postcondition, a wrong variant refuses likewise. Valid Initialization and Handler points for both invariant and constraint, valid Pre for a precondition and valid Post for a postcondition admit unchanged. | Test |

## Dependencies

QSL-1082 must define how its accepted binding's point reaches the V2
`state_clause`/`operation_anchor` graph, with an occurrence-specific association
and QSpec-owned wire and diagnostic-pairing rules. On current main, QSL's modern
`qsl-package::export_bindings` returns `BoundItem` abstraction bindings with
no execution point, while its native `ClauseBinding.execution_point` is a
separate native-checking path. QSpec FR-341's V2 clause body has kind,
parameters, anchor and condition; FR-342's operation anchor has context,
operation and frame. The V2 node, occurrence and source-map records retain
no point. Consequently an IR accessor cannot reconstruct the accepted
`Initialization` versus `Handler` name from the present bytes. QSL owns the
binding decision and QSpec owns its V2 carrier; IR owns the typed model
`ExecutionPoint` and this admitted accessor. The driver alone invokes
`export_bindings` under ADR-017 AR-4; CG receives no new point input.
