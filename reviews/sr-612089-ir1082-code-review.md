---
id: SR-612089
title: "IR-1082 code-review review of typed operation views"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/; crates/quire-contract-model/src/lib.rs; tests/it/; spec/checked_package/functional/; spec/model/functional/"
review_set: subset
---

## Summary

Rust/API review covered typed accessors, deserialization, reader validation, public exports, tests, and repo idioms. Reviewed PR #646 at 2309ad5d550a09f9e6e0e3634365976b6d1744bd against merged spec main 7f89788e512a7db8afb5ff5e28f948778c89ec45. No applicable AssuranceProfile was found. No vendoring, gate weakening, raw JSON accessor, or compatibility layer was added.

## Verdict

**CONDITIONAL** — A public construction path bypasses the admitted-view promise.

## Scope

- FR-019-AC-5 (examined): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.
- FR-038-AC-331 (examined): Implemented (IR-1082). Through the public V2 reader, an admitted field-member operation returns its exact `(CheckedNodeId, name)` declaration/name pair from `projection_field()`; an admitted member of another kind and an operation with no member return `None`. A malformed or unselected field declaration is refused by admission and cannot be returned.
- FR-038-AC-332 (examined): Implemented (IR-1082). Through the public V2 reader, `member_declaration()` returns the exact admitted `CheckedNodeId` for a field member and a catalogued non-field member carrying `declaration`, including `quire.op.quantity.convert`; it returns `None` when the admitted member has no declaration or is absent. A malformed declaration is refused by admission and cannot be returned.
- FR-038-AC-333 (examined): Implemented (IR-1082). Through the public V2 reader, an admitted rounding-mode operation returns a `CheckedOperationModeView` with `OperationModeKind::Rounding` and its exact admitted value. A different admitted mode kind returns that kind and its value; an operation with no mode returns `None`. A malformed or catalog-ineligible mode is refused by admission and cannot be returned.
- FR-038-AC-334 (examined): Implemented (IR-1082). Through the public V2 reader, `law_definitions()` returns the exact admitted `CheckedArtifactRef` authority/identity pair for each law, in admitted order; no laws return an empty view. A law with an unselected or malformed definition is refused by admission and cannot be returned. The accessor exposes no raw operation record.
- FR-040-AC-37 (examined): Implemented (IR-1082). Through the public V2 reader, each admitted invariant, constraint, precondition and postcondition application returns its exact `StateClauseKind` from `state_clause_kind()`; an admitted non-state-clause operation returns `None`. A malformed clause member is refused by admission and cannot produce a value through this accessor. The accessor returns a typed value without exposing a raw operation record.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public Deserialize populates admitted views from unvalidated JSON | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:176 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Direct deserialization now leaves `views: None`; the validator installs typed facts from a successfully checked `OperationWire`. The new adverse test proves forged raw operations expose no typed views. |
