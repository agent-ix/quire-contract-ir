---
id: SR-1740
title: "Code review of IR-644 wrapped optional leaf walk"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@66c362435cd627f5d72718f8f34b065ee97713dd; crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_recursive_leaves.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed PR #301 at its frozen head against FR-038-AC-151/152, including Rust implementation, exact wrapper shape, direct-reference control, malformed refusals, and the focused tests. No code defect was found.

## Verdict

**PASS** — The new branch accepts exactly one `optional` binding to an option node, preserves the record field edge, and rejects malformed shapes at operation leaves. Seven focused recursive-leaf tests passed.

## Scope

- **FR-038-AC-151 (examined):** PLANNED for IR-644 code; no backing test yet. Over the QSL-shaped recursive `List` record with an integer field and `next` encoded as `binding(next, aggregate([binding(optional, reference Option<List>)]))`, `structural.eq` over two `List` values admits with `leaves` empty, and the leaf derivation terminates through the option's `inner` edge at the record reentry. Replacing that integer field with a text field whose profile is selected admits exactly its text leaf followed by `["field:next", "inner", "recursion:0"]`; omission of that recursion leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Adding a healthy text sibling after `next` retains its own leaf after the recursion leaf in declaration order, proving that the wrapper consumes exactly one field edge. A direct field `reference` to `Option<List>` also admits with the same `field:next`, `inner` path.
- **FR-038-AC-152 (examined):** PLANNED for IR-644 code; no backing test yet. With the rest of the `List` comparison well formed and its identity members recomputed after each mutation, replace only the value of `next` by an `aggregate` with no member, two `optional` members, a member named otherwise, a non-binding member, or an `optional` binding whose value is not a reference to an option type. Each mutation is admitted by the flat body grammar; after identity re-derivation, each reaches the operation check and refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, rather than silently omitting `next` or deriving a leaf from a different member.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Rust review included panic, bounds, type resolution, test assertions, trace bindings, vendoring and duplicate logic, and CI diff; no workflow files changed.
- Test binding: `tc_048_wrapped_optional_record_fields_keep_their_leaf_order` → FR-038-AC-151; `tc_048_malformed_optional_record_fields_refuse_at_operation_leaves` → FR-038-AC-152. Their matrix registration is assessed separately in SR-1741.
- Focused command: `cargo test --workspace --test it checked_package_v2_recursive_leaves::tc_048_ -- --nocapture` — 7 passed.
