---
id: SR-2026110602354641
title: IR-1380 PR 940 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md; spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md;
  crates/quire-contract-model/src/checked_package/v2/mod.rs; tests/it/checked_package_v2_model_root.rs
review_set: subset
---

## Summary

Ticket: IR-1380. Reviewed PR #940 frozen head 88eb1e140, scoped to the changed API, test and three spec artifacts. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- FR-038-AC-494 (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): An admitted `CheckedPackageV2` shall expose its selected model package identities as a borrowed sequence in ascending identity order through `selected_model_packages`. A package with no selected model returns an empty sequence; a refused or incomplete read exposes no package. The read neither invents source aliases nor accepts caller-authored model selections.
- FR-350-AC-10 (examined, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Given selected model roots whose graph order differs from ascending package identity order, TC-461 records exactly the package identities returned by the admitted checked package's `selected_model_packages` read in that order, including the empty sequence for a package with no selected model. A renamed, missing, duplicated or reordered model entry fails strict correspondence reading; the bridge does not infer a source alias or accept a caller-authored alias map.
- TC-461 (examined, `spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md`): 8. Select model roots in graph order `z/model`, `a/model`, then read the
   admitted package's `selected_model_packages` and require the correspondence
   `models` array to be `a/model`, `z/model`. Repeat with no selected model and
   require an empty array. Rename, remove, duplicate and reorder an entry in
   the encoded record; strict reading refuses each changed array. Supply no
   source alias map to the bridge (FR-350-AC-10).

Plan completion: not assessed. Scoped AC-to-test check; the integration bridge is owned by IR-1005.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new owner-only test tags FR-350-AC-10 but never constructs or strictly reads a temporal correspondence record, so the tag falsely marks bridge behavior as covered. | tests/it/checked_package_v2_model_root.rs:25-55 |

## Verdict

**CONDITIONAL** — One actionable finding is recorded above.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The owner-only test now tags only FR-038-AC-494. FR-350-AC-10 remains assigned to bridge TC-461 in IR-1005. |
