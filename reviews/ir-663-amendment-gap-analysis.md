---
id: SR-2943
title: "Gap analysis of the IR-663 private intake retention amendment"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR 321 (IR-663 amendment, specification only); FR-038-AC-167, FR-038-AC-173, FR-038-AC-174, FR-038-AC-175; TC-048"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-663. Planless gap analysis scoped to the four amended criteria.
`quire matrix --format tsv` at base and at head: 336 criteria in both, the
same identities, statuses (tagged 301, untagged 31, tagged-by-ignored-test 2,
method-without-symbol 2) and binders; only the AC-167/173/174/175 statements
differ. `quire matrix --strict` exits 1 at both base and head, from the same
pre-existing untagged set, so the strict result is no worse. AC-174 and AC-175
are untagged, which matches their PLANNED / UNRUN status. AC-167 and AC-173
are tagged by existing relationship-refusal tests that cannot observe
retention, so the retention clauses this amendment keeps in those two
criteria appear covered when they are not.

## Verdict

CONDITIONAL for this change: there are no regressions and only medium
findings. The repository-wide matrix stays FAIL on the 31 untagged criteria
already present at base. AC-174/175 are deliberately untagged (PLANNED).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Coverage inflation: AC-167 and AC-173 now carry private-retention obligations ("At the private `admit_selection` refusal return, a nested relationship retains its own authentic supplied identity and valid origin"; "the private intake refusal retains any actual supplied declaration identity ...") but compute `tagged` from `tc_048_relationship_admission_refuses_bad_identity_shape_and_end_meaning`, `tc_048_relationship_roles_and_metadata_are_read_from_the_authored_declaration` and `tc_048_selected_relationship_declaration_refusals_precede_navigation`. None of these can assert identity or origin, because `SelectionRefusal` has no such fields. Keep retention only in AC-174/175 and cross-reference it from AC-167/173. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3799,3805; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:240,298-305; tests/it/checked_package_v2_model_members.rs:860-867 |

## Coverage

- Criteria in scope: 4. Tagged: 2 (AC-167, AC-173; the retention clauses in both are unbacked, see FND-001). Untagged: 2 (AC-174, AC-175; PLANNED / UNRUN by design).
- Matrix delta base to head: 0 identities added or removed, 0 status or binder changes, 4 statement changes.
- Property classification (informational, `quire properties`): extractable 101/179 to 100/179; invariant shapes 26 to 25.
- Semantic review: skipped (specification-only change; no code under review).
- Plan completion: not assessed

## Dispositions

Round 1, fix commit "spec: close IR663 private intake review findings".

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
