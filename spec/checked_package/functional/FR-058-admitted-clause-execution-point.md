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

## Behavior

1. When the accessor receives a clause id, Contract IR SHALL resolve it in
   the same admitted package and refuse `UnknownClause` if absent.
2. If the resolved node is not `state`/`state_clause`, then Contract IR SHALL
   refuse `NotStateClause`.
3. If the supplied occurrence is not an authentic `claim` occurrence of that
   resolved node, then Contract IR SHALL refuse `MissingClaimOccurrence`.
4. If the admitted producer relation carries no accepted execution point for
   that exact package, clause and claim, then Contract IR SHALL refuse
   `MissingExecutionPoint`.
5. When that relation carries an accepted point, Contract IR SHALL return its
   exact model variant and `AnchorName`, including the distinction between
   `Initialization` and `Handler` for two claims over the same invariant
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
| FR-058-AC-2 | Two authentic claim occurrences sharing one invariant node but bound respectively to initialization `startup` and handler `tick` return distinct points even though the node id, clause kind and body anchor are equal. Replacing `tick` with another handler name changes that claim's result. No clause-kind or anchor-derived default returns either value. | Test |
| FR-058-AC-3 | An absent clause id returns `UnknownClause`; a present non-state-clause id returns `NotStateClause`; a wrong-role, wrong-ordinal or other-node claim returns `MissingClaimOccurrence`. The first applicable refusal is returned in that order, with the supplied clause id and occurrence and no point. | Test |
| FR-058-AC-4 | A selected state clause with an authentic claim but no accepted producer point returns `MissingExecutionPoint` and no point. A point associated with another package id, clause id or claim occurrence cannot satisfy this lookup; it also returns `MissingExecutionPoint` when the selected tuple has no point. | Test |
| FR-058-AC-5 | An external CG-style Rust consumer calls the public accessor using only `&CheckedPackageV2`, `&CheckedNodeId` and `&CheckedOccurrence`, exhaustively distinguishes the typed refusal codes and receives `quire_contract_model::ExecutionPoint` without importing QSL internals, inspecting V2 body JSON, or calling `export_bindings`. Repeated calls and a clone return equal results without changing package equality or identity. | Test |
| FR-058-AC-6 | Source inspection finds one QSpec-governed producer-to-V2 point carrier admitted and retained by IR, no IR-owned copy of QSL binding types, no new caller point or compatibility path, and no claimed implementation while the QSL-1082/QSpec carrier is absent. | Inspection |

## Dependencies

QSL-1082 must define how its accepted binding's point reaches the V2
`state_clause`/`operation_anchor` graph, with an occurrence-specific association
and QSpec-owned wire and reader rules. On current main, QSL's modern
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
