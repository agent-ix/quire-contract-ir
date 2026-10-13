---
id: FR-347
title: "Admit a conforming record at operation-call, field-read, case-arm, union-payload and function-call positions"
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
# FR-347: Admit a conforming record at operation-call, field-read, case-arm, union-payload and function-call positions

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
comparison for a record-typed parameter. QSpec FR-322-AC-69 relaxes other
positions the same way, and this requirement states the reader's rule at each
one the reader checks today: a field-read operand, a `case` arm body, a
`union_value` payload and, with the callee's body `parameters` binding (FR-322-AC-70), a
`function.call` argument. It also adds a body-root result check, which the
reader does not make today. This requirement changes nothing else.

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

### Field projection over a record operand (two changes)

Today every `field` member read whose declaring node is a model record refuses
`ill_typed`/`operator-ineligible` at the member's `name`: `check_model_member`
requires the recovered owner to be an object type, and an owner that
declares a record is not one. No admitted package reads a record field. This
requirement makes two changes.

1. **Record owners.** A `field` member read whose `declaration` is a record
   value type `P` recovers `P`'s owner and resolves the field by name among
   `P`'s effective members, own and inherited from `P`'s ancestors, as
   FR-038 step 3 does for an object type. The member type is compared with
   `result_type` by node key as before. The object-type requirement stays for
   an `operation` member, and a record owns no operation (refused at `name`).
2. **Conforming operand.** The operand's type node (argument 0) is admitted
   when it is `P` (exact) or a recovered `model`/`record_value_type` node of
   the same selected document with a supertype chain to `P`. A field declared
   only by a descendant of `P` is not found among `P`'s effective members and
   refuses at the member's `name`. An operand of an object type, or of a
   record that does not conform, refuses `ill_typed`/`operator-ineligible` at
   `arguments/0`.

### Case arm bodies and union payloads

Today a `case` arm body must have exactly the application's `result_type`
(`union_arms_hold`), and a `union_value` payload term must have exactly the
member's declared payload type. Where the expected type is a record value
type, an arm body or payload term whose type is a recovered
`model`/`record_value_type` conforming to it is admitted by the rule above;
every other expected type, and every binder's `semantic_type` (a binder is
bound to the payload type, not an argument), stays exact. A failing arm body
refuses as today: the `union_arms` constraint not holding is
`ill_typed`/`operator-ineligible` at the `case` node itself,
`/semantic_graph/nodes/{n}`, not at an argument. A failing payload is
`ill_typed`/`type-mismatch` at the `union_value` node.

### Result positions

The reader compares no function or expression body root with the node's
declared type today. This requirement adds the check (QSpec FR-322-AC-69
refuses a body root that does not conform). For a `function` node, the root
is the node its body binding `body` references; for an `expression` node, it
is the root application's `result_type`. The root's type node must equal the
node's `semantic_type`, or, when that is a record value type, conform to it by
the rule above. Otherwise the reader refuses `ill_typed`/`type-mismatch`, the
cause of the `union_value` payload comparison, at the node
(`/semantic_graph/nodes/{n}`). A node whose body names no root takes no check.

### Work accounting

The walk is the reader's existing ancestor walk, reused whole: it does not
stop at the parameter's (or `P`'s) record, and it charges one `work` unit for
every `supertypes` edge reachable from the argument's record, each ancestor's
edges followed once, to the same meter as the member resolution of
FR-038-AC-30. The row charged, and reported as `incomplete` at
`/lock/model_selections/<i>`, is the selection of the document that declares
the expected record (the parameter's, the case result's, the payload's, the
callee parameter's or the body result's type), because the walk runs in that
document; the operation-call and field-read rules have the declaring owner's
row. A `GiftTaxedMoney` argument therefore costs 2 units at a
`Money` parameter and also at a `TaxedMoney` parameter. In a diamond (`D`
with supertypes `B` and `C`, each with supertype `A`) the walk costs 4 at any
ancestor. An exact match charges none. A limit one below the work a read used
returns `incomplete` for `work`.

### Function-call arguments

Today the reader compares neither the number nor the types of a
`quire.op.function.call`'s arguments with the callee (FR-038, "Documented
limits"). This requirement adds both checks, from the callee function node's
body. QSL FR-092 "Function nodes" is the source of the body binding shape;
QSpec FR-322-AC-70 is the rule that uses it.

- **Where the declared types come from.** The callee is argument 0, a
  `reference` to a `function` node of this package. The function node's body
  is an `aggregate` of bindings whose first binding, `parameters`, is an
  `aggregate` of `reference` terms to the `value`/`parameter` nodes in
  parameter order, including a parameter the function's body never reads. The
  declared type of parameter `i` is the `semantic_type` of the node the
  `i`th reference names. The reader reads nothing else for the signature.
- **The checks, in order.** The number of arguments after the callee equals
  the number of references in `parameters`, else `ill_typed`/
  `operator-ineligible` at `arguments`. Then each argument, in position
  order, has its parameter's type node exactly, or conforms to it when the
  parameter's type is a record value type, by the rule above, else
  `ill_typed`/`operator-ineligible` at that argument.
- **No `parameters` binding.** A callee function node whose body holds no
  member named `parameters` takes no checks, as today.
- **Malformed binding.** The binding is the member named `parameters` that is
  the first member of the body. When a member of that name is present and the
  binding is malformed, the reader refuses `ill_typed`/`operator-ineligible`
  at the call's callee argument, `/semantic_graph/nodes/{n}/body/arguments/0`.
  Malformed means: the member is not the first of the body, its value is not an
  `aggregate`, an entry is not a `reference`, an entry names no node of the
  graph, or an entry names a node that is not a `value`/`parameter` node.
- **Dependency callee.** A callee reached by a `dependency_reference` names a
  node of a supplied dependency package, whose body the reader can index. It
  must have a package-independent signature (FR-038), which names no
  model-owned record, so a record parameter there cannot be conformed to and
  stays exact. A `dependency_reference` callee's arguments stay unchecked, as
  FR-038's "Documented limits" state; this requirement applies the checks to a
  `reference` callee only.

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
| FR-347-AC-4 | A parameter typed `Integer`, `Int[0, 9]`, `Boolean`, `Reference<O>` or `Set<Money>[0, 1]` (declared with multiplicity `[0, 1]`, unique and unordered) is compared by node key exactly: an argument typed `TaxedMoney` and one typed `Set<TaxedMoney>[0, 1]` at the `Set<Money>[0, 1]` parameter, an `Int[0, 5]` argument at an `Int[0, 9]` parameter and a `Reference` of a subtype at a `Reference<O>` parameter each refuse `ill_typed`/`operator-ineligible` at the argument; the same parameter with its exact argument admits. A hand-built package whose argument is typed `Option<Money>` at a `Money` parameter refuses the same way. | Test (TC-448) |
| FR-347-AC-5 | An argument typed at a `composite_type`/`record` node with a source owner, with the same name and fields as `TaxedMoney`, refuses `ill_typed`/`operator-ineligible` at `charge(m: Money)`: ancestry is read from the selected document only. A `TaxedMoney` record of a second selected document does not conform to the first document's `Money` and refuses the same way. | Test (TC-448) |
| FR-347-AC-6 | A call with two record-typed parameters, a conforming first and a non-conforming second argument, refuses at argument 2 (the second parameter); with both non-conforming it refuses at argument 1. A call with the wrong argument count refuses at `arguments` before any argument is judged, as before. | Test (TC-448) |
| FR-347-AC-7 | The receiver of a call to `charge` typed at a `Reference` of a subtype of the declaring object refuses `ill_typed`/`operator-ineligible` at `arguments/0`, as before this requirement, and an admitted call's `result_type` that differs from the operation's declared result type by key still refuses `ill_typed`/`operator-ineligible` at `result_type`. | Test (TC-448) |
| FR-347-AC-8 | Conformance of a `GiftTaxedMoney` argument charges 2 `work` units at a `Money` parameter and also 2 at a `TaxedMoney` parameter (the walk does not stop at the parameter's record); with a diamond `D` (supertypes `B`, `C`; each with supertype `A`) the argument `D` at an `A` parameter charges 4. An exact argument charges none. For each, a work limit one below the measured total returns `incomplete` for `work` and the exact total admits. | Test (TC-448) |
| FR-347-AC-9 | A `record.project` read of `occurred_at` declared by `OrderPlaced`, with `declaration` naming `OrderPlaced`, admits over an operand typed `OrderPlaced` (the exact record read, which refuses today), over an `ExpressOrderPlaced` child and over a `RushOrderPlaced` grandchild, and admits an inherited field read over a child-typed `declaration` (`order_id` named on `ExpressOrderPlaced`). It refuses `ill_typed`/`operator-ineligible` at `arguments/0` over an unrelated record and over an object type, a read of `promised_by` (declared only by the child) with `declaration` naming `OrderPlaced` refuses at the member's `name`, and an `operation` member naming a record refuses at `name`. | Test (TC-448) |
| FR-347-AC-10 | A `case` over a union whose arm bodies are typed `TaxedMoney` and `Money` with `result_type` `Money` admits; an arm body typed `Voucher` or `Money` at `result_type` `TaxedMoney` refuses `ill_typed`/`operator-ineligible` at the `case` node, `/semantic_graph/nodes/{n}`; a binder whose `semantic_type` is a conforming child of the payload type refuses (binders stay exact); a non-record `result_type` stays exact. | Test (TC-448) |
| FR-347-AC-11 | A `union_value` whose member payload is declared `Money` admits a payload term typed `TaxedMoney` and `GiftTaxedMoney`, and refuses `Voucher` and `Money` at a `TaxedMoney` payload with `ill_typed`/`type-mismatch` at the node; a non-record payload stays exact. | Test (TC-448) |
| FR-347-AC-12 | A `function` node with declared `semantic_type` `Money` whose `body` binding references a root typed `Money`, `TaxedMoney` or `GiftTaxedMoney` admits, as does an `expression` node whose root application has that `result_type`; a root typed `Voucher`, or typed an ancestor at a child-typed declared type (`Money` under `TaxedMoney`), refuses `ill_typed`/`type-mismatch` at `/semantic_graph/nodes/{n}`; a non-record declared type stays exact. | Test (TC-448) |
| FR-347-AC-13 | A `function.call` whose callee is a `reference` to a `function` node with body binding `parameters` = `[reference(p0), reference(p1)]`, `p0` typed `Money` and `p1` typed `Integer`, admits arguments `[Money, Integer]`, `[TaxedMoney, Integer]` and `[GiftTaxedMoney, Integer]`. It refuses `ill_typed`/`operator-ineligible` at `arguments` for one or three arguments, and at the argument for `[Voucher, Integer]`, `[Money, Int[0, 5]]` and, with `p0` typed `TaxedMoney`, `[Money, Integer]`. A parameter the function body never reads still counts: a callee with three `parameters` references and a body reading one refuses two arguments. A callee function node with no `parameters` binding takes no argument check. | Test (TC-448) |
| FR-347-AC-14 | A callee function node whose `parameters` member is present and malformed refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{n}/body/arguments/0` for each of: the member second in the body, a value that is not an `aggregate`, an entry that is a `literal`, an entry referencing an absent node, and an entry referencing a node that is not a `value`/`parameter` node. | Test (TC-448) |
## Open Questions

- Whether the reader should also accept a conforming argument at a
  `Reference<P>` parameter. FR-322 "Reference conformance" owns reference
  conformance; this requirement leaves it unchanged. Not chosen here because
  QSL FR-359 asks only for record values.

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the operation check,
  the owner recovery and the ancestor walk this requirement reuses.
- QSpec FR-322-AC-69 (conforming records) and FR-322-AC-70 (the callee's body
  `parameters` binding) are the producer side of this rule (QSL FR-359
  cross-repo requirement 5); this requirement states the reader's rule and
  follows them where it differs.
