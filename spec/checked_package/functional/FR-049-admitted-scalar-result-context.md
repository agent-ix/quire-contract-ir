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

PLANNED/UNRUN (IR-769; amendment pending QSL-710). For one QSL driver
requirement over an exact scalar expression occurrence, Contract IR SHALL
lower the expression and its admitted result-context roots together before it
exposes typed reverse consumer context. The result-context roots are IR
lowering requests, not additional QSL requirements, routed items or claims.
Contract IR SHALL derive the context from the one admitted FR-330 package and
its authoritative body and type validation; the consumer SHALL NOT search or
lower the original checked graph again.

The previously specified `lower([expression])` positive is false: ordinary
[FR-035](./FR-035-complete-v1-contract-package-lowering.md) lowering exports
only the expression's forward closure. An enclosing `numeric.narrow` and its
result bound are absent from that package unless the narrow is separately
named as a lowering root. This amendment does not change that ordinary
lowering behavior.

## Inputs

At the IR lowering boundary: one admitted `CheckedPackageV2`; one QSL driver
requirement's exact occurrence and expression `CheckedNodeId`; its
QSL-authoritative enclosing conversion key or keys, when present; the selected
lowering profile and limits. QSL-710 owns the not-yet-reviewed request field,
accessor, occurrence scope and cardinality. An IR caller SHALL NOT infer a
conversion key by a CG graph walk or substitute a result-bound type key for a
conversion key.

At the result-context query: the one admitted `ContractPackageV1` emitted from
that lowering, the same expression and occurrence context, and the expected
scalar bound form. The query does not accept the original checked graph.

## Outputs

A typed result distinguishes `Lowered`, the exact non-lowered request
disposition, and `Absent`, as [FR-035](./FR-035-complete-v1-contract-package-lowering.md)
does. A lowered result includes the existing forward reached closure, the
represented local body-reference consumers of the exact expression, and
exactly one result-context outcome:

- `DirectBounded`: the requested expression's own admitted `semantic_type` is
  a readable bounded domain of the expected form. Its typed key, admitted node
  and source occurrences are the result bound; reverse consumers do not select
  a different bound.
- `Unnarrowed`: a scalar-typed expression has no conversion-shaped candidate
  in the complete represented consumer set for this requirement occurrence.
  Other consumers alone do not supply a reverse result bound.
- `Narrowed`: one distinct admitted bounded-domain key of the expected form,
  its exact admitted node and source occurrences, and the valid conversion
  key or keys with their source occurrences. Repeated valid conversions to
  the same bound do not create distinct bounds.
- `MissingBound`: the direct bounded type has the wrong form, or at least one
  conversion-shaped candidate has no readable admitted bounded-domain target
  of the expected form over the expression's scalar type.
- `AmbiguousBound`: valid conversion-shaped candidates name distinct target
  bounds, or any conversion-shaped candidate and another local body-reference
  consumer both use the expression.

An incomplete context-root selection, inconsistent admitted graph, failed
shared body validation, or exhausted declared work limit returns a typed error
without a partial result or guessed bound. A non-lowered context root likewise
cannot silently supply a bound. The exact error shape for context-root
incompleteness remains pending the reviewed QSL-710 request shape.

The variant and typed keys, not formatted text, define the result.
`Unnarrowed` is context absence, not a fabricated bound; a consumer may then
apply its separately specified forward-closure result rule.

## Behavior

### Contextual lowering and authority

For one QSL driver requirement, Contract IR SHALL distinguish its sole
user-facing expression item from the extra context roots used to represent its
result. Contract IR SHALL validate the supplied conversion keys against the
admitted checked graph, the exact expression key and the requirement's source
occurrence. It SHALL determine the complete local body-reference consumer set
for that scoped expression from the admitted checked graph using its existing
body validator, and lower the expression and those consumer keys in one call.
For the QSL `a + 1` positive, the explicit lowered root set is the add and its
enclosing `numeric.narrow`; the latter is context-only. A missing supplied
conversion or an unrepresented local consumer that could change the result
SHALL refuse the contextual result, rather than treating an add-only package
as complete. The selected dependency boundary stays as in
[FR-035](./FR-035-complete-v1-contract-package-lowering.md).

The output SHALL remain one ordinary six-field FR-330 package. Each lowered
root, including a context-only root, SHALL have the exact
[FR-035](./FR-035-complete-v1-contract-package-lowering.md) feature-accounting
record and source correspondence. Adding a context root legitimately changes
the represented graph, accounting and canonical package identity; no new wire
field or separate lowering product is introduced. Neither the context root nor
its accounting record SHALL create a second QSL requirement, routed item,
claim-map entry or proof obligation.

### Admitted result-context query

Contract IR SHALL inspect all represented local nodes of the admitted FR-330
package for validated body references to the exact expression key. A forward
`reached_closure(expression)` walk alone is insufficient and SHALL retain its
existing membership: adding a separately lowered narrow does not make it a
forward descendant of the add. The query SHALL use the validated contextual
selection for the QSL requirement occurrence before claiming that its
represented consumer set is complete. An arbitrary reader-admitted FR-330
package without that selection may expose represented consumers, but SHALL
NOT claim an authoritative result bound from their apparent absence.

For a lowered expression typed by a scalar type, Contract IR SHALL inspect
the scoped represented consumers whose validated bodies contain a `reference`
term targeting the exact requested key. A conversion-shaped candidate is an `expression` of form
`conversion` whose `convert` application has operation identity exactly
`quire.op.numeric.narrow` and one argument, a direct reference to that key.
Contract IR SHALL classify candidates from that shape before checking their
result targets. A candidate is valid only if its result type names a readable
admitted bounded-domain target of the expected form whose semantic type equals
the requested expression's scalar type. Contract IR SHALL classify these
references using its admitted operation, body and type knowledge. A reference
that is not conversion-shaped is another consumer, never an inferred narrowing.
Other graph edges, including `semantic_type`, `dependencies`, and
source-map occurrences, do not themselves count as consumers.

Where the requested expression is itself typed by an admitted bounded domain,
Contract IR SHALL return `DirectBounded` with that direct type as its result
bound when it has the expected readable form, regardless of reverse consumers.
When that direct type has the wrong or unreadable form, Contract IR SHALL return
`MissingBound` with the direct type key; a consumer's target SHALL NOT replace
the direct type. For a scalar-typed expression, Contract IR SHALL apply this
precedence after collecting local body-reference consumers: no
conversion-shaped candidate returns `Unnarrowed`; any candidate together with
another non-candidate consumer returns `AmbiguousBound`; otherwise any
candidate with an invalid target returns `MissingBound`; otherwise two or more
distinct valid target keys return `AmbiguousBound`; otherwise one distinct
valid target returns `Narrowed`. Thus one valid and one invalid candidate
return `MissingBound`, including when valid candidates also name distinct
bounds. A wrong-form, unreadable or scalar-type-incompatible target is invalid;
none can be silently omitted to choose a valid sibling. None of these outcomes
assigns a result bound from an operand's bound.

The result SHALL retain the exact selected source package identity, lowering
profile, requirement occurrence, requested key, conversion keys, bound key and source-map occurrences
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

## Pending upstream contract

QSL-710 has not yet fixed the driver-facing conversion field and accessor, its
no-conversion and multiple-conversion representation, or its relation to a
requirement occurrence. Those details SHALL be reconciled here before this
amendment is published. In particular, QSL FR-075 permits two requirement
occurrences of one expression key with different result bounds; a global
expression-key-only reverse scan cannot select either occurrence's bound.

The reviewed upstream contract must settle three concrete joins before this
draft can be final:

1. Whether one requirement exposes zero-or-one enclosing conversion key or a
   typed set, and how an absent or multiply matched conversion is reported.
2. Which QSL occurrence identifies a conversion as the context of this exact
   requirement when another occurrence shares the expression key, and which
   other local consumers belong to that scoped completeness check.
3. How IR carries the validated occurrence-scoped context through an FR-330
   strict read without a seventh wire field, so a read-only query can
   distinguish a complete contextual package from an add-only package.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-049-AC-1 | For the reader-admitted QSL source `function f using v(a: Int[0, 9]): Int[0, 10] pure { a + 1 }`, the one QSL driver requirement names the add expression and its authoritative enclosing narrow key. One IR lowering over explicit roots `[add, narrow]` represents the narrow and result `Int[0, 10]`; the add's forward closure still contains only its original six nodes and operand `Int[0, 9]`. The admitted result-context query returns `Narrowed` with result `[0, 10]` for that requirement occurrence, checked against the independent QSL source intervals. Plain `lower([add])` remains an admitted six-field package with no represented narrow or `[0, 10]`, and cannot return the same authoritative context. | Test |
| FR-049-AC-2 | An expression directly typed by a readable admitted bounded domain of the expected form returns `DirectBounded` with that exact type key and sources, both with and without a represented reverse conversion; a wrong-form direct type returns `MissingBound` with its own key. A scalar-typed expression with no conversion-shaped candidate in its complete scoped consumer set returns `Unnarrowed`; one valid candidate returns `Narrowed`, while duplicate valid candidates to one target retain one bound and all conversion keys in typed-key order. | Test |
| FR-049-AC-3 | For one requirement occurrence, two valid candidates to distinct target bounds, or any candidate with another non-candidate local body-reference consumer, return `AmbiguousBound`. A sole wrong-form, unreadable or type-incompatible candidate returns `MissingBound`; one valid and one invalid candidate also return `MissingBound`, even beside distinct valid targets. Omitting a local consumer from the contextual root set cannot turn either refusal into `Narrowed` or `Unnarrowed`. | Test |
| FR-049-AC-4 | A non-lowered expression root returns its exact disposition, and an unrequested or absent expression key returns typed absence. A context-only root receives its own FR-035 accounting record but no QSL routed item or claim. An incomplete context selection, non-lowered required context root, inconsistent post-admission edge or exhausted work limit returns typed refusal/error without partial result context. | Test |
| FR-049-AC-5 | The result identifies the exact requirement occurrence, expression key, source package and lowering profile and returns admitted source-map occurrences for expression, conversion and result bound. Two QSL requirements sharing one expression key but carrying different result bounds remain distinguishable by their occurrences and authoritative conversion selections. A selected external reference retains its selected package/key pair; an identically keyed local node cannot satisfy it, and an unavailable selected package or target is refused at reader admission. | Test |
| FR-049-AC-6 | Contextual lowering emits exactly the existing six FR-330 fields, with one accounting record per expression or context root; adding a context root changes the graph, accounting and canonical identity as FR-035 requires. A read-only context query mutates none of those fields or the identity, and the add's forward closure does not grow to include its reverse consumer. CG consumes the IR result without an original checked graph or second lowering pass. | Test |

## Dependencies

[FR-035](./FR-035-complete-v1-contract-package-lowering.md) owns the admitted
six-field package and forward reached view.
[FR-038](./FR-038-consume-checked-package-v2.md) owns checked-package
admission and the operation/body rules. QSpec FR-330 owns the published wire.
Codegen's scalar result consumer is IR-744 and remains outside this IR-owned
requirement.
