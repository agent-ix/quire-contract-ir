---
id: SR-15721
title: "Code and Rust review — IR-975 single-body V2 wire"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #521 at f08265affe2165e852c2febd85659ef68da2e70d; checked_package/v2 production and conformance/test diff"
review_set: subset
---

## Summary

The reader uses the seven selection fields from the wire and derives the package hash input from the complete graph nodes without occurrences. The canonical writer emits the selection-only envelope, and the tests exercise published QSpec positives and both retired projection cases. A changed conformance module still claims a removed owner mutation control.

## Verdict

**PASS at 8521036b21b13dd3057a34cd83e3fa54ff73a85b** — the stale coverage description is corrected; no code or test behavior changed in the fix rounds.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Conformance module header still claims AC-154 projection-owner adverse entries, which this PR removed; describe the current graph-owner and shipped-projection controls. | tests/conformance_qspec/main.rs:11 |

## Rust review

The diff adds no unsafe code, panic or unchecked conversion in the production path. `decode_borrowed_wire` maps unknown immediate selection members to `MalformedWire` while preserving nested refusal classification. The node header check now precedes package identity as QSpec requires. `CheckedPackageIdentityPreimageRefV2` writes each graph node's identity fields and full body, omitting only occurrences. No copy of QSpec fixtures or old-wire admission path was added. The author-reported 37 passing conformance controls and final gate evidence were inspected as claims, not independently rerun during this review.

## Scope

Examined FR-038-AC-4, AC-107, AC-112, AC-123, AC-133, AC-154 and AC-310; FR-051-AC-4; FR-349-AC-3. Examined the PR's production Rust, test and conformance diff against QSpec FR-322 at published revision 4c687282. The stale AC-123/AC-133 normative text is recorded in the spec-review artifacts.

## Custody correction

This artifact supersedes the same reviewer method under SR-15714 on IR-975. SR-15714 had already been assigned to IR #522; the original finding text above is unchanged; later dispositions or head confirmations record the current verdict.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At 5e2eaa1, the module header names AC-310 and describes exact malformed-wire pointers for shipped projections and extra selection members. The obsolete AC-154 projection-owner claim is gone. |
