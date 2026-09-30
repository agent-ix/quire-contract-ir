---
id: SR-624
title: "code review of PR 229 (reaches_field reference_edge constraint)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@65d29c5a3bf803c340987fb21eaadb6bd173013b; crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_model_members.rs, spec/contract/FR-040-admit-frame-entries-and-state-clauses.md"
review_set: subset
---
# SR-624: code review of PR 229

## Summary

Ticket: IR-370. The PR replaces the unconditional refusal of the
`reference_edge` constraint with `check_reference_edge`
(`operations.rs:1125-1263`), which decides QSpec FR-322 "Reaches over a
field" as read from quire-specification `origin/main`. It also skips the
generic field-member check for a reference-edge operation
(`operations.rs:586-593`). Rust-review lane included.

Each FR-322 rule compared with the code:

- Rule 1, operand 0 is `Reference<D>` with `D` the member's declaration:
  matches (`operations.rs:1150-1160`).
- Rule 2, the member resolves on `D` by steps 2 and 3, with their refusals:
  matches. `owners.recover` and `package.resolve` are used, and
  missing-selection, stale-node-key and ambiguous-name are passed through.
- Rule 3, the field type is `Reference<T>`, `Option<Reference<T>>` or a
  bounded `Sequence<Reference<T>>`, with `T` the declaring owner: matches
  (`operations.rs:1197-1227`). A set, an unbounded sequence, a reference to
  another type and a scalar are all refused.
- Rule 4, operand 1 is `Reference<B>` with `B` conforming to `T`: **does not
  match.** The code calls the symmetric `DomainModel::conforms`. See FND-001.

Gates on 65d29c5 with CARGO_TARGET_DIR in the worktree: fmt-check, both clippy
lanes, the full test suite (179 + 48 + 2 doctests), corpus, `quire validate`,
deny, cargo-audit and audit-unsafe all pass. `quire coverage --strict` fails
with 22 unbacked rows. The same command on a clean `origin/main` (3e7935f)
gives the same 22 rows; the two outputs differ only in line numbers. So the
coder's claim holds, and this PR neither adds nor removes an unbacked row.

Mutation oracle (`cargo test --test it reaches_field`):

- **Go red:** always-admit, always-refuse, dropping rule 1, dropping rule 3,
  dropping the sequence-bounds requirement, admitting any collection kind,
  admitting any reference type, and dropping rule 4.
- **Stay green:**
  - Replacing symmetric with one-directional conformance. No test pins the
    direction.
  - Dropping the relationship-declaring-node check. Equivalent in practice,
    because `resolve` refuses a non-object-type node anyway.
  - Dropping the same-package check.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Rule 4 uses the symmetric `DomainModel::conforms`, so it admits an operand 1 whose type `B` is a strict supertype of `T`. FR-322 says "B conforms to T", and FR-151 defines that relation one way: `B` is `T` or `B` has a declared supertype chain to `T`. Reproduced: with `Sub.child: Option<Reference<Sub>>`, `reaches_field(Reference<Sub>, Reference<Order>, child)` is admitted. With a one-way check it is refused. | crates/quire-contract-model/src/checked_package/v2/operations.rs:1251-1258 |

## Finding Detail

- FND-001: FR-322 "Reaches over a field" rule 4 reads "operand 1 has type
  `Reference<B>` whose object type `B` conforms to `T`". FR-151 reads "Type
  `S` conforms to type `T` exactly when `S` and `T` are the same effective
  type or a chain of declared supertypes leads from `S` to `T`". That is one
  way.
  - The "Reference conformance" section is symmetric only because it says so
    explicitly: "one conforms to the other", and "Operand order does not
    matter". Rule 4 cites that section for owner recovery and for the rule
    that only declared supertypes count. It does not restate "either
    direction".
  - TC-281 RE-05 admits a subtype target. No vector admits a supertype target.
  - FR-322 closes with "Any other operand ... refuses
    `ill_typed`/`operator-ineligible`".
  - The repo's own new FR-040 paragraph repeats "that conforms to `T`", so the
    code disagrees with this repo's statement too.
  - Fix: decide rule 4 one way. `B == T`, or `T` is in `ancestors(B)`. For
    example, add a `DomainModel::conforms_to(sub, sup)` next to the symmetric
    `conforms`. Then add a supertype-target refusal case to
    `tc_048_reaches_field_refuses_an_invalid_edge_where_it_fails` so the
    direction is pinned.
  - If the owners mean the symmetric reading, FR-322 needs a QSpec ruling
    saying so. Until then, the literal text refuses this case.
  - Reproduction: add a `child` field typed `Sub` to `Sub` in
    `edge_document`, then admit `(SUB, "child", SUB, ORDER)`. It passes on
    65d29c5 and fails with a one-way `conforms`.

## Verdict

Not mergeable until FND-001 is fixed. The fix is small and local. Rules 1-3,
refusal locations, budget charging and step-2 refusal pass-through are
correct. The mutations show the new tests catch every rule except the
direction of rule 4.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 467ac07: rule 4 now calls the one-way `DomainModel::conforms_to(end_owner.node, edge_owner)` (model_members.rs:504-513, operations.rs:1254); `(SUB, "child", SUB, ORDER)` refusal pins it. Mutations symmetric-conform and reversed-conform both go red. |
