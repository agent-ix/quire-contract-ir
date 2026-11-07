---
id: SR-2026110602354641
title: IR-1380 PR 940 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md; crates/quire-contract-model/src/checked_package/v2/mod.rs; tests/it/checked_package_v2_model_root.rs
review_set: subset
---

## Summary

Ticket: IR-1380. Original review covered PR #940 head 88eb1e140 and its three changed spec artifacts. The scope update below records the final PR head. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- FR-038-AC-494 (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): An admitted `CheckedPackageV2` exposes its selected model package identities as a borrowed sequence in ascending identity order through `selected_model_packages`. A package with no selected model returns an empty sequence; a refused or incomplete read exposes no package. The read neither invents source aliases nor accepts caller-authored model selections.
- FR-350 `models` axis (examined, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): each owner-admitted selected model package identity in ascending identity order, as returned by `CheckedPackageV2::selected_model_packages`; no alias member

Plan completion: not assessed.

- FR-038-AC-493 (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): Inspection of `AdmissionBinding` and the V2 reader finds that the public capability uses the reader's original admission buffers; its fields are private, with no public constructor, buffer accessor, copied carrier or compatibility representation.

## Scope update (disposition pass 2)

At PR head cc239d097, FR-350-AC-10 and the added TC-461 step/expected result were removed. The current PR retains the owner API and FR-350 `models` axis correction; executable bridge evidence remains future IR-1005 work. The original findings below and their first dispositions are historical records of the earlier reviewed head, not claims that AC-10 remains in the current specification. No new finding arises from this confined spec change.

## Scope update (disposition pass 3)

At PR head 024529dae, the FR-038-AC-494 table declaration is again parsed as a criterion and the owner test trace remains FR-038-AC-494 only. The coder reports targeted Quire matrix, validation and diff checks passed. The patch does not alter the test body or public API. No new finding arises from this confined correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new owner-only test tags FR-350-AC-10 but never constructs or strictly reads a temporal correspondence record, so the tag falsely marks bridge behavior as covered. | tests/it/checked_package_v2_model_root.rs:25-55 |

## Verdict

**PASS after dispositions** — Both historical findings have fixed outcomes; INSP-001 supplies the AC-493 inspection evidence when committed in this PR.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The owner-only test now tags only FR-038-AC-494. FR-350-AC-10 remains assigned to bridge TC-461 in IR-1005. |
| FND-002 | fixed | Fixed by the independent Pass inspection act INSP-001 for FR-038-AC-493 at 58386de291c8291a1b313e591aa06b431b728052, authored for spec/evidence/inspections.md; the coder must commit that exact record. |

## New findings (disposition pass 4)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | AC-493 now declares Inspection, which strict matrix classifies as method-without-symbol rather than tagged; no Inspections evidence act records the inspector, commit, and verdict for AC-493. The IR-1370 SpecReview describes a review but does not discharge the inspection registry obligation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6007 |

## Scope update (disposition pass 5)

I independently inspected the public API at 58386de291c8291a1b313e591aa06b431b728052. The authored inspection record is `/tmp/ir1380-inspection-58386de29/spec/evidence/inspections.md`, intended for `spec/evidence/inspections.md`; its Pass row names FR-038-AC-493, the inspector and exact inspected commit. No new finding arose from this inspection.
