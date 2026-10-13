---
id: FR-347
title: "Admit a conforming record as an operation-call argument or field-read operand"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-208
    type: references
  - target: ix://agent-ix/quire-specification/FR-151
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-359
    type: references
---
# FR-347: Admit a conforming record as an operation-call argument or field-read operand

## Description

When the `quire.checked-package/v2` reader ([FR-038](./FR-038-consume-checked-package-v2.md))
checks a model-owned `operation` member call, the reader SHALL admit an
argument whose type is a record value type that conforms to the record value
type its parameter declares, and SHALL keep every other argument exact.

QSpec FR-208 and FR-151 say a value of a record subtype is a value of each
ancestor wherever a parameter names the ancestor. QSL FR-359 (cross-repo
requirement 4) asks the reader to accept the checked package QSL emits for
such a call. Today FR-038 "Model-owned members" step 4's operation check
compares each argument's type node with the parameter's declared type by node
key alone, so a child passed to a parent-typed parameter refuses
`ill_typed`/`operator-ineligible`. This requirement replaces that exact
comparison for a record-typed parameter. QSpec FR-322-AC-69 likewise relaxes the
operand of a `record.project` field read, and the reader's field check is
exact today (the operand's type node must equal the member's `declaration`),
so a field read over a child-typed operand refuses; this requirement relaxes
that check by the same rule. It changes nothing else.

A record **conforms** to a record `P` when it is `P` or `P` is among its
ancestors at any depth. The relation is one-way: a child conforms to its
ancestor, and an ancestor never conforms to a descendant.

## Inputs

An application of `operation` kind `operation` over a model-owned declaring
node, with its `arguments`, and the selected domain package document that
declares the operation and the records ([FR-038](./FR-038-consume-checked-package-v2.md)
"Model-owned members").

## Outputs

The call or read admits, or the reader refuses `ill_typed`/`operator-ineligible` at
`/semantic_graph/nodes/{n}/body/arguments/{position}`, the cause and locus the
exact comparison gives today.

## Behavior

### What the reader consults

The package carries no `supertypes` member and no conversion node on the
checked side (QSL FR-359 item 3). The ancestry of a record is the
`supertypes` its declaration carries in the domain package document the lock
selects, which the reader already reads, and pins by digest, for the
effective-field and Reference-conformance rules of FR-038 "Model-owned
members" (FR-038-AC-29). The reader therefore decides conformance as follows
and consults nothing else:

1. **The parameter's record.** The parameter's declared `typeRef` names a
   record value type declaration in the selected document, whose
   declaration key is the expected type node key of step 4.
2. **The argument's record.** The argument's type node (`operand_type_node`)
   is a `model`/`record_value_type` node whose `ModelOwner` the reader has
   recovered by step 2 to a declaration of a selected document.
3. **Same document, then ancestry.** The two declarations belong to one
   selected document, and a chain of declared `supertypes` in that document
   leads from the argument's declaration to the parameter's. The chain
   follows the ancestors the reader already walks for FR-038-AC-29; no
   second ancestor table is built.

An argument whose type node key equals the parameter's expected key is
admitted as today, without a walk. Any other argument is admitted only when
steps 2 and 3 hold.

### Exact kinds stay exact

- A parameter of any type other than a single record value type (`Boolean`,
  `Integer`, `Int[lo, hi]`, `Reference<O>` and every collection) is compared
  by node key exactly. A collection of a conforming record is not covariant:
  an argument of type `Set<S>[0, 1]` at a `Set<P>[0, 1]` parameter refuses, as
  does a bare `S` there, because the collection's type identity includes its
  element type (QSL FR-359 item 5). A domain package document cannot declare
  an `Option<P>` parameter (a parameter has no presence; only a field does,
  and a `[0, 1]` parameter is a bounded `Set`), so no `Option` parameter case
  exists in a document-backed package. An `Option<S>`-typed argument at a
  record parameter `P` is a hand-built node and refuses like any non-record
  argument.
- An argument that is not a recovered `model`/`record_value_type` node (a
  `composite_type`/`record` node with a source owner, an object type, an
  anonymous structural node) is compared exactly. Ancestry is not decidable
  for it from the package.
- The receiver (argument 0) is checked as before: exact `Reference<X>` of the
  declaring node, so a `Reference` of a subtype refuses
  `ill_typed`/`operator-ineligible` at `arguments/0`. This requirement does
  not change it.
- The application's `result_type` is compared with the operation's declared
  result type by node key, as before.

### Refusals

An argument whose record is unrelated to the parameter's, is an ancestor of
the parameter's record (an intermediate ancestor included), or belongs to
another selected document, refuses `ill_typed`/`operator-ineligible` at that
argument. The first offending argument in position order is reported, and the
arity check still precedes the argument checks.

### Field projection over a conforming operand

For a `field` member whose `declaration` is a record value type `P` (the
declaration QSL FR-359 item 2 names: the record that declares the field), the
operand's type node (argument 0) is admitted when it is `P` or the same
conformance holds as above: a recovered `model`/`record_value_type` node of
the same selected document with a supertype chain to `P`. The field is
resolved by name among `P`'s effective members as before, so a field declared
only by a descendant of `P` is not found and refuses at the member's `name`.
An operand of an object type, or of a record that does not conform, refuses
`ill_typed`/`operator-ineligible` at `arguments/0`. The result type is the
type `P` gives the field, compared with `result_type` as before.

### Work accounting

The walk is the reader's existing ancestor walk, reused whole: it does not
stop at the parameter's (or `P`'s) record, and it charges one `work` unit for
every `supertypes` edge reachable from the argument's record, each ancestor's
edges followed once, to the same meter as the member resolution of
FR-038-AC-30. A `GiftTaxedMoney` argument therefore costs 2 units at a
`Money` parameter and also at a `TaxedMoney` parameter. In a diamond (`D`
with supertypes `B` and `C`, each with supertype `A`) the walk costs 4 at any
ancestor. An exact match charges none. A limit one below the work a read used
returns `incomplete` for `work`.

### Function-call arguments

A `quire.op.function.call` argument is not covered yet: the checked package's
`function` node does not carry its declared parameter types. QSpec FR-322 is
adding that parameter list. When it lands, the reader's target rule for
`function.call` is: first the argument count equals the parameter count, else
`ill_typed`/`operator-ineligible` at `arguments`; then each argument, in
position order, is its parameter's type exactly, or conforms to it when the
parameter is a record value type, by the rule above, with the same refusal at
the first offending argument. A later amendment states it as acceptance
criteria.

### Scope

Only `record_value_type` is covered. QSL FR-359 also treats an event type as
a record value, but the reader admits no `model`/`event_type` node until QSpec
adds one (FR-359 cross-repo requirement 7); event conformance joins this rule
with that form.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-347-AC-1 | Over a selected document declaring `Money`, `TaxedMoney` with supertype `Money`, `GiftTaxedMoney` with supertype `TaxedMoney`, an unrelated record `Voucher`, and an object type with the operation `charge(m: Money)`, a call whose argument is typed at `Money` admits (control, exact), one typed at `TaxedMoney` admits, and one typed at `GiftTaxedMoney` (grandchild) admits. The child cases admit with no `supertypes` member and no conversion node in the package. | Test (TC-448) |
| FR-347-AC-2 | In the same document a call to `charge(m: Money)` whose argument is typed at `Voucher` (unrelated) refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{n}/body/arguments/1`. | Test (TC-448) |
| FR-347-AC-3 | A call to `settle(t: TaxedMoney)` with an argument typed at `Money` (the ancestor) refuses `ill_typed`/`operator-ineligible` at that argument, and so does a call to `grandsettle(g: GiftTaxedMoney)` with an argument typed at `TaxedMoney` (an intermediate ancestor) or at `Money`. `settle` with a `GiftTaxedMoney` argument admits. | Test (TC-448) |
| FR-347-AC-4 | A parameter typed `Integer`, `Int[0, 9]`, `Boolean`, `Reference<O>` or `Set<Money>[0, 1]` (declared with multiplicity `[0, 1]`, unique) is compared by node key exactly: an argument typed `TaxedMoney` and one typed `Set<TaxedMoney>[0, 1]` at the `Set<Money>[0, 1]` parameter, an `Int[0, 5]` argument at an `Int[0, 9]` parameter and a `Reference` of a subtype at a `Reference<O>` parameter each refuse `ill_typed`/`operator-ineligible` at the argument; the same parameter with its exact argument admits. A hand-built package whose argument is typed `Option<Money>` at a `Money` parameter refuses the same way. | Test (TC-448) |
| FR-347-AC-5 | An argument typed at a `composite_type`/`record` node with a source owner, with the same name and fields as `TaxedMoney`, refuses `ill_typed`/`operator-ineligible` at `charge(m: Money)`: ancestry is read from the selected document only. A `TaxedMoney` record of a second selected document does not conform to the first document's `Money` and refuses the same way. | Test (TC-448) |
| FR-347-AC-6 | A call with two record-typed parameters, a conforming first and a non-conforming second argument, refuses at argument 2 (the second parameter); with both non-conforming it refuses at argument 1. A call with the wrong argument count refuses at `arguments` before any argument is judged, as before. | Test (TC-448) |
| FR-347-AC-7 | The receiver of a call to `charge` typed at a `Reference` of a subtype of the declaring object refuses `ill_typed`/`operator-ineligible` at `arguments/0`, as before this requirement, and an admitted call's `result_type` that differs from the operation's declared result type by key still refuses `ill_typed`/`operator-ineligible` at `result_type`. | Test (TC-448) |
| FR-347-AC-8 | Conformance of a `GiftTaxedMoney` argument charges 2 `work` units at a `Money` parameter and also 2 at a `TaxedMoney` parameter (the walk does not stop at the parameter's record); with a diamond `D` (supertypes `B`, `C`; each with supertype `A`) the argument `D` at an `A` parameter charges 4. An exact argument charges none. For each, a work limit one below the measured total returns `incomplete` for `work` and the exact total admits. | Test (TC-448) |
| FR-347-AC-9 | A `record.project` read of `occurred_at` declared by `OrderPlaced`, with `declaration` naming `OrderPlaced`, admits over an operand typed `OrderPlaced`, an `ExpressOrderPlaced` child and a `RushOrderPlaced` grandchild. It refuses `ill_typed`/`operator-ineligible` at `arguments/0` over an unrelated record and over an object type, and a read of `promised_by` (declared only by the child) with `declaration` naming `OrderPlaced` refuses at the member's `name`. | Test (TC-448) |

## Open Questions

- Whether the reader should also accept a conforming argument at a
  `Reference<P>` parameter. FR-322 "Reference conformance" owns reference
  conformance; this requirement leaves it unchanged. Not chosen here because
  QSL FR-359 asks only for record values.

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the operation check,
  the owner recovery and the ancestor walk this requirement reuses.
- QSpec FR-322 is to state record-operand conformance for operation-call
  arguments (QSL FR-359 cross-repo requirement 5); this requirement states the
  reader's rule and follows FR-322 where it differs.
