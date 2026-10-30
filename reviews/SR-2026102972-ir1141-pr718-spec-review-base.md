---
id: SR-2026102972
title: "spec-review-base review of IR-1141 PR #718"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-038, FR-053, TC-048, TC-453"
review_set: subset
---

## Summary

Reviewed the four-file spec-only PR #718 against QSpec FR-322 and AC-74. The no-max admitted result, exact decimal maximum, reference target kinds, compound unit checks, and stated refusal precedence align with QSpec. The TC-048 reference-target mutation setup has a defect. No applicable AssuranceProfile was found.

## Examined scope

- `FR-038-AC-22` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-038-AC-124` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-038-AC-269` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-038-AC-279` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-053-AC-1` (examined): spec/model/functional/FR-053-selected-population-binding.md
- `FR-053-AC-3` (examined): spec/model/functional/FR-053-selected-population-binding.md
- `FR-053-AC-4` (examined): spec/model/functional/FR-053-selected-population-binding.md
- `TC-048 derived reference target` (examined): spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
- `TC-048 compound unit` (examined): spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
- `TC-048 population` (examined): spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
- `TC-453 selected population` (examined): spec/model/matrix/TC-453-selected-population-binding.md
- `QSpec FR-322 derived shapes` (context_only): agent-ix/quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-048 says to mutate only a reference body target while expecting `invalid-derived-shape`. QSpec FR-322 requires `composite_type`/`reference` dependencies to equal the body targets and checks that list at the earlier graph-shape stage. If the dependency remains on the original target, the package refuses `invalid_semantic_graph` at `/dependencies` before the target-kind check. Update the dependency list with each substituted target and keep all other checks valid. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:624-632 |

## Verdict

FAIL until TC-048 constructs graph-valid semantic target mutations.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The substituted target is now also the reference node’s exact dependency, so graph shape passes before target-kind validation. |

## Disposition verdict

PASS; both original test-setup findings are fixed, with no new finding in the amendment.
