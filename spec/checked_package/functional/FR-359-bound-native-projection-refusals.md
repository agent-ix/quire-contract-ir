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
# FR-359: Bound clock and Exact-projection refusals under the native join budget

## Description

When a consumer reads a selected monitor clock or projects one Exact value type
from an admitted checked package, Contract IR SHALL charge the read's work,
owned result, scratch storage and owned refusal payload to the caller's shared
[FR-358](./FR-358-compose-native-identity-join-budget.md) `NativeJoinBudget`
before the corresponding inspection or allocation. Each read SHALL return a
complete success or one typed refusal without a partial view.

## Inputs

An admitted `CheckedPackageV2`, the selected clause and occurrence for a clock
read or requested node and selected identities for an Exact projection, and
one mutable `NativeJoinBudget` that may already have charges from stage-row
acquisition or source joins.

## Outputs

A successful read returns the same selected clock or Exact type as the prior
read at sufficient allowance. A refusal returns its existing stable semantic
category where its diagnostic is payable, or the exhausted resource kind when
the next work or storage charge is denied. A resource refusal has fixed-size
metadata identifying the denied kind and attempted charge without allocating
an owned node path, form or partial view. The shared budget exposes cumulative
charges and the first unpaid charge.

## Behavior

The two reads SHALL require `&mut NativeJoinBudget` and use its cumulative
counters. Neither read SHALL create a fresh allowance from a numeric work
limit or expose a numeric-limit overload. Each read SHALL begin and finish
under [FR-358](./FR-358-compose-native-identity-join-budget.md)'s delta and
first-unpaid rules. Its success delta includes the complete returned value;
refusal clears the success delta and retains previously admitted charges. A
denied charge leaves both counters unchanged and cannot wrap them.

Before cloning a `CheckedNodeId`, copying a dynamic form or path, creating an
owned diagnostic container, reserving a result container, or retaining an
owned result element, Contract IR SHALL admit the corresponding logical
storage charge under [FR-358](./FR-358-compose-native-identity-join-budget.md)'s
unit rules. The reads SHALL not allocate merely to report storage exhaustion.
If a semantic diagnostic cannot be paid, the read SHALL return fixed-size
storage exhaustion and preserve the first unpaid charge.

At zero storage, a request that needs an owned result or diagnostic SHALL
return fixed-size storage exhaustion. When every diagnostic charge is admitted,
semantic defects retain their stable category, first selected locus and
meaning. Work and storage charge order SHALL follow actual inspection and
ownership steps, including nested Exact projection. The implementation SHALL
not reserve a worst-case diagnostic that a successful read never uses.

Selected facts and type decisions remain governed by
[FR-352](./FR-352-project-admitted-exact-value-types.md). The clock read retains
its admitted clause, occurrence, binding and name checks. No QSL identity
judgment, source reparse, evaluator, compatibility entry point, copied
artifact or fixed fallback allowance is introduced.

The public `project_selected_type_declarations` API also switches from a
numeric limit to required `&mut NativeJoinBudget` in this stage. Full runtime
storage and diagnostic precharge, including its unit-definition and model
member helpers, remains a separate qualification under IR-1366; this FR does
not claim that runtime path as complete.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-359-AC-1 | Selected clock and Exact projection require the same caller-owned `&mut NativeJoinBudget` used by stage-row acquisition and source joins. A prior affordable operation can leave either resource one unit short of a read that succeeds under a fresh ample budget; the shared read refuses at its first unpaid step. The runtime projection public signature also requires this budget, with no numeric-limit overload; its full accounting is deferred to IR-1366. | Test, Inspection |
| FR-359-AC-2 | For a real admitted success in each qualified read family, measuring work and storage with ample limits and rerunning at both exact limits returns the same complete value. One unit below either measured requirement returns the typed exhausted kind, first unpaid charge and no partial value; cumulative counters stay within the limits. | Test |
| FR-359-AC-3 | A semantic refusal in each qualified family whose owned diagnostic includes a dynamic node path or form retains its stable category and first locus at ample limits. Zero storage returns fixed-size storage exhaustion; exact diagnostic storage returns the full semantic refusal, and one below it returns storage exhaustion. No refusal exposes a partial owned payload or successful delta. | Test, Inspection |
| FR-359-AC-4 | Nested Exact projection and its error propagation charge each newly owned payload once. A payable semantic refusal retains its category and locus; an unpaid conversion returns fixed-size storage exhaustion. Repeated inputs and limits, including an admitted package clone, give equal values or first stops and equal counters. | Test |
| FR-359-AC-5 | Existing selected clock and Exact success and non-budget refusal decisions remain unchanged at sufficient limits. Neither qualified read has uncharged owned result, scratch or refusal allocation, a second accounting scheme, or a QSL or runtime dependency in the IR model. Runtime full-precharge evidence is not credited by this criterion. | Test, Inspection |

## Dependencies

- [FR-358](./FR-358-compose-native-identity-join-budget.md) defines shared
  cumulative work and logical-storage accounting.
- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns Exact identity
  and type projection decisions.
- [FR-354](./FR-354-project-admitted-runtime-context.md) owns selected runtime
  declaration decisions and the staged API transition.
- [FR-019](../../model/functional/FR-019-rust-library-interface.md) indexes
  the public model API.
