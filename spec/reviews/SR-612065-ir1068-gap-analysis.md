---
id: SR-612065
title: "IR-1068 canonical point gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 635 tests/it/identity.rs, tests/it/checked_package_v2_frame_entries.rs, crates/quire-contract-model/src/wire.rs; FR-011-AC-3, FR-058-AC-11, FR-019-AC-5"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: reviews
---

## Summary

Ticket: IR-1068. Planless, PR-diff-scoped trace review of canonical point construction, authored wire intake, V2 exact-name preservation and public inventory evidence.

## Verdict

**CONDITIONAL** — the new traced test covers `Clause::new` and package-wire refusal, but no changed test exercises malformed authored point names through the expression-operation or binding-expression wire decoders. The missing case matches the code defect in SR-612064. No full local test gate was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new FR-011-AC-3/FR-058-AC-11 test mutates only a package clause's `anchor` wire member. It never sends `tick now` through `check_expression_input`, which handles both the expression operation and binding expression input. The test can pass while those authored wire decoders violate FR-011's identifier refusal. | tests/it/identity.rs:22; crates/quire-contract-model/src/wire.rs:597 |

## Coverage

- FR-011-AC-3, role: examined. Excerpt: “An empty or malformed authored execution-point name, including one supplied through the public canonical point to an authored clause constructor, returns `invalid_identifier` under the existing `AnchorName` rule.” The new test asserts constructor and package decoder behavior only.
- FR-058-AC-11, role: examined. Excerpt: “The crate root exports exactly one public four-variant point type, `ExecutionPoint`... a V2 reader admits `tick now` and the accessor returns that exact variant and name to a CG-style external consumer.” Existing TC-462 readback was updated to the canonical type; full QSL producer adaptation is not claimed in this IR code review.
- FR-019-AC-5, role: examined. Excerpt: “The public default-feature crate-root items have an exact documented inventory and no undocumented public export.” The edited Public items row reflects the new root export, and the TC-058 inventory gate remains.
- FR-058-AC-1/3/4/5/6/9/10, role: context_only. Existing accessor test cases were checked for type-only changes.
- Reverse trace: `ExecutionPointName`, authored constructor validation and V2 use are owned by FR-011 and FR-058. No unowned behavior found in this diff.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed |, `expression_inputs_refuse_non_identifier_point_names` tests both empty and `tick now` through `operation_refusal` (unbound expression operation) and `check_expression_input(..., Some((owner, anchor)))` (bound expression input), asserting `InvalidIdentifier` and the authored anchor path in both routes. |

## Disposition verdict

Round 1: FND-001 fixed on inspection. The FR-058 producer-origin and QSL migration obligations remain outside this IR PR.
