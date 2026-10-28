---
id: SR-612072
title: "IR-1082 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #645 22858d99; FR-019 Public items, FR-038 AC331-334, FR-040 AC37"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: reviews
---

## Summary

Ticket: IR-1082. Reviewed frozen PR #645 head 22858d99 against authoritative main c485581f and measured CG consumer inventory 707d5fa9. FR-019 exact public inventory compared with TC-058; all five operation projections checked against admitted V2 reader semantics.

## Verdict

**FAIL** — FR-019 inventory breaks its declared TC-058 control until the planned names are excluded from the current parser.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The planned FR-019 mode types are backticked, so TC-058 expects exports absent from this spec-only head | spec/model/functional/FR-019-rust-library-interface.md:193; tests/it/public_interface.rs:883-906 |

## Coverage

- FR-019-AC-5, role: examined. Excerpt: “The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.”
- FR-038-AC-331, role: examined. Excerpt: “PLANNED (IR-1082). Through the public V2 reader, an admitted field-member operation returns its exact `(CheckedNodeId, name)` declaration/name pair from `projection_field()`; an admitted member of another kind and an operation with no member return `None`. A malformed or unselected field declaration is refused by admission and cannot be returned.”
- FR-038-AC-332, role: examined. Excerpt: “PLANNED (IR-1082). Through the public V2 reader, `member_declaration()` returns the exact admitted `CheckedNodeId` for a field member and a catalogued non-field member carrying `declaration`, including `quire.op.quantity.convert`; it returns `None` when the admitted member has no declaration or is absent. A malformed declaration is refused by admission and cannot be returned.”
- FR-038-AC-333, role: examined. Excerpt: “PLANNED (IR-1082). Through the public V2 reader, an admitted rounding-mode operation returns a `CheckedOperationModeView` with `OperationModeKind::Rounding` and its exact admitted value. A different admitted mode kind returns that kind and its value; an operation with no mode returns `None`. A malformed or catalog-ineligible mode is refused by admission and cannot be returned.”
- FR-038-AC-334, role: examined. Excerpt: “PLANNED (IR-1082). Through the public V2 reader, `law_definitions()` returns the exact admitted `CheckedArtifactRef` authority/identity pair for each law, in admitted order; no laws return an empty view. A law with an unselected or malformed definition is refused by admission and cannot be returned. The accessor exposes no raw operation record.”
- FR-040-AC-37, role: examined. Excerpt: “PLANNED (IR-1082). Through the public V2 reader, each admitted invariant, constraint, precondition and postcondition application returns its exact `StateClauseKind` from `state_clause_kind()`; an admitted non-state-clause operation returns `None`. A malformed clause member is refused by admission and cannot produce a value through this accessor. The accessor returns a typed value without exposing a raw operation record.”

## Assurance Context

No applicable AssuranceProfile was identified for these three changed FR rows. Selected context unavailable: none. Active exceptions: none.
