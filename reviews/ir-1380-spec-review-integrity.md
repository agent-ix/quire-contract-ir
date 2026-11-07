---
id: SR-2026110602354643
title: IR-1380 PR 940 spec-review/integrity review
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md; crates/quire-contract-model/src/checked_package/v2/mod.rs; tests/it/checked_package_v2_model_root.rs
review_set: subset
---

## Summary

Ticket: IR-1380. Original review covered PR #940 head 88eb1e140 and its three changed spec artifacts. The scope update below records the final PR head. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- FR-038-AC-494 (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): An admitted `CheckedPackageV2` exposes its selected model package identities as a borrowed sequence in ascending identity order through `selected_model_packages`. A package with no selected model returns an empty sequence; a refused or incomplete read exposes no package. The read neither invents source aliases nor accepts caller-authored model selections.
- FR-350 `models` axis (examined, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): each owner-admitted selected model package identity in ascending identity order, as returned by `CheckedPackageV2::selected_model_packages`; no alias member

## Scope update (disposition pass 2)

At PR head cc239d097, FR-350-AC-10 and the added TC-461 step/expected result were removed. The current PR retains the owner API and FR-350 `models` axis correction; executable bridge evidence remains future IR-1005 work. The original findings below and their first dispositions are historical records of the earlier reviewed head, not claims that AC-10 remains in the current specification. No new finding arises from this confined spec change.

## Scope update (disposition pass 3)

At PR head 024529dae, the FR-038-AC-494 table declaration is again parsed as a criterion and the owner test trace remains FR-038-AC-494 only. The coder reports targeted Quire matrix, validation and diff checks passed. The patch does not alter the test body or public API. No new finding arises from this confined correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The Dependencies paragraph still limits the bridge-owned cross-repository contract to AC-1 through AC-9 and omits new AC-10, conflicting with its TC-461 bridge obligation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md:331-340 |

## Verdict

**PASS after dispositions** — The historical finding above was resolved or superseded by the final scope. See Dispositions and Scope update.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-350 now names AC-10 in the bridge-owned cross-repository contract range. |
