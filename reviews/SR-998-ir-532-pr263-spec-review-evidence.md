---
id: SR-998
title: "evidence review of PR 263 (FR-038-AC-81 to AC-88, TC-048, tests.md)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@376269e0d67ee1451ee9629dbe897c06a8a730de; FR-038-AC-81 to FR-038-AC-88 verification methods; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md FR-038 and TC-048 rows; tests in crates/quire-contract-model/src/checked_package/v2/operations.rs and tests/it/checked_package_v2_reader.rs"
review_set: subset
---
# SR-998: evidence review of PR 263

## Summary

Ticket: IR-532. Every new AC is `Test (TC-048)`, which fits refusal-pointer
criteria. The matrix was measured with `make spec` at head and at origin/main:

- Validate passes, with the grammar baseline unchanged (1 finding, FR-014).
- Coverage goes from 174/220 to 174/228. FR-038 goes from 53/78 to 53/86.
- Strict coverage reports 23 unbacked rows before and after, and the sets are
  identical (diffed from the `--json` output). `make spec` exits 1 on main for
  the same 23.

The FR-038 row's range now runs through AC-88, and the TC-048 row lists AC-81 to
AC-88. Both say planned and untagged, which is accurate. The PR body's numbers
match these measurements.

Notes for the binding PR (outside this diff, not findings):

- The `operations.rs` module doc, lines 29-56, is stale. It still says
  `argument_family` resolves only `reference` and `binding`, and that `literal`
  and nested `application` operands bypass the family checks. Both now resolve,
  as lines 675-694 and AC-85 show.
- The doc comment at 3628-3635 says the field's type "pins `rounding` to
  `nearest-even`". The fixture pins `text_profile` `nfc`.
- The doc comment at 2999-3003 claims the preimage is QSL's pinned vector (see
  SR-994 FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-81 to AC-87 are worded as reader behaviour ("An application ... refuses"). Their only evidence is unit calls of `operation_defect` on fixtures the reader's grammar would refuse: `literal` terms with no `type` or `value_kind`, and old-shape artifact references. AC-81's "admits" control could not pass a full read. AC-68 already uses the honest form, "observed on that node (a unit-level check of the node's operation step)". The new ACs should say the same, or gain reader-level evidence | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1403-1409; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:287-330 |
| FND-002 | low | End-to-end evidence for AC-88 already exists but is traced to the wrong AC. `tc_048_an_application_node_in_a_recursion_group_keys_by_fr322_ordinals` (tests/it/checked_package_v2_reader.rs:2499-2593) asserts graph-order group keys and `stale-node-key`, and is tagged FR-038-AC-17, whose text says nothing of keys. The PR's enumeration and TC-048's AC-88 procedure ("as unit tests of operations.rs") omit it. The binding PR should retag it to AC-88 | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:35-38 |

## Verdict

The matrix bookkeeping is correct and honest. The evidence level should be stated
as unit-level, or reader-level evidence should be added. The existing integration
test should be bound to AC-88.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9d4379b |
| FND-002 | deferred | A trace tag lives in code, so the retag of tests/it/checked_package_v2_reader.rs:2502 from FR-038-AC-17 to FR-038-AC-88 is the binding PR's (IR-532 code, after #253); the PR body's Binding PR scope lists it. |
