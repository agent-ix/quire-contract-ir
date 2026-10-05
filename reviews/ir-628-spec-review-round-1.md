---
id: SR-1564
title: "Round-1 spec review of quire-contract-ir PR #296 fix round (IR-628 admission-time field tables)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@bd47f6aa58501e5d88afff2657f634f9f2b4f382; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Typed accessor for a model object type's fields' items 1-26, IR-628-Q1..Q3, FR-038-AC-136..144), spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md, spec/checked_package/matrix/tests.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: IR-628. This covers the new findings of disposition pass 1 over the fix round
(7ffa956..bd47f6a, one commit). The defect classes of round 0 were re-checked against the new and
changed text, which is items 1-26, AC-136..144 and TC-227.

What I re-measured:

- `make spec`: 37 unbacked rows at the merge base `bf36cda`, 47 at head. The ten new rows are
  exactly FR-038-AC-136..144 and TC-227, all planned. Validate is clean.
- FR-019-AC-5 and TC-058 are planned (`spec/model/matrix/tests.md:17`, `29`), so the Note on the
  FR-019 table is accurate.
- The PR body now says CG must amend FR-015-AC-77, AC-78 and AC-81.

**Retained memory.** It is bounded by the work charged. Each table entry is an exposed field, and
building a table visits each of its members at one charged unit. The owned index holds one entry
per declaration. Retained memory is therefore at most about `work` times a constant (1e6 units
under `CheckedPackageReadLimits::bounded()`), and I found no route to hold memory admission has not
paid for.

**`NotModelObjectType` and the unread-node divergence.** The resolution of round 0 (item 8, item 17,
and "What the accessor does for a node admission did not check") is consistent with the guarantee
paragraph, which promises only that returned bounds are the document's. I checked the worrying
case: a consumer falling back to body members for a selected-key node with a tampered body, which
the accessor reports as `NotModelObjectType`. That node cannot be framed. Any frame entry, state
clause or read on a `model` node with no `declaration` runs `is_model_declaration_node`, then
`resolve_member`, then `recover`, and is refused `stale-node-key` at admission (`frame.rs:480`,
`state.rs:198`, `model_members.rs:899-918`). So the divergence is confined to nodes no consumer
route names.

## Verdict

Changes requested: two medium and two low new findings. Every round-0 finding is fixed (see the
Dispositions sections of SR-1559 to SR-1563).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Items 1-3 make admission compute a table for every object type declaration of every selected document, charged "in the units step 3 charges": an ancestor walk per type, plus `resolve`'s per-call ancestor memo for every redefining owner. For an inheritance chain of depth N, that is about N²/2 ancestor edges. AC-143 and AC-144's own document (10000 types, 1000 redefinitions) needs about 10^10 units, and AC-144 admits it at the exact limit and again one below. That is not a runnable test. A pure unread chain of about 1500 types, which admits today under `bounded()` (work 1e6), would return `incomplete`. Q2 understates this. It names only "object types no read names", but every declared object type is charged, and a read charges `resolve` again on top. It also does not say the growth is quadratic in hierarchy depth. Either charge each table incrementally from its supertypes' tables (linear in edges plus output), or state the quadratic cost and its effect on `bounded()`, and shrink AC-143/AC-144's document to a size a test can admit. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2029, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2222, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2865 |
| FND-002 | medium | Item 3 does not place the new table computation in FR-038's reader order. FR-038 fixes the first-fault order of every refusal and limit. A package whose work runs out in table building and that also carries a graph-stage refusal gets `incomplete` if tables are built in the lock stage (FR-322 step 1, before `validate_graph`), and gets the graph refusal if they are built after it. Two implementations would differ in an observable outcome, and AC-144 pins no precedence row. Name the stage (for example "after each selection's document is read, in lock order, before the graph stage") and add a precedence row. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2029, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2865 |
| FND-003 | low | AC-141's `IntRange` perturbation, "`upper - 1`, or `upper + 1` when `upper` is `i128::MIN`", gives an empty range for any degenerate `Int[v, v]` with `v > i128::MIN`. AC-138's `Int[0, 0]` becomes `Int[0, -1]`, so the perturbed read's type node has `min > max`, and such a node may be refused for its own shape (for example under IR-627's stage) rather than `ill_typed` at the read. Perturb `upper + 1` when `lower == upper < i128::MAX`, and `lower - 1` at `Int[i128::MAX, i128::MAX]`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2862 |
| FND-004 | low | The fix round promises one `shall` per item, but items 5, 16 and 17 still carry two each. Item 5 says "shall retain that table as ambiguous ... and shall not refuse admission". Item 16 says "shall give `member_type` `None` and shall never be saturated". Item 17 says "shall read no node body ... and shall read no `result_type`". Split each. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2037, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2079, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2081 |

## Dispositions

Round 2, reviewed at ca74e19099387db1c45f57ff320b95ff89a2e263.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ca74e19099387db1c45f57ff320b95ff89a2e263 |
| FND-002 | fixed | ca74e19099387db1c45f57ff320b95ff89a2e263 |
| FND-003 | fixed | ca74e19099387db1c45f57ff320b95ff89a2e263 |
| FND-004 | fixed | ca74e19099387db1c45f57ff320b95ff89a2e263 |
