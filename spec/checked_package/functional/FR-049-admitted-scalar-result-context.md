---
id: FR-049
title: "Expose admitted scalar result context for an exact requested expression"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-specification/FR-330
    type: references
---
# FR-049: Expose admitted scalar result context for an exact requested expression

## Description

PLANNED/UNRUN (IR-769). When an admitted `ContractPackageV1` is queried for one
exact requested scalar expression, Contract IR SHALL expose typed reverse
consumer context sufficient to identify its result bound without requiring the
consumer to search a checked-package graph. This context is a read-only view of
the same admitted FR-330 package and its authoritative IR body and type
validation. It is not another lowering product or a new semantic identity.

## Inputs

An admitted `ContractPackageV1`, the exact `CheckedNodeId` of one requested
expression, and the expected scalar bound form (`integer_range` or another
supported scalar range form). The caller supplies no original
`CheckedPackageV2`, source graph, or inferred result bound.

## Outputs

A typed result distinguishes `Lowered`, the exact non-lowered request
disposition, and `Absent`, as [FR-035](./FR-035-complete-v1-contract-package-lowering.md)
does. A lowered result includes the existing reached closure and exactly one
reverse-context outcome:

- `Unnarrowed`: no local body reference consumes the expression as a
  qualifying numeric narrowing conversion.
- `Narrowed`: one distinct admitted bounded-domain key of the expected form,
  its exact admitted node and source occurrences, and the qualifying conversion
  key or keys with their source occurrences. Repeated qualifying conversions to
  the same bound do not create distinct bounds.
- `MissingBound`: a qualifying conversion exists but its admitted target is not
  a readable bounded domain of the expected form over the expression's scalar
  type.
- `AmbiguousBound`: qualifying conversions name distinct target bounds, or a
  qualifying conversion and another local body-reference consumer both use the
  expression.

An inconsistent admitted graph, failed shared body validation, or exhausted
declared work limit returns the existing typed view error without a partial
closure or reverse context. The variant and typed keys, not formatted text,
define the result. `Unnarrowed` is context absence, not a fabricated bound; a
consumer may then apply its separately specified forward-closure result rule.

## Behavior

For a lowered expression typed by a scalar type, Contract IR SHALL inspect
local consumers whose validated bodies contain a `reference` term targeting
the exact requested key. A qualifying consumer is an `expression` of form
`conversion` whose `convert` application has operation identity exactly
`quire.op.numeric.narrow` and one argument, a direct reference to that key.
Its result type SHALL name the admitted bounded-domain target. The bound's
semantic type SHALL equal the requested expression's scalar type. Contract IR SHALL
classify these references using its admitted operation, body and type knowledge;
references; a nonqualifying reference is another consumer, never an inferred
narrowing. Other graph edges, including `semantic_type`, `dependencies`, and
source-map occurrences, do not themselves count as consumers.

Where the requested expression is itself typed by an admitted bounded domain,
Contract IR SHALL give that direct result type precedence. The reverse context SHALL NOT
replace its bound. For a scalar-typed expression with qualifying conversions,
Contract IR SHALL return `Narrowed` only for one distinct compatible target
bound and no other local body-reference consumer. A wrong-form, unreadable or
type-incompatible target SHALL return `MissingBound`; mixed consumers or
distinct compatible bounds SHALL return `AmbiguousBound`. Ambiguity SHALL take
precedence over selecting any one conversion's bound. In the absence of a
qualifying conversion, Contract IR SHALL return `Unnarrowed`, even when other
consumers exist. This does not assign a result bound from an operand's bound.

The result SHALL retain the exact selected source package identity, lowering
profile, requested key, conversion keys, bound key and source-map occurrences
from the admitted package. A `dependency_reference` SHALL remain the selected
external package identity and target key described by
[FR-035](./FR-035-complete-v1-contract-package-lowering.md); it SHALL NOT be
resolved through an identically keyed local node or treated as a local reverse
consumer. A selected external target that is needed for an exact result SHALL
remain at that admitted package boundary. The reader SHALL refuse an unavailable
selected package or target before constructing an admitted value. The API SHALL NOT claim a
local result bound from an external target it has not resolved under that
selection.

Contract IR SHALL derive the result from the six existing FR-330 fields and its
admitted IR knowledge. Querying it SHALL leave those fields, their canonical identity,
the [FR-035](./FR-035-complete-v1-contract-package-lowering.md) forward
`reached_closure` membership, and the reader's admission and first-refusal
order unchanged. No new wire field, copied body grammar, per-node `ir_id`, or
second codegen graph traversal is part of this contract.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-049-AC-1 | A reader-admitted QSL package from `function f using v(a: Int[0, 9]): Int[0, 10] pure { a + 1 }`, requested by the exact `quire.op.integer.add` expression key, returns a lowered context whose qualifying `numeric.narrow` conversion identifies result bound `Int[0, 10]`; the operand's `Int[0, 9]` bound remains separately present and is not returned as the result bound. Independent QSL source and range assertions establish both intervals. | Test |
| FR-049-AC-2 | An admitted scalar expression with no qualifying local conversion returns `Unnarrowed` with no result-bound key; one valid conversion returns `Narrowed` with the exact conversion and target keys, while duplicate valid conversions to that same target retain one distinct bound and all conversion keys in typed-key order. | Test |
| FR-049-AC-3 | Two qualifying conversions to distinct target bounds, or one qualifying conversion plus another local body-reference consumer, return `AmbiguousBound` without choosing an operand bound or any one target; a sole conversion with wrong bound form, unreadable target or mismatched scalar type returns `MissingBound`, and an otherwise nonqualifying reference cannot be promoted to a narrowing. | Test |
| FR-049-AC-4 | A non-lowered requested key returns its exact typed disposition; an unrequested or absent key returns typed absence, including when the key occurs in another root's closure. An inconsistent post-admission body/source edge or exhausted work limit returns the existing typed view error with no partial context. | Test |
| FR-049-AC-5 | The result identifies the exact requested key, source package and lowering profile, and returns admitted source-map occurrences for its expression, conversion and bound. A selected external reference retains its selected package/key pair; an identically keyed local node cannot satisfy it, and an unavailable selected package or target is refused at reader admission rather than yielding a guessed local bound. | Test |
| FR-049-AC-6 | Querying all outcomes leaves the six FR-330 wire fields and canonical identity byte-for-byte unchanged; the preexisting `reached_closure` key set does not grow to include a reverse consumer, and CG can consume the typed context without an original checked graph or a second lowering pass. | Test |

## Dependencies

[FR-035](./FR-035-complete-v1-contract-package-lowering.md) owns the admitted
six-field package and forward reached view.
[FR-038](./FR-038-consume-checked-package-v2.md) owns checked-package
admission and the operation/body rules. QSpec FR-330 owns the published wire.
Codegen's scalar result consumer is IR-744 and remains outside this IR-owned
requirement.
