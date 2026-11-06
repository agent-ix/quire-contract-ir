---
id: FR-359
title: "Bound clock and type-projection refusal payloads under the native join budget"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---
# FR-359: Bound clock and type-projection refusal payloads under the native join budget

## Description

When a consumer reads a selected monitor clock, projects one Exact value type,
or projects selected runtime type declarations from an admitted checked package,
Contract IR SHALL charge the work and logical storage of the read and its
owned refusal payload to the caller's shared [FR-358](./FR-358-compose-native-identity-join-budget.md)
`NativeJoinBudget` before performing the corresponding inspection or allocation.
The three reads SHALL return a complete success or one typed refusal without a
partly authoritative view.

## Inputs

An admitted `CheckedPackageV2`, the request and selected identities or rows
required by the existing read, and one mutable `NativeJoinBudget` that may
already have charges from stage-row acquisition or source joins. The clock
read receives its selected clause and occurrence; Exact projection receives
its requested node and selected identities; runtime type projection receives
its selected identities, tuple rows and source-unit rows.

## Outputs

A successful read returns the same selected clock, Exact type or complete
runtime type declarations as the corresponding prior read at sufficient
allowance. A refusal returns its existing stable semantic category where that
category is payable, or the exhausted resource kind when its next work or
storage charge is denied. A resource refusal has fixed-size diagnostic
metadata sufficient to identify the denied kind and attempted charge without
allocating an owned node path, form string or partial view. The caller can
inspect cumulative charges and the first unpaid charge on the shared budget.

## Behavior

Each public read SHALL charge its work and storage through a required
`&mut NativeJoinBudget` parameter and its cumulative counters. No read SHALL create a fresh allowance from a numeric
work limit or use a separate uncharged diagnostic path. The operation SHALL
begin and finish under [FR-358](./FR-358-compose-native-identity-join-budget.md)'s
delta and first-unpaid rules. Its success delta includes construction of the
complete returned value; refusal clears that success delta and retains all
previously admitted charges. The first denied charge leaves both counters
unchanged and cannot wrap them.

Before cloning a `CheckedNodeId`, copying a dynamic form or path, creating an
owned diagnostic container, reserving a result container, or retaining an
owned result element, Contract IR SHALL admit the corresponding logical
storage charge under [FR-358](./FR-358-compose-native-identity-join-budget.md)'s
unit rules. This applies to error construction on every semantic and resource
exit, including a nested Exact error converted to a runtime type error. The
read SHALL not allocate merely to report that storage is exhausted. Where
the full semantic diagnostic is unaffordable, the read SHALL return the
fixed-size storage-exhaustion form, preserving the first unpaid charge rather
than reporting an incompletely constructed semantic diagnostic.

When a request would allocate its refusal under a zero storage allowance,
Contract IR SHALL return the fixed-size storage-exhaustion form. When every
diagnostic storage charge is admitted, Contract IR SHALL retain the same semantic
defect's stable category, first selected
locus and existing diagnostic meaning. Work and storage charge order SHALL
follow actual bounded inspection and ownership steps, so a one-unit-short
limit stops at the first unpaid step. The implementation SHALL neither
preallocate a full diagnostic before charging it nor reserve a worst-case
payload that a successful read never uses.

The selected facts and decision semantics remain governed by
[FR-352](./FR-352-project-admitted-exact-value-types.md) and
[FR-354](./FR-354-project-admitted-runtime-context.md). The clock read retains
its admitted clause, occurrence, binding and name checks. No QSL identity
judgment, source reparse, evaluator, compatibility entry point, copied
artifact or fixed fallback allowance is introduced.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-359-AC-1 | The three public reads require the same caller-owned `&mut NativeJoinBudget` used by stage-row acquisition and source joins. A prior affordable operation can leave either resource one unit short of a read that succeeds under a fresh ample budget; the shared read refuses at its first unpaid step. Public API inspection finds no numeric-limit overload or fresh local budget. | Test, Inspection |
| FR-359-AC-2 | For a real admitted success in each read family, measuring its work and storage with ample limits and rerunning at both exact limits returns the same complete value. One unit below either measured requirement returns the typed exhausted kind, first unpaid charge and no partial value; cumulative counters never exceed the limits. | Test |
| FR-359-AC-3 | A semantic refusal in each family whose owned diagnostic includes a dynamic node path or form is returned with the existing category and first locus at ample limits. At zero storage it returns a fixed-size storage refusal without owned diagnostic allocation; at the exact diagnostic storage charge it returns the full semantic refusal, and one below it returns storage exhaustion. The returned error and budget expose no partial owned payload or successful delta after refusal. | Test, Inspection |
| FR-359-AC-4 | A nested Exact refusal converted during runtime type projection is charged once for each newly owned payload; it preserves the Exact category and locus when payable and returns the fixed-size storage refusal when conversion is unpaid. Repeated inputs and limits, including an admitted package clone, give equal values or first stops and equal counters. | Test |
| FR-359-AC-5 | Existing selected clock, Exact type and runtime declaration success and non-budget refusal decisions remain unchanged at sufficient limits. The implementation has no uncharged owned result, scratch or refusal allocation along these three paths, no second accounting scheme, and no QSL or runtime dependency in the IR model. | Test, Inspection |

## Dependencies

- [FR-358](./FR-358-compose-native-identity-join-budget.md) defines shared
  cumulative work and logical-storage accounting.
- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns Exact identity
  and type projection decisions.
- [FR-354](./FR-354-project-admitted-runtime-context.md) owns selected runtime
  declaration decisions.
- [FR-019](../../model/functional/FR-019-rust-library-interface.md) indexes
  the public model API.
