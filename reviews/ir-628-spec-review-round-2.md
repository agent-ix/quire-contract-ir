---
id: SR-1572
title: "Round-2 spec review of quire-contract-ir PR #296 fix round (IR-628 incremental field tables)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@ca74e19099387db1c45f57ff320b95ff89a2e263; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Typed accessor for a model object type's fields' items 1-33, 'Cost, stated', IR-628-Q1..Q3, FR-038-AC-136..144), spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-628. These are the new findings of disposition pass 2 over bd47f6a..ca74e19 (one commit).
I re-measured the following.

**Cost arithmetic (re-derived).** Under item 5, a chain whose types each declare one field and
extend the previous type charges 1 for the root and k + 1 at depth k ≥ 2 (one own field, one edge,
k − 1 copied entries). The total is 1 + Σ_{k=2..N}(k + 1) = N(N+1)/2 + N − 1:

| N | Sum | Total |
| --- | --- | --- |
| 4 | 1+3+4+5 | 13 |
| 8 | 1+3+…+9 | 43 |
| 1000 | 500,500 + 999 | 501,499 |
| 1500 | 1,125,750 + 1,499 | 1,127,249 |

The total for N = 1500 exceeds `bounded()`'s work limit of 1,000,000
(`crates/quire-contract-model/src/checked_package/shared.rs:32-41`). I tried to falsify the AC-144
numbers and could not. AC-143's 200-type document with 100 redefinitions charges on the order of
10^4 to 10^5 units, so it is runnable.

**Stage placement.** `validate` runs `validate_lock` before `validate_graph` (`v2/mod.rs:728-732`).
In the lock stage, `validate_domain_packages` (step 1 for every row) precedes
`admit_dependencies`, the feature check and `validate_definition_ref`. Application
`stale-node-key` is raised in `operations.rs`, inside the graph stage. Both AC-144 precedence rows
agree with that order.

**Other checks.**
- AC-141's perturbations keep `min <= max`, including `Int[0, 0]` and the `i128`/`u64` edges.
- Q2 is honest: it says every declared object type is charged, that packages admitting today can
  become `incomplete`, and gives the 1500/1000 example.
- `make spec`: 37 unbacked rows at `bf36cda`, 47 at head. The new rows are exactly AC-136..144 and
  TC-227. There are no CI edits.
- Of the 33 items, 32 carry exactly one `shall`. Item 2 carries two (FND-003).

## Verdict

Changes requested: one medium and two low new findings. Every SR-1564 finding is fixed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Item 2 builds each object type's table "from its own fields and the tables of its declared supertypes, each declaration built once", but documents with cyclic supertypes are admitted today. `check_supertype` (`model_members.rs:1808-1817`) checks only that a supertype names an object of the document. No refusal row covers a cycle. `DomainModel::ancestors` carries a `reached` set for exactly this case (`model_members.rs:623-645`). For `A extends B, B extends A`, or `A extends A`, there is no build order: each table needs the other first. Item 5's charge and the retained table are then undefined, and a naive recursive build does not terminate. Say what the build does for a supertype cycle (for example, one table per strongly connected component, as `resolve` effectively computes today, charged per item 5 over the component), or refuse the cycle at step 1 (an admission change to flag in Q2), and add a cyclic-supertype row to AC-137 or AC-144. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2026 |
| FND-002 | low | Item 3 says `resolve` selects the named member from the table "by no second walk of the declarations". Item 6 says a read's step 3 charge "shall stay as it is today". Today's charge is produced by the walk: ancestor pops, members per owner, redefinition pairs, and the per-call ancestor memo of each redefining owner (`model_members.rs:674-760`). The spec does not say how an unchanged count is obtained without that walk, for instance a per-type step 3 charge recorded when the table is built. An implementer would otherwise keep the walk (violating item 3) or change the count (violating item 6). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2029, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2039 |
| FND-003 | low | Item 2 carries two `shall` clauses ("shall build the table ... and shall not re-walk the ancestors"), so "one shall per item" holds for 32 of the 33 items. Split it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2026 |

## Dispositions

Round 3, reviewed at 265e08258548c8138d57e140f9b9cfeedad3ef43.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 265e08258548c8138d57e140f9b9cfeedad3ef43 |
| FND-002 | fixed | 265e08258548c8138d57e140f9b9cfeedad3ef43 |
| FND-003 | fixed | 265e08258548c8138d57e140f9b9cfeedad3ef43 |
