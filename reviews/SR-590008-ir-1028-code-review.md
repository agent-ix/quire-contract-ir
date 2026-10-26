---
id: SR-590008
title: "Code review — IR-1028 indexed application group key lookup"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/application_identity.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---

## Summary

Ticket: IR-1028. Reviewed the PR #585 Rust diff at frozen head 9c10bf965bc1360597f2c9452c8ee6a3d01a30d2, including the 100,000-node cyclic protocol test and affected key derivation call path.

## Verdict

**PASS** — Indexed group lookup preserves graph-order ordinals and key validation; no code or Rust-style defect was found in the changed scope.

## Scope examined

- FR-038-AC-88: group-local preimage, reference rewriting, stale-key refusal, and admission.
- `application_identity.rs`: indexed ordinal lookup, group size, reference rewrite dispatch.
- `operations.rs`: group index construction, digest re-derivation, and the 100,000-member test.
- Reader duplicate-key check and existing key/refusal tests were read as context. No CI, manifest, or spec file changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence and limits

The new test asserts the last member's `{size, ordinal}`, wraparound `group_reference`, and successful re-derivation of every application key. Existing tests cover a fixed small-group preimage, nested rewrites, stale-key refusal, and exact byte/work limits. The computed Quire matrix reports FR-038-AC-88 as tagged, including the new test. The author reports focused tests passed; this reviewer ran no Cargo or rustc command under the host build cap. The dispatching lead owns the final aggregate gate.
