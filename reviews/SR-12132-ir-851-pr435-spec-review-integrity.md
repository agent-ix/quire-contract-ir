---
id: SR-12132
title: "IR-851 spec integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir (PR #435); spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-053-selected-population-binding.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-053
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-851 spec integrity review

## Summary

Ticket: IR-851. Checked id allocation, cross-references, matrix consistency and consistency with FR-053 and FR-019.

- AC-262..267 were free on origin/main: `git grep FR-038-AC-26[2-9] origin/main` finds nothing, and the previous highest id was AC-261. Nothing is renumbered.
- Each new AC is `Test (TC-048)` and has one PLANNED / UNRUN tests.md row.
- `quire coverage --strict --scope.` reports 43 unbacked rows and 0 contradicted, which is the baseline.
- `quire validate --scope. 'spec/**/*.md'` exits 0 (DuplicateArchetype and inline-schema warnings only, both pre-existing).
- `cargo test --locked --test it tc_058` passes 4/4 on the branch, so leaving out the FR-019 row is correct until the code PR exports the types.
- FR-053 is left unchanged, and that is consistent: its description (:26) and AC-5 speak of edge-FIELD multiplicity, and a relationship end is not a field.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038-AC-262..267 ids | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | FR-038-AC-262.. FR-038-AC-267, Test (TC-048) |
| tests.md rows AC-262..267 | examined | spec/checked_package/matrix/tests.md | PLANNED / UNRUN (IR-851); no tagged test yet |
| tests.md FR-038 aggregate row | examined | spec/checked_package/matrix/tests.md |... FR-038-AC-257, FR-038-AC-258 through FR-038-AC-261 |
| FR-038 FR-053 split paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | FR-053 and its AC-5 name model_object_fields as the sole authority for model edge-FIELD multiplicity; that sentence is about fields and stays true |
| FR-053 description | context_only | spec/model/functional/FR-053-selected-population-binding.md | model_object_fields remains the sole authority for model edge-field multiplicity. |
| FR-053 Public boundary | context_only | spec/model/functional/FR-053-selected-population-binding.md | No item is re-exported from the quire-contract-ir root. |
| FR-053-AC-5 | context_only | spec/model/functional/FR-053-selected-population-binding.md | model_object_fields remains the only edge-field multiplicity authority, and the root crate re-exports none of the new items. |
| FR-038-AC-267 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | the model root exports none of the selected_population items (FR-053-AC-5) |
| FR-019 deferral sentence | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The code change that exports them adds that row; this specification change does not |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-267 says "the model root exports none of the `selected_population` items (FR-053-AC-5)". The model root declares `pub mod selected_population;`, so it does export them under the path `quire_contract_model::selected_population::*`. As written, the clause is false today, and a test of it would have to fail. It also cites the wrong crate: FR-053 "Public boundary" and AC-5 forbid a re-export from the quire-contract-ir root. Fix: "the model root `pub use`s none of the `selected_population` items and the quire-contract-ir root re-exports none (FR-053-AC-5)". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4614, crates/quire-contract-model/src/lib.rs:92, spec/model/functional/FR-053-selected-population-binding.md:115-116 |
| FND-002 | low | The FR-038 aggregate row in tests.md (line 21) lists every AC range through AC-261. PLANNED ranges are listed there too, with a status note: AC-239..241 and AC-165..173. IR-832's spec PR added AC-258..261 to this row . This PR does not add AC-262..267, so the aggregate row and its status prose are now stale. Fix: append the range with a PLANNED (IR-851) note. | spec/checked_package/matrix/tests.md:21 |

## Verdict

PASS WITH FINDINGS. The ids, the matrix rows, the TC-048 link, the FR-019 deferral and the FR-053 field-versus-end split are consistent. FND-001 is a false clause in an AC and must be reworded before merge.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 : AC-268 says "no selected_population item is pub use'd at the model crate root or re-exported from the quire-contract-ir root (FR-053-AC-5); selected_population stays reachable only by its module path". Verified: crates/quire-contract-model/src/lib.rs:92 has only `pub mod selected_population;` and no `pub use` of its items, and src/lib.rs re-exports nothing from it. |
| FND-002 | fixed | Round 1 : the tests.md:21 aggregate row lists "FR-038-AC-262 through FR-038-AC-268" with a "🚧 AC-262 through AC-268 (IR-851)... PLANNED / UNRUN, spec only" note. |
