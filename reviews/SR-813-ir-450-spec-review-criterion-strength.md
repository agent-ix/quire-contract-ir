---
id: SR-813
title: "criterion strength review of PR 257 quantity bound class (IR-450)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@10a17ec6b705ec88b5442a3367b826043d76ed2b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md"
review_set: subset
---
# SR-813: criterion strength review of PR 257 quantity bound class

## Summary

Ticket: IR-450. This review asks whether FR-038-AC-73 and amended AC-8 can fail. AC-73's
raising cases fail against today's reader, which lowers all three, so the criterion is not
tautological. Its lowering cases (the requested `unit`, the requested `compound_unit` with
its dependencies, and the annotation-only unit) are the ones that catch the wrong
implementation, one that returns true from `requires_bound()` for the two forms (SR-811
FND-001). Under that implementation a requested unit is in `typed` with no covering domain
and raises. Two clauses are weaker than they look.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-73's clause "a `bounded_domain` whose `semantic_type` is that node does not cover it" cannot discriminate. No `bounded_domain` covers a position under any implementation, because `positions` is tested before `is_bounded`, and `is_bounded` only reads the reachable closure. TC-050's procedure says "with and without a `bounded_domain`" but does not require that domain to be reachable from the requested node. A fixture whose domain is unreachable passes trivially. Require the domain to be reachable, for example typed by a second field or parameter, so the "with" case exercises `is_bounded`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1179; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md:72-81 |
| FND-002 | low | The prose names four cases where a quantity does not raise: a requested unit, a `compound_unit`'s dependencies, a `literal.type` annotation, and an `application`'s `result_type`. AC-73 tests only the first three. The `result_type` case is the one QSL agreement depends on for a constant quantity application, which QSL records as `Bounded` because literals have no roots. It is also the case an implementation that adds `result_type` edges to `positions` would break. Add a function whose body applies an operation with a quantity `result_type`, over no quantity-typed parameter, and assert that the application lowers. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1053-1058, 1179 |

## Verdict

Approve with changes. AC-73 can fail and catches the main wrong implementation. FND-001 and
FND-002 close the two gaps.

## Dispositions

Round 1 at 4b19dbe6da4c34665076d325b4d7d385d265fa50. The clause that could not fail is replaced. AC-73 and TC-050 now cover a record whose field is typed at a reachable `bounded_domain` over a `unit`. That record must raise and must name the unit rather than the domain. Both today's reader and a predicate that tests only direct positions would lower it, so the case can fail. A case is also added for an application whose `result_type` is a quantity, over parameters typed at a bounded type, and it must lower. This matches QSL, which records it Bounded: its roots are bounded and literals add no domain. It also fails an implementation that adds `result_type` edges to the positions. No new findings.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed 4b19dbe6da4c34665076d325b4d7d385d265fa50 |
| FND-002 | fixed | fixed 4b19dbe6da4c34665076d325b4d7d385d265fa50 |
