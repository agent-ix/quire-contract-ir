---
id: FR-347
title: "Admit a conforming record as an operation-call argument"
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
# FR-347: Admit a conforming record as an operation-call argument

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
comparison for a record-typed parameter and for nothing else.

A record **conforms** to a record `P` when it is `P` or `P` is among its
ancestors at any depth. The relation is one-way: a child conforms to its
ancestor, and an ancestor never conforms to a descendant.

## Inputs

An application of `operation` kind `operation` over a model-owned declaring
node, with its `arguments`, and the selected domain package document that
declares the operation and the records ([FR-038](./FR-038-consume-checked-package-v2.md)
"Model-owned members").

## Outputs

The call admits, or the reader refuses `ill_typed`/`operator-ineligible` at
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
  `Integer`, `Int[lo, hi]`, `Reference<O>`, `Option<X>` and every collection)
  is compared by node key exactly. A collection or `Option` of a conforming
  record is not covariant: an argument of type `Sequence<S>` or `Option<S>`
  at a `Sequence<P>` or `Option<P>` parameter refuses, because the collection's
  type identity includes its element type (QSL FR-359 item 5).
- An argument that is not a recovered `model`/`record_value_type` node (a
  `composite_type`/`record` node with a source owner, an object type, an
  anonymous structural node) is compared exactly. Ancestry is not decidable
  for it from the package.
- The receiver (argument 0) is checked as before: exact `Reference<X>` of the
  declaring node. This requirement does not change it.
- The application's `result_type` is compared with the operation's declared
  result type by node key, as before.

### Refusals

An argument whose record is unrelated to the parameter's, is an ancestor of
the parameter's record (an intermediate ancestor included), or belongs to
another selected document, refuses `ill_typed`/`operator-ineligible` at that
argument. The first offending argument in position order is reported, and the
arity check still precedes the argument checks.

### Work accounting

A walk charges one `work` unit per `supertypes` edge followed, to the same
meter as the member resolution of FR-038-AC-30. An exact match charges none.
A limit one below the work a read used returns `incomplete` for `work`.

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
| FR-347-AC-4 | A parameter typed `Integer`, `Int[0, 9]`, `Boolean`, `Reference<O>`, `Option<Money>` or `Sequence<Money>` is compared by node key exactly: an argument typed `TaxedMoney` at the `Option<Money>` or `Sequence<Money>` parameter, an argument typed `Option<TaxedMoney>` at the `Option<Money>` parameter, an `Int[0, 5]` argument at an `Int[0, 9]` parameter and a `Reference` of a subtype at a `Reference<O>` parameter each refuse `ill_typed`/`operator-ineligible` at the argument; the same argument at its exact type admits. | Test (TC-448) |
| FR-347-AC-5 | An argument typed at a `composite_type`/`record` node with a source owner, with the same name and fields as `TaxedMoney`, refuses `ill_typed`/`operator-ineligible` at `charge(m: Money)`: ancestry is read from the selected document only. A `TaxedMoney` record of a second selected document does not conform to the first document's `Money` and refuses the same way. | Test (TC-448) |
| FR-347-AC-6 | A call with two record-typed parameters, a conforming first and a non-conforming second argument, refuses at argument 2 (the second parameter); with both non-conforming it refuses at argument 1. A call with the wrong argument count refuses at `arguments` before any argument is judged, as before. | Test (TC-448) |
| FR-347-AC-7 | The receiver of a call to `charge` typed at a `Reference` of a subtype of the declaring object is refused or admitted exactly as before this requirement (FR-347 changes no receiver check), and an admitted call's `result_type` that differs from the operation's declared result type by key still refuses `ill_typed`/`operator-ineligible` at `result_type`. | Test (TC-448) |
| FR-347-AC-8 | Resolving the conformance of a `GiftTaxedMoney` argument at a `Money` parameter charges one `work` unit per `supertypes` edge followed (two), an exact `Money` argument charges none, and a work limit one below the measured total returns `incomplete` for `work`; the exact total admits. | Test (TC-448) |

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
