---
id: SR-1560
title: "Integrity review of quire-contract-ir PR #296 (IR-628 typed model object fields accessor)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@7ffa956c25fe491767cb790d8168e958929000e4; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Typed accessor for a model object type's fields' items 1-15, FR-038-AC-136..143), spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md"
review_set: subset
---

## Summary

Ticket: IR-628. I checked the fifteen decided items and FR-038-AC-136..143 for internal consistency,
for consistency with the reader code they cite, and for whether each criterion can be executed and
can fail.

- Items 1, 2, 3, 7, 8, 9, 12 and 13 are consistent with the code. `field_type` is the function
  `check_model_member` calls (`operations.rs:1825`). `MemberType` holds exactly `Boolean`,
  `Integer`, `IntRange(i128, i128)`, `Reference`, `Option` and `Collection{kind, element,
  bounds: Option<(u64, u64)>}`. Member names are `member_name(identity)`, and a `BTreeMap<&str, _>`
  over them is bytewise UTF-8 order.
- Items 6 and 10 match `resolve` (two unhidden same-name members give `ambiguous`) and
  `slot_type`/`element_type` (`None` for a positive lower bound with no upper bound, an interface,
  an unbound value type, or other meanings).
- AC-136, AC-137 and AC-138 are executable, and their mutation rows can be caught as stated.
  `check_model_member` refuses a mismatched `result_type` as `ill_typed`/`operator-ineligible` at
  `result_type` (`operations.rs:1871`), as AC-137 says.

Model declaration nodes are not swept at admission. `recover` runs only when a read, frame entry,
state clause or relation names the node; `identity.rs` and `structural.rs` re-derive no key for
`model`/`object_type`. So an unread `model`/`object_type` node with a selected key and a non-empty
body, or with no selected key, is admitted. AC-139's cases can be built for that reason. The same
fact puts items 4 and 11 in conflict (FND-001).

## Verdict

Changes requested: one high and two medium findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Item 11 contradicts item 4. Item 11 says the accessor "shall read no node body" and "shall return the same fields for two packages that differ only in a node body that admission accepts". Item 4 requires `ModelOwners::recover`, which reads the passed node's body and fails unless it equals `aggregate{[]}` (`model_members.rs:908-912`). Admission does not run `recover` on an unread model declaration node, so it accepts both bodies. Two such packages give `Ok(fields)` and `NotModelObjectType`; AC-139 itself pins the second outcome ("one whose key matches but whose body is not `aggregate{[]}`"). Item 11 needs to exclude the passed node's own fixed-shape check. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2064, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2028 |
| FND-002 | medium | AC-139's ambiguity case cannot be executed as written. It asks for `AmbiguousField` "in a package where a read of `x` is refused `ambiguous_declaration`/`ambiguous-name`", but a refused package yields no `CheckedPackageV2` to call the accessor on. The intent seems to be two packages: an admitted one with no read of `x`, where the accessor returns `AmbiguousField("x")`, and a sibling with a read of `x`, which admission refuses. The criterion should say so. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2791 |
| FND-003 | medium | AC-140 becomes vacuous once IR-627's stage lands, and IR-627's code change is already in progress (branch `code/ir-627-structural-key-rederivation`). With that stage, the tampered package is refused, so the only executable half is "the unmutated package's result is unchanged". The mutation row "a reader that returns the bounds of the node a read names" is then caught by nothing, because in every admitted package that node's bounds equal the declared ones. To keep the regression independent of landing order, the criterion needs a form that does not depend on admitting a tampered package, for example a crate-internal test that pairs a retained model with a graph whose read node body differs. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2792 |
| FND-004 | low | AC-141's refusing half is not defined for every field it covers. It names `Reference` fields, but the perturbation is defined only for ranges ("`upper` increased by one") and collection bounds; a `Reference` has neither. For the `i128::MAX` upper of AC-138, "increased by one" overflows. Define a `Reference` perturbation (another object type's key) and use `upper - 1` at the maximum. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2793 |
| FND-005 | low | Item 5 says the effective fields are "produced by [`DomainModel::resolve`'s] selection and by no second implementation". Today `resolve(node, kind, name, budget)` returns one member by name and charges a `Budget` on a `WorkMeter`, and item 15 says the accessor charges no work limit. Meeting both means extracting `resolve`'s effective-member computation into a shared function, or running it under an unlimited meter. The spec should name that refactor so that an implementer does not enumerate names by a second walk. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2035 |


## Dispositions

Round 1, reviewed at bd47f6aa58501e5d88afff2657f634f9f2b4f382.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-002 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-003 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-004 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-005 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
