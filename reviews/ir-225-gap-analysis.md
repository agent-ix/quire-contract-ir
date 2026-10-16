---
id: SR-11675
title: gap-analysis and test-oracle strength of IR-225 catalogue-driven member-result admission
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir@9e44b4b21063e5ea2c5e6f25520a9bce0301e8b3; PR 409 branch code/ir-225-member-family; diff origin/main...9e44b4b (base ef4ab49);
  crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it (5 files), spec/checked_package (FR-038, TC-048, tests.md)
review_set: subset
---

## Summary

Ticket: IR-225. PR: quire-contract-ir#409. Method: quoin gap-analysis (planless;
"Plan completion: not assessed") with test-oracle strength folded in. The
computed trace (`quire trace --id`) binds the following tests:

- FR-038-AC-247: tests/it/admitted_result_context.rs:137,
  tests/it/checked_package_v2_ieee_admission.rs:528 and
  tests/it/checked_package_v2_operand_families.rs:1015.
- FR-038-AC-248: ieee_admission.rs:528 and operand_families.rs:1015, :1128 and
  :1167.
- FR-038-AC-249: ieee_admission.rs:557.
- FR-038-AC-250: admitted_result_context.rs:137 and operand_families.rs:1015
  and :1070.
- FR-038-AC-251: operations.rs:8924 (private-guard lib test).

`quire coverage --strict` reports 50 unbacked rows and 0 contradicted.
`quire validate` reports 884/884 docs. No new production code lacks an owning
requirement.

Mutants were applied one at a time in a throwaway worktree. Each was run
against the 7 new `it` tests, AC-223's
`tc_048_cross_width_ieee_result_is_the_type_argument_of_the_catalogued_family`,
and the new lib test:

| Mutant | Result |
| --- | --- |
| M1 restore the IEEE to_float32/to_float64 allow-list | killed (to_rational, numeric.narrow, quantity.convert, lib) |
| M2 drop `result == declaration` equality | killed |
| M3 drop the `member_family` fit | killed |
| M4 apply the family check to every deferred form | killed (collection.size model_population) |
| M4b drop only the `model_population` skip | killed |
| M4c drop the `compound_unit` and `alias` skips | **survived** (equivalent: `operand_family` is `None` for both) |
| M5 apply equality to `position`/`element` member results | killed |
| M6 accept the old `type_argument` key | killed |
| M7 select only the first 10 catalogue entries (skip every `quire.op.collection.*`) | **survived** |
| M8 run the result check before `check_operands` | **survived** |
| M9 run the result check after `check_mode_type` | **survived** |
| M10 fail open on a missing `member_family` | killed (lib) |
| M11 skip equality when the declared family is unresolved | **survived** |
| M12 allow extra keys in the `type_argument` member | killed |

Public-reader probes, admitted at main and refused at HEAD (see SR-11674):
collection.fold, collection.reduce, model.lookup and model.all_instances.
There is no admitted fixture anywhere for ieee.from_exact,
numeric.convert_rounding, collection.sum.decimal, collection.sum.float32 or
collection.sum.float64.

## Verdict

**FAIL (one high, two medium, one low).** Tags are truthful: each tagged test
asserts its criterion with exact code, cause, pointer and locus. The AC-249
cases are exact. The AC-250 negative controls kill the position/element
mutant. The AC-251 private-guard test kills the fail-open mutant. AC-222,
AC-223 and AC-246 text is unchanged.

The tests cover only to_rational, numeric.narrow, quantity.convert and the
admitted collection.size control out of the 21 member-result entries, and none
of the type-argument entries outside that set. That gap let the fold, reduce,
lookup and all_instances regression pass a 443-test green gate. The
`Implemented` status on AC-247 is therefore contradicted for fold and reduce.
The ordering assertion in AC-247 is not exercised: M8 and M9 both survive.

## Examined scope

```yaml
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Through `CheckedPackageV2::read`, a catalogued `result: member`/`member: type_argument` application whose member has a valid `declaration` admits only when `result_type` equals that declaration, regardless of whether its family resolves. ... After earlier member-shape and operand checks, this result refusal precedes `result_inner`, state-clause result, model-member, mode-type and leaf checks; a simultaneous wrong rounding-mode type on `quantity.convert` does not displace the `result_type` refusal.
- id: FR-038-AC-248
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For a catalogued `type_argument` member result with `member_family`, a declared type whose family the reader resolves under the FR-322 ruling must fit the catalogue family; ... An otherwise valid `quire.op.collection.size` whose `type_argument` declaration is a `model_population` node and whose `result_type` equals that declaration admits.
- id: FR-038-AC-249
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `type_argument` member missing `declaration`, carrying a malformed declaration, or carrying `type_argument` instead of `declaration` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{i}/body/operation/member`, with the application node as locus.
- id: FR-038-AC-250
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Through the public reader, valid `quire.op.numeric.narrow` and `quire.op.quantity.convert` applications each admit with equal member declaration and `result_type`, and each refuses a mismatched `result_type` ... valid `tuple.project` and `collection.index` packages with member declarations distinct from their `result_type` remain admitted.
- id: FR-038-AC-251
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: If a currently catalogued `type_argument` member-result entry changes away from `result: member`/`member: type_argument`, or loses a required `member_family`, the reader refuses the typed package.
- id: tests.md/FR-038-AC-247..251
  path: spec/checked_package/matrix/tests.md
  role: examined
  excerpt: '| FR-038 | FR-038-AC-247 | TC-048 | Implemented (IR-225). | Result/declaration equality for every type-argument member result; ...'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Coverage gap: no test admits any type-argument entry outside to_rational, numeric.narrow, quantity.convert, collection.size and the pre-existing sum.quantity and to_float32/to_float64 cases. In particular, nothing covers fold, reduce (member result without `member_family`), lookup or all_instances (`type_argument` with a non-member result). The selection regression recorded in SR-11674 FND-001 and FND-002 therefore passes all 443 `checked_package_v2` tests, and M7 (skip every `quire.op.collection.*` entry) survives. AC-247 says "every type-argument member result". | tests/it/checked_package_v2_operand_families.rs:1012-1210; tests/it/checked_package_v2_ieee_admission.rs:525-576 |
| FND-002 | medium | Status honesty: FR-038-AC-247 and its tests.md row read `Implemented (IR-225)`, but a valid `collection.fold` or `collection.reduce` with `result_type` equal to its declaration is refused at HEAD. The criterion is contradicted for 2 of the 21 entries, not implemented. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4369; spec/checked_package/matrix/tests.md:65 |
| FND-003 | medium | Vacuous ordering oracle: the two-defect `quantity.convert` case pairs a wrong result (`integer`) with a `toward-zero` mode. Under an `integer` result that mode is not a mode-type defect, so moving the result check after `check_mode_type` (M9) survives. Moving it before `check_operands` (M8) also survives, because no case combines an operand defect with a result defect. AC-247's ordering clause is asserted by no test that can fail. | tests/it/checked_package_v2_operand_families.rs:1030-1063 |
| FND-004 | low | AC-247 says equality holds "regardless of whether its family resolves", but no adverse case has an unresolved declaration (compound_unit, alias, model_population) with an unequal `result_type`. Mutant M11, which skips equality when the family is unresolved, survives. | tests/it/checked_package_v2_operand_families.rs:1125-1210 |

## Dispositions

Round 1. Verified at 9e7fe57352f25477d3c7d4b359da450cb17ce529 by reviewing
`git diff 9e44b4b..9e7fe57`.

The whole mutant set was rerun against the 11 IR-225 tests. The tests
include the new `tc_048_every_catalogue_entry_retains_its_constructible_admission`,
`fold_and_reduce_admit_without_member_family_and_refuse_unequal_result`
and `non_member_model_results_remain_admitted`. All 16 mutants are killed:
M1, P2 (three-IEEE allow-list), P1 (member kind alone), M2, M3, M4, M5,
M6, M7, M8, M9, M10, M11, M12 and M13. The former M4c no longer applies
because the arms it deleted were removed (SR-11674 FND-003).

The differential test reads `CHECKED_OPERATION_CATALOG_V1`. That is the
same constant `operation_catalog.rs:41` reads, and nothing is copied into
the repository.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Four kinds of coverage were added. A catalogue-wide differential builds 47 admitted controls, including fold, reduce, lookup and all_instances. Fold/reduce tests check equal and unequal results. A lookup/all_instances test checks admission. There are also mismatched-result refusals for count, sum.integer, collection.convert, numeric.convert, rational.narrow, quantity.convert and numeric.narrow. M7 is killed. The remaining selected entries without a control are recorded as FND-005. |
| FND-002 | fixed | AC-247 now states that fold and reduce admit equal results without a family constraint, and the code does admit them (probe and test). The `Implemented` status is now backed. |
| FND-003 | fixed | The mode case now pairs an `integer` declaration with a `unit` result and `toward-zero`, which is a true mode-type defect (AC-213). A new case pairs an empty `arguments` array with a wrong result and expects the refusal at `/body/arguments`. M8 and M9 are both killed. |
| FND-004 | fixed | Unequal results now refuse for a `compound_unit` declaration (unit result) and a `model_population` declaration (integer result). M11 is killed. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | Six of the 21 selected type-argument member-result entries have no admitted control in the catalogue differential: `ieee.from_exact`, `numeric.convert_rounding`, `collection.sum.decimal`, `collection.sum.float32`, `collection.sum.float64` and `collection.sum.quantity`. All six fall under the blanket "unavailable" reason. Five have no admitted fixture anywhere in the repository. `collection.sum.quantity` does have one (`quantity_sum_uses_its_result_type_exact_rounding_pin`), yet the differential lists it as unavailable. If equality or `family_fits` were wrong for the umbrella families of these five (`ieee_float`, `decimal`, and IEEE profile laws), the new rule could refuse every valid package of that kind and no test would fail. | tests/it/checked_package_v2_catalogue_differential.rs:579-702; tests/it/checked_package_v2_operand_families.rs:1374 |

Round 2. Verified at f30fd77bb53ef69a268dbbe0c8fbf92fa4ddc7fc by reviewing
`git diff 9e7fe57..f30fd77`.

**Selected entries.** Enumerated from the locked catalogue, all 21
`result: member`/`member: type_argument` entries now have an admitted
control in `admitted_fixture`. The test's second loop filters the catalogue
by that shape, so a missing control fails it through `expect`. Each control
is also given a mismatched result, which must refuse at `result_type`.

**Base-reader check.** With ef4ab49's `operations.rs` and `temporal.rs`
swapped in, every one of the 56 controls admits. The test then fails only
in the refusal loop, as expected. The six new controls were therefore
admitted before IR-225 too.

**Lookalikes.** The seven lookalike entries all have admitted controls:
- `result: member` with another member kind: record.project, tuple.project,
  collection.index, model.navigate and model.dispatch_call.
- `member: type_argument` with another result: model.lookup and
  model.all_instances.

A new assertion requires every unavailable entry to have neither
`result: member` nor `member: type_argument`, so no entry the selector
could confuse is left uncontrolled.

**Mutants.** All 18 are killed:
- The round-1 set: M1, P1, P2 and M2 to M13.
- Q1, a broken `family_fits` for `ieee_float`/`decimal`, killed by the
  from_exact, convert_rounding and sum.decimal controls.
- Q2, dropped `collection.sum.*` equality, killed by the selected-entry
  refusal loop.
- Q3, result check moved before the member-kind check, killed by the
  restored precedence test.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-005 | fixed | All six entries now have admitted controls in the differential: from_exact and the two IEEE sums with an `ieee_profile` law, convert_rounding, sum.decimal, and sum.quantity through `quantity_sum_control()`. Each also has a mismatched-result refusal. All six admit under the base reader as well. |
