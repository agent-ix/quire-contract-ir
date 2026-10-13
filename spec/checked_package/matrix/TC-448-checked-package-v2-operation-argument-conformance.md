---
id: TC-448
title: "CheckedPackage V2 operation-call arguments admit conforming records and keep other kinds exact"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-347
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-359
    type: references
---
# TC-448: CheckedPackage V2 operation-call arguments admit conforming records and keep other kinds exact

## Description

Verify FR-347-AC-1 through FR-347-AC-13: the V2 reader admits an operation-call
argument whose record type conforms to the parameter's declared record type,
refuses an unrelated, ancestor or foreign-document record with
`ill_typed`/`operator-ineligible` at the argument, and keeps every
non-record parameter exact.

## Test Procedure

Build a lock selecting a domain package document declaring the records
`Money`, `TaxedMoney` (supertype `Money`), `GiftTaxedMoney` (supertype
`TaxedMoney`) and `Voucher` (no supertype), and an object type with the
operations `charge(m: Money)`, `settle(t: TaxedMoney)`, `grandsettle(g:
GiftTaxedMoney)`, `pair(a: Money, b: TaxedMoney)`, and operations with
`Integer`, `Int[0, 9]`, `Boolean`, `Reference<O>` and `Set<Money>[0, 1]`
parameters (a parameter has no presence, so `Option<Money>` is not
declarable; the collection vectors use the `[0, 1]` unique multiplicity),
and a diamond `D` (supertypes `B`, `C`; each with supertype `A`) with an
operation over `A`. For field reads, add `OrderPlaced` with `occurred_at`, its
child `ExpressOrderPlaced` adding `promised_by`, and grandchild
`RushOrderPlaced`; a union with a `Money` payload member and a `case` over it;
a `function` node with `semantic_type` `Money`; and callee `function` nodes whose body binds `parameters` to `reference`s to
`value`/`parameter` nodes typed `[Money, Integer]` (one variant with a third,
unread parameter, one with no `parameters` binding). Build a checked package for each call below,
computing every expected node id through `quire-canonical`, not through the
reader, and read each.

Positive: `charge` with `Money`, `TaxedMoney` and `GiftTaxedMoney` arguments;
`settle` with `TaxedMoney` and `GiftTaxedMoney`; `grandsettle` with
`GiftTaxedMoney`; each non-record parameter with its exact type.

Negative, each a single mutation of a positive case: `charge` with `Voucher`;
`settle` with `Money`; `grandsettle` with `TaxedMoney` and with `Money`;
`Set<Money>[0, 1]` with `TaxedMoney` and with `Set<TaxedMoney>[0, 1]`; a
hand-built package with an `Option<Money>`-typed argument at a `Money`
parameter; `Int[0, 9]` with `Int[0, 5]`;
`Reference<O>` with a `Reference` of a subtype; `charge` with a source-owned
`composite_type`/`record` node shaped like `TaxedMoney`; `charge` with a
`TaxedMoney` of a second selected document; `pair` with a conforming and a
non-conforming argument in each order, and with both non-conforming; a wrong
argument count. For the work case, measure the work of the `GiftTaxedMoney`
argument read at a `Money` and at a `TaxedMoney` parameter (both 2) and of
`D` at `A` (4), then read at each limit and one below. Field reads: `occurred_at`
over `OrderPlaced`, child and grandchild operands (admit), over an unrelated
record and an object type (refuse at `arguments/0`), and `promised_by` named on
`OrderPlaced` (refuse at `name`); the exact `OrderPlaced` read and an inherited
read named on the child (admit); case arms, union payloads, the function body
root and `function.call` argument counts and types, read from the callee's `parameters`
binding, per AC-10 to AC-13.

## Expected Results

Positive packages admit with no `supertypes` member or conversion node.
Each negative package, including the receiver-subtype case at `arguments/0`,
returns `ill_typed`/`operator-ineligible` at
`/semantic_graph/nodes/{n}/body/arguments/{position}` of the first offending
argument (the arity case at `arguments`), and no package. The measured work
admits and one less returns `incomplete` for `work`.

## Status

PLANNED / UNRUN: no implementation yet. Target file
`tests/it/checked_package_v2_operation_arguments.rs`.
