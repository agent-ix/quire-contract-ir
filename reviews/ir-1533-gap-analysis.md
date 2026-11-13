---
id: SR-2026111317273300000002
title: "gap-analysis of IR-1533 reaches_population over reaches_field"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #1020 at d082fe9d; FR-038 Typed population of a reaches application (field-edge link), FR-038-AC-542..544; tests/it/checked_package_v2_model_members/reaches_population.rs; crates/quire-contract-model/src/checked_package/v2/{operations.rs,reaches_population.rs,relationship_end.rs}"
review_set: subset
---

## Summary

Gap analysis for quire-contract-ir PR #1020. Ticket: IR-1533. Plan completion: not assessed. This was a manual check of acceptance criteria against tests. `quoin matrix` was not run, because it would need a build and the build lanes were busy. Each new AC has exactly one tagged test, and each test is tagged with the AC it asserts.

The ticket's acceptance items were read as data and judged against the FR text:

1. The accessor resolves Reference, Option<Reference> and bounded Sequence<Reference> fields. AC-542 covers these, plus an inherited field. The code meets it, but the test cannot pass at head (code-review SR FND-001, FND-002).
2. The ticket says "a reaches_field without the link keeps the typed refusal". The PR returns `Ok(None)` (AC-543). That matches the FR, where the absence of the key means no selected binding. It is also the reaches behaviour (AC-269/270), and `NotReachesApplication` is kept for nodes that are not reaches applications. A literal reading, refusing `NotReachesApplication`, would make the same node a "reaches application" or not depending on an optional key. So this reading is sound, and the deviation from the ticket wording is recorded here, not raised as a finding.
3. The ticket asks for tests over packages from the QSL producer. None are present (FND-001).

The extra reader admission check is within scope. The accessor's contract says admission refuses every state that `UnresolvedPopulation` reports. Without a reader check, the accessor would hand FR-053 a selector over an unchecked node, or one of the wrong type. On main, a `population` key on a `reaches_field` field member is admitted unread. The only packages the change can newly refuse are ones that carry such a key with an invalid value. QSL origin/main (local ref 5893d92cd) emits none: the producer is QSL-1904, which is not merged.

## Examined scope

- FR-038-AC-542 (examined): tc_048_reaches_population_reads_each_reaches_field_form.
- FR-038-AC-543 (examined): tc_048_reaches_population_over_a_reaches_field_without_a_link_is_none.
- FR-038-AC-544 (examined): tc_048_a_reaches_field_population_of_another_type_than_the_owner_is_refused.
- FR-038-AC-275 and FR-038-AC-276 (context_only): the existing field-member and dependency-join outcomes. The AC-275 test uses `quire.op.record.project`, not `reaches_field`, so it is unaffected.

## Verdict

AC-to-test traceability is correct for all three new ACs. The tests fail at head for the code defects recorded in the code-review SR. The ticket's producer-package acceptance item is not met (FND-001). The least-ordinal clause of AC-542 is not exercised (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Ticket acceptance item 3 asks for tests over checked packages from the QSL producer ticket (QSL-1904). The PR tests only hand-built fixtures, and the PR does not say this item is deferred. Either add a conformance test over a QSL-produced package once QSL-1904 lands, or record an explicit deferral with a follow-up on IR-1533. | tests/it/checked_package_v2_model_members/reaches_population.rs:1173-1255 |
| FND-002 | low | AC-542 requires "its least-ordinal `Type` occurrence", but `link_field_population` adds the population with the single ordinal `[0]`, so least-ordinal selection is never exercised for `reaches_field`. The reaches tests use `&[4, 1, 7]`. Add more than one ordinal in at least one case and assert the least. | tests/it/checked_package_v2_model_members/reaches_population.rs:1158, 1192 |
