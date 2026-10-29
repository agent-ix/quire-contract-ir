---
id: SR-11676
title: IR-225 spec review of the FR-038-AC-251 amendment (catalogue entry-shape selection)
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; PR 409, spec hunks of PR diff; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, tests/it/checked_package_v2_catalogue_differential.rs
review_set: subset
---

## Summary

Ticket: IR-225. PR: quire-contract-ir#409. This is a base spec review of a
spec change that arrived inside the code PR. FR-038-AC-251 used to require a
refusal when the catalogue drifted. That was unsatisfiable without a pinned
set of expected entries, so it was rewritten:

- Member-result checking is selected by the entry's shape alone
  (`result: member` and `member: type_argument`).
- Every other catalogued operation keeps its pre-IR-225 admission outcome.
- Fold and reduce get the equality check only.

The change touches four places that must agree: the FR-038 normative bullet
(FR-038 lines 3563-3576), the AC-251 row (line 4372), the TC-048 AC-251 step
(TC-048 lines 399-420) and the tests.md row (line 67). The backing test is
`tests/it/checked_package_v2_catalogue_differential.rs`.

## Verdict

**PASS with two low findings.** The amendment is faithful, falsifiable and
honest.

- **Faithful.** The FR bullet, the AC-251 row, the TC-048 step and the
  tests.md row all state the same selector (`result: member` and
  `member: type_argument`). They all keep lookup, all_instances and the other
  member kinds outside the rule, and all say fold and reduce get equality
  only. No "Withdrawn" wording remains. The status cell records the amendment
  and its reason ("amended from an expected-set shape guard because the reader
  selects from the current catalogue entry").
- **Falsifiable.** The reviewer ran both planner mutants against the backing
  tests. Selecting by member kind alone fails the differential and
  `non_member_model_results_remain_admitted`. An allow-list of the three IEEE
  identities fails the non-IEEE mismatched-result controls.
- **Honest.** The test reads `CHECKED_OPERATION_CATALOG_V1`, the same constant
  the reader embeds (operation_catalog.rs:41), so no copy of the catalogue
  lands in the repository. It pins the exact sets: 47 admitted and 113
  unavailable out of 160 entries.

**The 113 unavailable entries.** All 113 are left out because this file's
helpers cannot build them, not because of any semantic exclusion:

- 107 are operations this rule does not select. Inspection shows the selector
  returns `None` for them before any work. The member-shape change affects
  only `type_argument` members, so their admission cannot change.
- The other 6 are selected entries: `ieee.from_exact`,
  `numeric.convert_rounding` and `collection.sum.decimal/float32/float64/quantity`.
  Lacking an admitted control for these is a gap-analysis finding
  (SR-11675 FND-005), not a spec defect.

Quire state: `validate` passes 884/884, and `coverage --strict`
reports 50 unbacked rows and 0 contradicted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The TC-048 AC-251 step says a wire member kind that disagrees with the entry "still refuses earlier at `operation.member` (AC-249)". AC-249's text covers only a missing, malformed or misspelled `declaration`, not a member-kind mismatch. That rule comes from the pre-existing member-kind criteria (FR-038-AC-65, AC-69, AC-98). The amendment deleted the only IR-225 test that put a kind mismatch on a `type_argument` entry ahead of the result check. So the precedence claim is cited to the wrong criterion, and no IR-225 test backs it. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:412-414 |
| FND-002 | low | The TC-048 step asks for the unavailable set to be named "with its fixture limitation stated", and AC-251 says the test "names the exact unavailable fixture set". The test gives one blanket reason for all 113 entries ("need operands, selected laws, model declarations, or temporal/state context beyond the nominal and structural builders used here"). Several listed identities already have admitted fixtures elsewhere in tests/it: `collection.sum.quantity`, `record.project`, `model.navigate` and `model.dispatch_call`. A reader of the spec would take the set as unconstructible, when it is only unconstructible from this one file's helpers. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:399-403; tests/it/checked_package_v2_catalogue_differential.rs:579-583 |

## Examined units

```yaml
- id: FR-038-AC-251
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Member-result checking is selected by the catalogue entry shape alone (`result: member` and `member: type_argument`). Every other catalogued operation, including `model.lookup`, `model.all_instances` and the other member kinds, keeps its pre-IR-225 admission outcome; `collection.fold` and `collection.reduce`, which have no `member_family`, are checked for result-type/declaration equality only. A catalogue-wide differential test reads every entry from the same published catalogue as the reader, requires admission for each constructible control, and names the exact unavailable fixture set.
- id: FR-038/Member-result-bullet
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Selection follows the catalogue entry shape alone (`result: member` and `member: type_argument`). Every other catalogued operation, including `model.lookup`, `model.all_instances` and other member kinds, keeps its prior admission outcome. `collection.fold` and `collection.reduce` have no `member_family` and receive only the result-type/declaration equality check.
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '... regardless of whether its family resolves or the entry has a `member_family` constraint ... a simultaneous operand defect wins over a result defect, while a simultaneous genuine rounding-mode type defect on `quantity.convert` does not displace the `result_type` refusal.'
- id: FR-038-AC-249
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `type_argument` member missing `declaration`, carrying a malformed declaration, or carrying `type_argument` instead of `declaration` refuses `invalid_package`/`operation-member-mismatch` at `/semantic_graph/nodes/{i}/body/operation/member`, with the application node as locus.
- id: TC-048/AC-251-step
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: For AC-251, read every entry from the published catalogue used by the reader. Build a fresh, previously admitted package for each identity supported by the fixture helpers and require continued admission. Compare the remaining identity set with an explicitly named and counted unavailable set, with its fixture limitation stated, so a new catalogue entry cannot silently escape the test.
- id: tests.md/FR-038-AC-251
  path: spec/checked_package/matrix/tests.md
  role: examined
  excerpt: Catalogue-wide differential admission test reads every entry from the reader's published source, admits every constructible pre-IR-225 fixture, and compares the explicitly named, counted unavailable set.
```

## Dispositions

Round 2 (the first disposition pass for this artifact). Verified at
 by reviewing
`git diff origin/main...HEAD`.

The differential's denominator is stated the same way in the test
message, TC-048 and tests.md: "behavioral controls for 56 of 160 entries",
explicitly "not a complete baseline". No "full baseline" claim remains.

The 104 unavailable entries are split as follows:

| Class | Count | Entries |
| --- | --- | --- |
| control, temporal, state, claim and protocol context | 25 | control.*, temporal.*, state.*, claim.clause, protocol.control |
| IEEE profile and typed operands | 15 | IEEE arithmetic and comparison |
| "selected model declarations" | 3 | model.deref (`inner:0`, no member), model.reaches (boolean, `relationship_end`), model.reaches_field (boolean, `field`) |
| other typed operands or member fixtures | 61 | string, uuid and timestamp equality; collection constructors and higher-order operations; decimal, enum, text, quantity, option, reference and structural operations; integer.div/rem; function.call |

In the third class, "selected" refers to lock model selections. None of
those three is a selector-selected entry. In the fourth class, the reason
is the classifier's default arm, so it is a residual bucket. It is still
accurate as "not built here", though imprecise for integer.div/rem (which
need an `integer_division` law) and function.call (which needs a function
declaration). No entry in it can be confused with the selector's shape;
the new assertion enforces that.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The TC-048 citation now reads AC-69 and AC-98, with AC-65 for the vocabulary. AC-69 ends "every member that disagrees with its entry refuses alike", which covers a `position` member on a `type_argument` entry. The restored test `tc_048_member_kind_mismatch_precedes_member_result_check` (fold control, `position` member, wrong result) expects `invalid_package`/`operation-member-mismatch` at `operation.member`, with exact code, cause, pointer and locus. Mutant Q3 (result check before the member-kind check) is killed by this test. |
| FND-002 | fixed | The single blanket reason is replaced by four per-class reasons from `unavailable_class`. The test pins the count of each class (25/15/3/61) and the exact 104-name set, and asserts that no unavailable entry is a member-result or `type_argument` entry. The denominator wording no longer implies the set is unconstructible in general. |
