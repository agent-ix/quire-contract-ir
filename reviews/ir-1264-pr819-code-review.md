---
id: SR-2026110101269
title: "IR-1264 PR 819 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1264 PR #819; crates/quire-contract-model/src/checked_package/v2/{owner,model_members}.rs; tests/it/{checked_package_v2_model_fields,checked_package_v2_owners}.rs; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed the frozen Rust and test diff against FR-038-AC-420, QSpec FR-322 owner presence and AC-68, and TC-281 EV-01/EV-02. The event declaration owner path is correct, but the event invariant clause owner path remains excluded.

## Verdict

One medium finding. The frozen PR is not ready to merge until the event invariant owner join is resolved or its scope is explicitly dispositioned. No applicable AssuranceProfile was found under the IR spec tree.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | An invariant clause owned by a selected event type still refuses the owner join | crates/quire-contract-model/src/checked_package/v2/owner.rs:260-265; QSpec FR-322-AC-68 |

## Coverage

- FR-038-AC-420: examined; undeclared event ModelOwner, EV-01/EV-02, source-owned declared event, malformed owner paths.
- QSpec FR-322-AC-68: examined; event declaration and invariant clause ownership.
- QSpec TC-281 EV-01/EV-02: examined; positive and wrong-kind owner cases.
- `model_members.rs` event construct intake and `owner.rs` schema and owner join: examined.
- `checked_package_v2_model_fields.rs` and `checked_package_v2_owners.rs` new tests: examined.
- FR-038 planned marker removal: context only.

## Evidence

`model.event_types` gains selected event declarations, and `model_kind_matches` admits an event declaration itself. Its `CheckedNodeKind::Function` invariant branch still accepts only object and record declarations. A function clause with `clause = invariant` and a `ModelOwner` pointing to a selected event type therefore reaches `missing_declaration`/`missing-selection` at the function node, although QSpec FR-322-AC-68 requires that join to pass. The existing record invariant test at `tests/it/checked_package_v2_owners.rs:1239-1258` demonstrates the same owner-join seam. The new tests exercise the public reader and would fail on the old event-owner behavior. Focused TC-281 and declared-source-owner tests passed (2/2); `cargo fmt --check`, scoped Quire validation and `git diff --check` passed. The aggregate gate belongs to the lead after disposition.
