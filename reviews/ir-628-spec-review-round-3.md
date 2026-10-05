---
id: SR-1573
title: "Round-3 spec review of quire-contract-ir PR #296 fix round (IR-628 supertype cycles and field read charge)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@265e08258548c8138d57e140f9b9cfeedad3ef43; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-322 work-limit paragraph, section 'Typed accessor for a model object type's fields' items 1-37, 'Cost, stated', IR-628-Q2, FR-038-AC-136..144), spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-628. These are the new findings of disposition pass 3, the cap round, over ca74e19..265e082
(one commit).

**Cycle handling.** I re-measured it against `DomainModel::ancestors` and `resolve`
(`model_members.rs:623-760`).

- For `A extends A`, `ancestors(A)` pops `A` twice and reaches `{A}`, so `resolve` iterates the
  owners `A, A` and exposes `A`'s fields.
- For a two-cycle `A <-> B`, `ancestors(A)` is `{A, B}`, so both types expose both fields, and the
  owner sets of `A` and `B` are identical. One shared table per strongly connected component is
  therefore exactly what `resolve` returns for each member (items 4 and 6).
- For `g redefines f` on `A extends A`, `f` is hidden as a target. The pair loop finds `A` in
  `ancestors(A)`, so `g` hides itself, and neither field is exposed. This matches AC-137.
- Charges under item 9:

  | Case | Own fields | Edges | Copied | Total |
  | --- | --- | --- | --- | --- |
  | Self-cycle, one field | 1 | 1 | 0 | 2 |
  | Two-cycle, one field each | 2 | 2 | 0 | 4 |

- Recording the expected tables by running the pre-change `resolve` is a sound differential test
  design.

**Field read charge.** Item 10 replaces the step 3 walk for a field with one unit per table entry.
That count is at most today's walk charge, which covers ancestor edges, every owner's fields and
operations, and redefinition pairs, so a field read never costs more than it does today. Item 7, the
Cost paragraph, the FR-322 work-limit paragraph (line 957), Q2 and the PR body all agree. They state
that operation resolution keeps the walk, that read-heavy packages fall in cost, and that the code
change must re-measure exact-limit tests.

**Counts.**
- All 37 items carry exactly one `shall`, counted programmatically.
- Numbering runs AC-136..144 and TC-227 with no collisions.
- `make spec`: 37 unbacked rows at `bf36cda`, 47 at head, and the new rows are exactly AC-136..144
  and TC-227. There are no CI edits.

## Verdict

Approved with one low finding. Every SR-1572 finding is fixed. This is the only remaining defect,
and the fix is one phrase.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Item 10 gives the new field-resolution charge only for "a read, frame entry or state clause that resolves a field". Item 7 routes every field case of `resolve` through the table, and an abstraction relation's field entry also resolves a field (`abstraction.rs:599-604`, `resolve_member(.., MemberKind::Field, ..)`). Its charge is therefore left unstated. State clauses resolve only operations (`state.rs:205`). Smallest fix: replace the subject of item 10 with "Every resolution of a field through `DomainModel::resolve` (a read, including a relationship edge read, a frame entry, or an abstraction relation's field entry)". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2057 |

## Dispositions

Round 4, reviewed at 6082ac0c84ec0340a375dc90872fb98601e68d0c.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 6082ac0c84ec0340a375dc90872fb98601e68d0c |
