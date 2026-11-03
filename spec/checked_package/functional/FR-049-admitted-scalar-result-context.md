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
  - target: ix://agent-ix/quire-spec-language/FR-062
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-075
    type: references
---
# FR-049: Expose admitted scalar result context for an exact requested expression

## Description

The IR-769 admitted-package query and TC-447 IR cases are implemented. The
QSL-712 checked per-item application/conversion pairing remains pending in
the QSL producer lane, so end-to-end item acceptance is not yet established.
For one QSL driver requirement over an exact scalar
expression occurrence, Contract IR SHALL lower the expression and its direct
conversion context together before it exposes typed result context. QSL
FR-062 and FR-075 own the checked requirement and its per-occurrence
`conversion_context() -> Option<&OccurrenceKey>`. The conversion is an IR
lowering request, not an additional QSL requirement, routed item or claim.
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
requirement's exact application `OccurrenceKey` and expression `CheckedNodeId`;
the QSL-owned `RequirementItem::conversion_context()` value, exactly
`Option<&OccurrenceKey>`; the selected lowering profile and limits. The
caller constructs the expression's and, when present, conversion's structural
source keys against the admitted V2 graph, then carries their corresponding
admitted V1 source-map occurrence ids without discarding the `expression`
role or ordinal. `None` means QSL's
checked result bound comes from the application's own result type. The caller
SHALL NOT infer a conversion by a CG graph walk or substitute the result-bound
type key for the conversion key.

At the result-context query: the one admitted `ContractPackageV1` emitted from
that lowering, the expression's `ContractSourceKeyV1`, the exact application
occurrence and the same QSL-owned optional conversion occurrence, plus the
expected scalar bound form. The caller constructs the expression key from its
QSL V2 requested id with `ContractSourceKeyV1::from_v2_request` while the
reader-admitted V2 package is available at the lowering boundary. Each
`ContractOccurrenceKeyV1` carries the admitted V1 source-frame node id,
`CheckedOccurrenceRole::Expression` and ordinal. A represented expression's
graph node and `source_map.nodes` outer id instead belong to the V1 output
frame; the two raw indexed ids need not be equal. The query does not accept
the original checked graph or infer an occurrence from ordinal alone.

## Outputs

A typed result distinguishes `Lowered`, the exact non-lowered request
disposition, and `Absent`, as [FR-035](./FR-035-complete-v1-contract-package-lowering.md)
does. A lowered result includes the existing forward reached closure and
exactly one occurrence-scoped result-context outcome:

- `DirectBounded`: QSL supplies `None` and the expression's own admitted
  `semantic_type` is a readable bounded domain of the expected form. That
  typed key, node and source occurrences are the result bound.
- `Unnarrowed`: QSL supplies `None` and the expression's own result type is a
  scalar type with no direct bounded result in this package. Represented
  conversions belonging to another requirement occurrence do not change it.
- `Narrowed`: QSL supplies `Some` direct conversion occurrence, and its
  admitted represented narrow directly consumes this expression. The result
  carries that one conversion key and occurrence plus its exact target bound
  key, node and source occurrences.
- `MissingBound`: the selected direct result type or conversion target has no
  readable bounded domain of the expected form over the expression's scalar
  type. It carries the failing typed key, without a substitute bound.

`IncompleteContext` is a typed refusal when the supplied `Some` occurrence is
not an admitted, represented, lowered direct narrow over the exact expression
source key, including when its root is absent, non-lowered or only an
identically keyed local collision. An application occurrence that is absent
from the admitted source map or whose source-frame structural identity differs
from the requested expression key also refuses before a bound is selected.
Raw id equality across the source and output frames is neither necessary nor
sufficient for admission. An inconsistent admitted
graph, failed shared body validation or exhausted work limit returns the
existing typed view error without a partial result or guessed bound.

The variant and typed keys, not formatted text, define the result.
`Unnarrowed` is the QSL checked own-result case, not an inference from no
represented reverse consumer and not a fabricated bound. A consumer may then
apply its separately specified own-result rule.

## Behavior

### Contextual lowering and authority

For one QSL driver requirement, Contract IR SHALL distinguish its sole
user-facing expression item from its optional context root. When
`conversion_context()` is `Some`, the driver SHALL pass both
`[item.node(), conversion_context.node()]` as explicit IR lowering roots for
that item, or pass the same pair to an equivalent IR contextual entry point.
When it is `None`, the item has only its expression root. For the QSL `a + 1`
positive, the two roots are the add and its direct `numeric.narrow`; the
narrow is context-only. For two requirements sharing one expression key, the
driver SHALL retain each item's own request index, application occurrence
and optional conversion occurrence when assembling roots. IR SHALL NOT select
a conversion by expression node alone. The selected dependency boundary stays as in
[FR-035](./FR-035-complete-v1-contract-package-lowering.md).

The output SHALL remain one ordinary six-member FR-330 package, including `source_keys`. Each
lowered root, including a context-only root, SHALL have the exact
[FR-035](./FR-035-complete-v1-contract-package-lowering.md) feature-accounting
record and source correspondence. Adding a context root legitimately changes
the represented graph, accounting and complete canonical package value; no new wire
field or separate lowering product is introduced. Neither the context root nor
its accounting record SHALL create a second QSL requirement, routed item,
claim-map entry or proof obligation.

### Admitted result-context query

Contract IR SHALL find feature accounting and the reached closure by the
typed expression source key, as FR-035 does. For a lowered request it SHALL
use the reached root's admitted output key to read the represented node and
`source_map.nodes` outer entry. It SHALL verify that the supplied application
occurrence's source-frame node id rebuilds to the requested source structural
key and that its role and ordinal match an inner occurrence of that output
entry. A same-numbered raw index in the two frames is not a match when the
rebuilt identities differ. For `Some`, IR SHALL find the exact conversion
occurrence in the admitted source map, its source structural key in a `lowered`
FR-035 feature-accounting record, and the corresponding represented output
node. The conversion occurrence's source-frame identity, role and ordinal
SHALL match that output node's admitted source-map entry. The node SHALL be
an `expression` of form `conversion` whose `convert` application has operation
identity exactly `quire.op.numeric.narrow` and one argument directly
referencing the represented expression's output key. The selected
conversion occurrence and application occurrence are distinct source-map
keys; IR SHALL preserve both without requiring their source regions to be
equal. A missing, non-lowered, nonrepresented or wrong-shape context returns
`IncompleteContext` before bound selection. A different represented direct
narrow over the same expression key passes these IR structural checks; only
the QSL checked request establishes which one belongs to the application
occurrence. Another represented application occurrence of that same
expression key likewise passes the IR source-map check; only the QSL checked
request establishes which application occurrence belongs to the item.
Contract IR SHALL use its
existing admitted body and operation knowledge for that check, not a copied
grammar in CG.

For a valid `Some`, Contract IR SHALL select only that direct conversion's
admitted result type as the occurrence's result-bound candidate. If the
target has the expected readable bounded form over the expression's scalar
type, the result is `Narrowed`; otherwise it is `MissingBound`. For `None`,
Contract IR SHALL use the expression's own admitted result type: a readable
bounded domain of the expected form gives `DirectBounded`, a scalar type gives
`Unnarrowed`, and a wrong or unreadable bounded form gives `MissingBound`.
These rules follow QSL's own-result versus direct-narrow distinction;
`DirectBounded` SHALL NOT override a valid `Some` selection.

The read-only query MAY scan represented local nodes to validate body
references and expose other consumers, but SHALL NOT infer a conversion from
them or treat an unrelated represented consumer as the selected context for
this requirement occurrence. Thus another occurrence's narrow, even when it
references the same expression key and appears in the same FR-330 package,
does not make this occurrence ambiguous when the QSL item supplies its own
context. IR cannot infer from FR-330 alone whether two represented narrows
over the same expression key were paired with the correct QSL item; the
driver SHALL preserve QSL's per-item occurrence/context association rather
than swap the contexts. QSL `check` refuses zero or multiple
matching direct narrows for a narrowed claim site before an admitted checked
package exists; an IR caller that supplies `Some` without the one direct
represented match receives `IncompleteContext`. A bare `lower([add])` package
cannot answer `Some(narrow)` as `Narrowed` because its narrow root and
occurrence are absent. The add's forward `reached_closure` remains unchanged.

The result SHALL retain the exact selected source package name, lowering
profile, requirement occurrence, requested key, selected conversion occurrence
and key when present, bound key and source-map occurrences
from the admitted package. A `dependency_reference` SHALL remain the selected
external package identity and target key described by
[FR-035](./FR-035-complete-v1-contract-package-lowering.md); it SHALL NOT be
resolved through an identically keyed local node or treated as a local reverse
consumer. A selected external target that is needed for an exact result SHALL
remain at that admitted package boundary. The reader SHALL refuse an unavailable
selected package or target before constructing an admitted value. The API SHALL NOT claim a
local result bound from an external target it has not resolved under that
selection.

Contract IR SHALL derive the result from the six existing FR-330 fields, including `source_keys`, and its
admitted IR knowledge. Querying it SHALL leave those fields, their complete canonical value,
the [FR-035](./FR-035-complete-v1-contract-package-lowering.md) forward
`reached_closure` membership, and the reader's admission and first-refusal
order unchanged. No new wire field, copied body grammar, per-node `ir_id`, or
second codegen graph traversal is part of this contract.

## QSL request boundary

QSL FR-062 and FR-075 own the direct conversion context and its
`Option<&OccurrenceKey>` accessor. QSL binds the value to each requirement
occurrence and rejects zero or multiple direct narrow matches for a narrowed
site before emitting a checked package. IR receives that checked value
alongside the admitted package after a strict FR-330 read; no seventh wire
field or global completeness claim from FR-330 bytes is required. For `None`,
IR relies on QSL's checked own-result assertion; a caller-created `None`
without that QSL request is not evidence that the original checked graph has
no reverse consumer. IR authenticates source-map correspondence for the
supplied application occurrence and, for `Some`, the selected conversion,
plus the represented narrow's direct reference to the expression's output key. The
QSL checked request, not source-region equality in IR, is authoritative for
which conversion occurrence belongs to which application occurrence.
At the lowering boundary the driver SHALL preserve QSL's indexed V2 request
identity long enough to construct the typed source key and SHALL retain the
admitted V1 source occurrence separately from the represented output key. A
consumer SHALL use IR's authenticated source/output correspondence when
selecting the result; it SHALL NOT prefilter a valid occurrence by comparing
its raw source-frame `node_id` with a represented output-frame `node_id`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-049-AC-1 | For the reader-admitted QSL source `function f using v(a: Int[0, 9]): Int[0, 10] pure { a + 1 }`, the one QSL driver item supplies its application occurrence and `Some` of the directly wrapping narrow occurrence. One IR lowering over roots `[item.node(), conversion_context.node()]` represents the direct narrow and result bound `Int[0, 10]` with two lowered records. Compared with plain `lower([add])`, the contextual graph contains that narrow and bound, while the add's forward reached closure has the same members by source structural identity, body and source occurrences, including operand `Int[0, 9]`. The admitted query for that same item returns `Narrowed` with result `[0, 10]` against independent QSL source intervals. Plain `lower([add])` remains an admitted six-member package, including `source_keys`, with no narrow or `[0, 10]` and returns `IncompleteContext` for that `Some` request. | Test |
| FR-049-AC-2 | For a QSL item with `None`, the expression's own readable bounded result type yields `DirectBounded` with its exact key and sources, a scalar own type yields `Unnarrowed`, and a wrong-form own bounded type yields `MissingBound`. For `Some`, one matching lowered direct narrow yields `Narrowed` from its target even when the expression also has an own bounded type; an invalid target yields `MissingBound`. An unrelated represented narrow, including another occurrence's narrow over the same expression key, changes none of these outcomes. | Test |
| FR-049-AC-3 | For `Some`, an absent, non-lowered or unrepresented conversion root, a conversion occurrence absent from the admitted source map (including a role or ordinal with no matching entry), a wrong operation/form, or an argument that does not directly reference the represented output key corresponding to the supplied expression source key returns typed `IncompleteContext` with no selected bound. A different admitted ordinal that names another represented direct narrow over the same expression key is structurally valid in IR; the QSL driver supplies the occurrence belonging to this item. Distinct source regions for a valid application and its direct narrow do not cause refusal. A local consumer that fails the direct-narrow checks cannot satisfy `Some` or turn a QSL `None` into `Narrowed`. QSL `check` refuses a narrowed site with zero or multiple matching direct narrow occurrences before IR receives an admitted package. | Test |
| FR-049-AC-4 | A non-lowered expression root returns its exact disposition, and an unrequested or absent expression key returns typed absence. A context-only root receives its own FR-035 accounting record but no additional QSL routed item or claim. An application occurrence absent from the admitted source map or naming a different expression key, a non-lowered required context root, an inconsistent post-admission edge or an exhausted work limit returns typed refusal/error without partial result context. Another represented application occurrence of the same expression key is not an IR-detectable mismatch; the QSL driver preserves the checked item's exact application occurrence. | Test |
| FR-049-AC-5 | The result identifies the exact application and selected conversion occurrences, expression key, source package and lowering profile, with admitted source-map entries for expression, conversion and result bound. Two QSL items sharing one expression key but carrying different direct-narrow occurrences and result bounds retain distinct outcomes when the driver supplies each item's own QSL checked occurrence/context pair, even when both narrows are represented in one package. IR does not claim to detect a swap between two represented direct narrows over that shared key without an IR-visible binding witness. A selected external reference retains its package/key pair; an identically keyed local node cannot satisfy it, and an unavailable selected package or target is refused at reader admission. | Test |
| FR-049-AC-6 | Contextual lowering emits exactly the existing six FR-330 fields, including `source_keys`, with one accounting record per expression or context root; adding a context root changes the graph, accounting and complete canonical bytes while retaining the package name as FR-035 requires. A read-only query with the QSL item occurrence and optional conversion occurrence mutates none of those fields or the package name, and the add's forward closure does not grow to include its reverse consumer. CG consumes the IR result without an original checked graph or second lowering pass. | Test |
| FR-049-AC-7 | PLANNED/UNRUN (IR-1299). In a reader-admitted real-QSL package with supported scalar siblings `f0` and `f2` separated by excluded root `f1`, each supported add and direct `numeric.narrow` is selected by its own V2-derived `ContractSourceKeyV1` and admitted V1 source occurrence and returns its own `Narrowed` bound. The excluded middle root retains its exact non-lowered disposition and cannot shift either sibling's selection. For at least one supported item, the V2 indexed request id, V1 source-map inner occurrence id and represented V1 output id have different raw graph-local indexes; the query still returns the matching lowered result by rebuilt structural identity and exact role and ordinal, and preserves the output root's admitted source occurrences. A source occurrence with a different rebuilt identity or wrong role or ordinal returns `IncompleteContext`, even when a raw index collides with the requested output id. Reordering unrelated output nodes or adding an independent excluded request does not change either supported result. No second lowering pass, CG graph walk or raw-id equality prefilter supplies the result. | Test |

## Dependencies

[FR-035](./FR-035-complete-v1-contract-package-lowering.md) owns the admitted
six-member package, including `source_keys`, and forward reached view.
[FR-038](./FR-038-consume-checked-package-v2.md) owns checked-package
admission and the operation/body rules. QSpec FR-330 owns the published wire.
QSL FR-062 owns the checked requirement's direct conversion occurrence, and
QSL FR-075 owns its driver-facing optional accessor and per-item mapping.
Codegen's scalar result consumer is IR-744 and remains outside this IR-owned
requirement.
