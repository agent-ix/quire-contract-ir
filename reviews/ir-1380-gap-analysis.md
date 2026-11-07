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

- FR-038-AC-494 (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): An admitted `CheckedPackageV2` shall expose its selected model package identities as a borrowed sequence in ascending identity order through `selected_model_packages`. A package with no selected model returns an empty sequence; a refused or incomplete read exposes no package. The read neither invents source aliases nor accepts caller-authored model selections.
- FR-350 `models` axis (examined, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): each owner-admitted selected model package identity in ascending identity order, as returned by `CheckedPackageV2::selected_model_packages`; no alias member

Plan completion: not assessed.

## Scope update (disposition pass 2)

At PR head cc239d097, FR-350-AC-10 and the added TC-461 step/expected result were removed. The current PR retains the owner API and FR-350 `models` axis correction; executable bridge evidence remains future IR-1005 work. The original findings below and their first dispositions are historical records of the earlier reviewed head, not claims that AC-10 remains in the current specification. No new finding arises from this confined spec change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new owner-only test tags FR-350-AC-10 but never constructs or strictly reads a temporal correspondence record, so the tag falsely marks bridge behavior as covered. | tests/it/checked_package_v2_model_root.rs:25-55 |

## Verdict

**PASS after dispositions** — The historical finding above was resolved or superseded by the final scope. See Dispositions and Scope update.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The owner-only test now tags only FR-038-AC-494. FR-350-AC-10 remains assigned to bridge TC-461 in IR-1005. |
