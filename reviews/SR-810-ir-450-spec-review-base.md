---
id: SR-810
title: "spec review of PR 257 quantity bound class (IR-450)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@10a17ec6b705ec88b5442a3367b826043d76ed2b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md; spec/checked_package/matrix/tests.md"
review_set: subset
---
# SR-810: spec review of PR 257 quantity bound class

## Summary

Ticket: IR-450. Base and correctness review of `git diff origin/main...HEAD`. The PR is
based on origin/main 1a59efc and merges cleanly. It changes the FR-038 prose on unbounded
forms, amends FR-038-AC-8, adds FR-038-AC-73 (Test TC-050, planned), the TC-050 procedure
section and three matrix rows. `make spec` gives the same result before and after: grammar
312/313 (baseline 1) and strict 23 unbacked (baseline 23).

Code, measured at the reviewed head. `requires_bound(kind)` (`lower.rs:562`) returns false
for `ScalarTypeForm::Unit` and `ScalarTypeForm::CompoundUnit`. In `lower_one`, a typing edge
from a `composite_type` node, or edge 0 (`semantic_type`) of a `value`/`parameter` node, adds
its target to `positions`. Every other typing edge (another node's `semantic_type`,
`dependencies`, a body `reference` target or `application` `result_type`) adds it only to
`typed`. A `literal.type` edge is `Annotation` and adds neither. The requested node is always
in `typed`. The rule the PR states can be implemented as a third predicate: a quantity raises
when it is in `positions` and never through `typed`. It must not be implemented by returning
true from `requires_bound` for the two forms, because that also sends them through the
`typed && !is_bounded` arm (SR-811 FND-001). The nominal fixture TC-052 lowers holds no
composite or parameter, so its `unit` node, lowered as the requested node, still lowers under
the new rule.

QSL agreement, measured against QSL origin/main 8d1deba4. QSL FR-097-AC-2 and ADR-014 §4
count each quantity in the closure of a claim's argument and bound-variable types as one
unbounded domain that takes no finite bound. The ignored TC-440 test
(`qsl-package/src/emit/extent_agreement.rs`, `Measure{len: metre}`) asserts only that IR
returns `RequiresBound` and that QSL names a `Quantity` domain. It does not check which node
IR names. Under the PR's rule, `Measure`'s field is a composite position, so IR raises,
naming `metre`. `Trip{d: km, v: mps}` raises at `km` and `mps`, and not at `km`'s target
`metre`. A collection, option or alias of a quantity raises at the composite. A parameter or
a query, `fold` or `reduce` binder (each a `value`/`parameter` node) raises at its
`semantic_type`. So an application over quantity parameters raises, as QSL's record
(Unbounded at those parameters) requires. A constant quantity application such as `1 m + 2 m`
is `Bounded` in QSL, because literals contribute no roots, and IR lowers it, because neither
`result_type` nor a literal's type is a position. That is why the rule must be position-only.
"Also refuse when merely reached" would make IR disagree here and on every requested unit.
The one divergence is a `let` binder of quantity type, a parameter node, which raises in IR.
QSL FR-097-AC-6 already places it outside the agreement, as it does for integers.

On the open question: AC-73 says IR names "its `unit` or `compound_unit` node", meaning the
node at the position. That is the right statement. It lets QSL move `Measure` into the main
TC-440 table with `Form("unit")` once the code lands. QSpec origin/main c76c6ae states no
bound class for quantity. FR-142 and TC-187 are evaluation rules only. FR-322 says only that
a `unit` is family `quantity`, "read from the type the node resolves to through its
`bounded_domain` base chain", which is what FND-001 turns on.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The spec does not say what happens when a position names a `bounded_domain` whose base is a quantity. The reader does not constrain a `bounded_domain`'s base type: `structural_type` forwards through it, and QSpec FR-322 reads the family through that chain, so a unit base is family `quantity`. A record field typed at, say, a `rational_range` whose `semantic_type` is a `unit` node names the domain, not the unit, at the position. The unit is reached only through the domain's own `semantic_type`, which the prose says is not a position, so under the stated rule the record lowers. QSL FR-097-AC-2 says a quantity's domain takes no finite bound. The prose asserts both that "no `bounded_domain` form ranges over" a quantity and that "a `bounded_domain` never covers a quantity position", and two implementers would read those differently for this shape. State it: either a `bounded_domain` whose base chain resolves to `unit`/`compound_unit` is a quantity at the position (and say which node the record names), or the reader refuses such a domain. Add the case to AC-73. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1049-1058; crates/quire-contract-model/src/checked_package/v2/operations.rs:1667-1690 |

## Verdict

Request changes (medium). The position-only rule is coherent with the lowering code and is
the rule QSL agreement needs. The open questions resolve as follows. The scope should stay
position-only. AC-73 should keep naming the `unit` or `compound_unit` node at the position.
The QSL Measure comparison needs no more than `RequiresBound`, and naming the node lets QSL
assert the form. Fix FND-001 here, together with SR-811 FND-001, before the code PR is cut.

## Dispositions

Round 1 at 4b19dbe6da4c34665076d325b4d7d385d265fa50 (base still origin/main 1a59efc, merges cleanly). FR-038 now defines a quantity as a `unit` or `compound_unit` `scalar_type`, or as a `bounded_domain` whose base chain ends at one. A position typed at a quantity raises `requires_bound`, either directly or through that chain, and names the `unit` or `compound_unit` node at the end of the chain. I checked this against `lower.rs` at the reviewed head. A position is the type node a composite or parameter edge names, which here is the `bounded_domain`. That domain is visited, and its edge 0 (`semantic_type`, its base) is a typing successor, so every node on the chain is in the closure and `ordered`. The unit at the end of the chain is therefore a reachable node that the record can name. The least-key rule then picks among the named chain ends.

QSL FR-097-AC-2 ("a quantity's domain takes no finite bound") classifies such a type as Unbounded, so the two now agree. QSL never emits this shape. The QSL cases from round 0 still hold: Measure, Trip, collections, parameters and binders raise, and an application with a quantity `result_type` over bounded parameters lowers. A note for the code PR: the chain walk must be bounded and safe against cycles, as `operations.rs` `structural_type` is, and must exclude `model_population`. No new findings.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed 4b19dbe6da4c34665076d325b4d7d385d265fa50 |
