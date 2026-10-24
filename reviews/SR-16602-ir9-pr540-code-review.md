---
id: SR-16602
title: "Code and Rust review — typed retained-term conversion causes"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #540 abd13e9017b56447f93a272fa887ceb29847ac03; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs, mod.rs, contract_package.rs; FR-349-AC-4, FR-038, TC-048"
review_set: subset
---

## Summary

Reviewed the exact PR diff, the flat body validator, the recursive decoder, both reader mappings and existing public refusal tests. The new private cause enum preserves missing, unknown and malformed-member distinctions in conversion without changing the published refusal mapping or abstraction-relation stage order.

## Verdict

**PASS** — no code or Rust finding in the changed scope. The cause is retained inside `decode_admitted`; this PR does not claim a new public refusal or a clean IR-9 AC-8 scan.

## Scope examined

| Unit | Role | Evidence |
| --- | --- | --- |
| `FR-349-AC-4` | examined | "A malformed or unknown term still refuses at the same first code, cause and JSON pointer as the existing reader, before an admitted typed package is returned." |
| `FR-038` | context_only | V2 reader ordering and typed refusal envelope. |
| `TC-048` | examined | New cause-order unit controls plus existing public abstraction, frame and extension refusal controls. |

The tests introduced in `bb4992c5` precede implementation `abd13e9`; their expected `TermConversionCause` variants are absent at the base. The helper checks fields in the original order, recursively propagates the first child's cause, and distinguishes an absent tag from an unknown tag and a malformed tag value. `validate_graph` and `ContractPackageWireV1` deliberately map every conversion error to their existing refusal; the abstraction relation keeps its deferred `None` path and later retry. No new public API, unsafe block, dependency, lock change, gate change or vendored artifact occurs in this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
