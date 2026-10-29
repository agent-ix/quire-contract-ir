---
id: SR-8440
title: "IR-766 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Checked the Rust closure traversal, selected-reference extraction, classification, exports, and focused tests. Three extra trace tags on the export-inventory test claim closure behaviors the test never exercises.

## Examined units

- FR-035-AC-6 — For an admitted lowered root, the typed view returns that exact root and each local node reached through `semantic_type`, declared `dependencies` or validated body references once in ascending typed-key order; independently walking those three edge sources yields the same key set, including a shared or cyclic reference, and every returned node retains its exact source correspondence. (examined).
- FR-035-AC-7 — A selected dependency reference in the view retains its selected dependency identity and target key and cannot be satisfied by an identically keyed local node or an absent, stale or wrong selected package; every lowered-root view exposes both bound and claim typed-key subsets, possibly empty, exactly matching Contract IR's existing lowering classification in typed-key order. (examined).
- FR-035-AC-8 — Querying a non-`lowered` requested key returns its exact typed disposition, while an unrequested or absent key returns typed absence even if its node occurs only as another root's reached meaning; neither returns a fabricated closure, and querying the view changes none of the six FR-330 wire properties or identity. (examined).
- FR-019-AC-5 — The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists, checked by a public-item inventory that fails on an added or missing item; no item in the table is reachable through a `quire_contract_ir` path. (examined).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Export-inventory test is falsely traced to closure traversal and source correspondence | tests/it/public_interface.rs:119 |
| FND-002 | medium | Export-inventory test is falsely traced to selected references and bound/claim subsets | tests/it/public_interface.rs:119 |
| FND-003 | medium | Export-inventory test is falsely traced to dispositions and wire immutability | tests/it/public_interface.rs:119 |

## Verdict

**CONDITIONAL.** The changed Rust path follows the admitted body grammar and preserves typed source correspondence. Focused acceptance tests cover all three edge kinds, selected packages, recursion, disposition, and unchanged wire identity.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |
| FND-003 | fixed | Fixed in the reviewed PR. |
