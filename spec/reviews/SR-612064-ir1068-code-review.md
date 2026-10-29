---
id: SR-612064
title: "IR-1068 canonical execution point code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 635 Rust diff in identity.rs, wire.rs, checked_package/v2, tests/it; FR-011-AC-3, FR-058-AC-11, FR-019-AC-5"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: reviews
---

## Summary

Ticket: IR-1068. Reviewed PR #635 against merged main. The single canonical public point type and V2 nonempty-name path are present, but authored expression wire input is widened by the same type.

## Verdict

**FAIL** — the expression-operation decoder admits a nonempty point name outside authored `AnchorName` grammar, contrary to FR-011-AC-3. A prior hosted compile failure from a missing `ExecutionPoint` import was already fixed in this reviewed head. No full local gate was run; hosted checks on the reviewed head were pending. The diff adds no vendoring, compatibility bridge, unsafe code or CI workflow change; `git diff --check` passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `ExpressionInput.execution_point` now deserializes the widened canonical `ExecutionPoint` directly. Its `ExecutionPointName` deserializer checks only nonempty text, and `check_expression_input` never calls authored `AnchorName` validation. A public expression-operation input with `handler{name:"tick now"}` can therefore pass this boundary instead of returning `invalid_identifier`; the same decoder serves binding-expression input. | crates/quire-contract-model/src/wire.rs:30; crates/quire-contract-model/src/identity.rs:973 |

## Coverage

- FR-011-AC-3, role: examined. Excerpt: “An empty or malformed authored execution-point name, including one supplied through the public canonical point to an authored clause constructor, returns `invalid_identifier` under the existing `AnchorName` rule. The same nonempty name may still be admitted through V2 when QSpec permits it.” FND-001 applies to the expression-operation and binding-expression wire decoders named in FR-011's Behavior.
- FR-058-AC-11, role: examined. Excerpt: “The crate root exports exactly one public four-variant point type, `ExecutionPoint`. An authored clause rejects `tick now` and an empty point name with FR-011's `invalid_identifier`, including when the caller builds the canonical point directly; a V2 reader admits `tick now` and the accessor returns that exact variant and name to a CG-style external consumer without inspecting JSON.” Local constructor and V2 readback assertions are present; QSL producer migration remains outside IR PR.
- FR-019-AC-5, role: examined. Excerpt: “The public default-feature crate-root items have an exact documented inventory and no undocumented public export.” The new `ExecutionPointName` root export is entered in the Public items table and `CheckedExecutionPoint` is removed from the root inventory.
- FR-058-AC-1/3/4/5/6/9/10, role: context_only. Existing V2 accessor tests were inspected for exact-name and signature preservation; this PR updates their expected type without changing their prior behaviors.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, `ExpressionInput.execution_point` is private `WireExecutionPoint`; `check_expression_input` calls its `validate()` method before binding comparison or expression checking. The four validation arms use `AnchorName::new`, so empty and `tick now` now return `InvalidIdentifier` for unbound expression operation and bound binding expression input. The V2 `ExecutionPoint::from_wire` path remains nonempty-only and separate. |

## Disposition verdict

Round 1: FND-001 fixed on inspection. Focused/hosted gate outcomes remain the lead's final gate evidence.
