---
id: SR-610003
title: Code review — IR-1033 owner model carrier
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{application_identity,encode,intake,mod,owner,protocol_body,structural}.rs;
  tests/{it,conformance_qspec}
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-064
  type: references
---

## Summary

IR-1033 PR #592; examined FR-064-AC-1..6 and FR-038 owner/catalog context.

## Verdict

**FAIL** — two actionable code findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Graph-stage clause dependency check only verifies sorted order and owner presence; an extra or omitted body dependency survives until the protocol stage, allowing a paired stale-key defect to win despite FR-064 requiring exact dependency refusal before keys. | crates/quire-contract-model/src/checked_package/v2/structural.rs:449-459 |
| FND-002 | high | Hosted Rust Checks fail make lint: qsl-string-edge reports new branch-gating occurrences in owner.rs:85-86, application_identity.rs:321-322 and mod.rs:1123; the PR cannot pass its required lint gate. | crates/quire-contract-model/src/checked_package/v2/owner.rs:85-86 |

## Coverage

Reviewed diff only; formal Rust style and wire boundary checks included. Hosted lint failure supplied by code lead at exact head; no aggregate gate rerun. QSpec TC-048 pinned fails on STD-498 fixture owner mismatch, which is an external fixture blocker. FR-064-AC-4 source producer half belongs QSL. FR-064-AC-5 remains Planned for PR #586.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |: Owner/extra dependency precedence is fixed. Omitted body references retain their FR-038-AC-285/324 located refusal. Pinned qsl-string-edge exits 0; hosted checks are pending. |
| FND-002 | fixed |: Owner/extra dependency precedence is fixed. Omitted body references retain their FR-038-AC-285/324 located refusal. Pinned qsl-string-edge exits 0; hosted checks are pending. |

## Current verdict

Owner/extra dependency precedence is fixed. Omitted body references retain their FR-038-AC-285/324 located refusal. Pinned qsl-string-edge exits 0; hosted checks are pending.
