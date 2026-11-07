---
id: SR-2026110602354650
title: "IR-1369 PR #942 spec-review-failure-domain"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; IR-1369; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md; spec/checked_package/matrix/TC-467-native-selected-identity-join-budget.md; crates/quire-contract-model/src/checked_package/v2/{native_join_budget,exact_projection}.rs; tests/it/exact_projection_limits.rs"
review_set: subset
---

## Summary

Ticket: IR-1369. Inspected zero and unequal limits, shared cumulative exhaustion, full and compact native refusals, insufficient semantic-diagnostic storage, secondary storage denial after an original work denial, and default scalar WorkMeter behavior. Typed owner metadata and original requested identities are retained where affordable; compact refusal adds no owned allocation. No new extension seam, graph traversal, panic path, shared state or mutable authority is introduced.

## Examined units

- FR-358-AC-8 (examined, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md): "The public work and storage getters return the caller's original, distinct configured limits before acquisition, after successful stage and type projection operations, and after a later shared-budget denial; neither getter reports the remaining allowance. Each full or compact `ExactTypeProjectionError` resource refusal carries the original bound of its denied resource, including zero and the first work denial when diagnostic storage is also denied. A non-resource refusal has no configured bound."
- FR-358 (examined, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md): "The budget SHALL expose its original configured work and storage limits through\n`work_limit` and `storage_limit`; successful charges, denied charges and later\noperations SHALL NOT change these values. A native resource refusal carried by\n`ExactTypeProjectionError` SHALL expose `configured_bound` as the original\nlimit for its denied resource, including zero. This bound SHALL remain present\nwhen diagnostic storage is unavailable and the refusal has no owned locus.\nIf allocating a work refusal's diagnostic also exceeds storage, the compact\nrefusal SHALL retain the original work denial a"
- TC-467 (examined, spec/checked_package/matrix/TC-467-native-selected-identity-join-budget.md): "The limit getters remain equal to the caller's original configured values, independently of consumption or denial. Full and compact native resource refusals retain the bound for the original denied kind; later diagnostic storage exhaustion cannot substitute its storage bound for a work bound. Zero is a present bound, while a non-resource refusal has no configured bound."
- FR-019 (context_only, spec/model/functional/FR-019-rust-library-interface.md): "The stable surface is the public API of quire_contract_model in a default-feature build: exactly the items the Public items table below lists."

## Verdict

**PASS** — No findings in this scoped method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence and limits

Reviewed source head 1111b0400c4040b2769c1197ec5f6bb454d51255 in the ticket's existing single worktree. Focused evidence: /tmp/ir1369-focused.log and /tmp/ir1369-focused-final.log. Final log records one owned/compact test and five exact_projection_limits tests passing, with strict workspace Clippy complete. No reviewer Cargo build or aggregate gate was run. The lead's one final combined-tree gate remains pending; cargo-deny and release-wide qualification are not claimed by this review. No applicable AssuranceProfile was found. Plan completion: not assessed.
