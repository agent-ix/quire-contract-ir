---
id: SR-2701
title: "IR-654 gap-analysis of the QSpec selection-evidence conformance harness"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #319; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-176), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md, tests/conformance_qspec/main.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, Makefile; ticket IR-654"
review_set: subset
---

## Summary

Ticket: IR-654, CODE stage, PR #319. Planless gap analysis; plan completion was not assessed.
The reviewed commit identity is recorded only in the Linear marker. Model `claude-opus-5-5`;
run `bb45ccd3-2be1-4a34-8a7d-e6ec32c95115`.

The reviewer ran `quire coverage --scope . --strict` on the PR head (quire 0.36.1, engine
0.50.1). It reports 323/360 rows backed, 31 unbacked and 0 contradicted. FR-038-AC-176 is not
among the unbacked rows. `quire trace --id FR-038-AC-176` lists three verifying claims:

- `tc_048_qspec_model_declaration_keys_use_production_derivation` (model_members/tests.rs:18)
- `tc_048_qspec_all_positive_packages_and_owner_identities` (main.rs:485)
- `tc_048_qspec_positive_inputs_fail_closed` (main.rs:545)

## Verdict

**No blocking gap.** Each clause of FR-038-AC-176 has an owning test:

| AC-176 clause | Covered by |
| --- | --- |
| All nine published positive packages are required by name and admit with their `package_id` | `POSITIVE_ALL` and the all-positive test |
| The selected `acme/orders` document is supplied under its selection digest and checked by the reader | `read_package_with_document` and the reader's `admit_document` |
| Node id and owner read back through the graph accessor and agree with the published graph and identity projection, with distinct two-owner ids | The all-positive test |
| Production `declaration_key` oracle over `model_declaration_nodes` | The private model test |
| Fail-closed inputs | The fail-closed test |

No production code was added, and the PR copies no fixture or digest catalog. The status flip
of the FR-038-AC-176 row and the TC-048 note covers only what now runs, and IR-630
re-derivation stays unclaimed.

Two weaknesses belong to SR-2700 and are not repeated here:

- The key oracle can stop running silently (SR-2700 FND-001). The static coverage credit for
  AC-176 does not depend on the oracle running.
- Three fail-closed assertions do not pin the affected name (SR-2700 FND-003).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-228 Status still says the AC-176 external conformance row is "PLANNED / UNRUN" and "absent from the executable conformance target until IR-654 code lands". This PR adds that row, so after merge TC-228 contradicts FR-038-AC-176 (now IMPLEMENTED) and the harness | spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md:77-85 |

### FND-001 detail

The PR updates the FR-038-AC-176 row and appends an implemented note to TC-048. It leaves
TC-228's Status section unchanged, and that section names the same two-owner fixtures and
`model-member-type-vectors.json` as pending coverage. Strict coverage does not read status
prose, so no gate catches it. Flip the TC-228 paragraph to name
`tests/conformance_qspec/main.rs` and the private model test, and keep the IR-630 sentence.
The FR-038 history note at line 2497 ("now plans to read") is historical narrative and can
stay.
