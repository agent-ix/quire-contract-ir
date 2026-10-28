---
id: SR-611012
title: "code-review review of IR-874 PR #617"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-283; TC-048"
review_set: subset
---

## Summary

Code review and Rust test review of PR #617 at 66cca9fc; the changed tests use the public reader and have deterministic assertions, but leave two AC-283 identity cases unverified.

## Verdict

**CONDITIONAL** — Fix the recorded coverage gaps before merging. Focused formatter, 136 protocol tests, and Clippy were reported by the coder; this review did not repeat build gates.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No set-backed source declaration reorder demonstrates stable identities | tests/it/checked_package_v2_protocol_body.rs:4812 |
| FND-002 | medium | No changed protocol reference target demonstrates changed identities | tests/it/checked_package_v2_protocol_body.rs:7538 |

## Coverage

- FR-038-AC-283 (examined): A protocol node derives its `quire.application-node/v1` key from its exact tag, form, Boolean type, declaration path, `null` recursion and body, with occurrences excluded; a second reference, source reformatting or declaring roles and other set-backed members in another source order preserves the key and package id, while a changed name, referenced target, literal, ordered capture or `sequence` child changes both. Role, channel, finish, terminal, control and compensation declarations use the fixed qualified-name segments of QSpec FR-464; queues have no declaration and generated occurrences, while other protocol members have declaration occurrences. A wrong path or occurrence role refuses at the node.
- TC-048 (examined): Reorder an authored `sequence` and an ordered capture list and require changed keys; reformat source and reorder role or other set-backed declarations and require unchanged keys.
- Changed file: tests/it/checked_package_v2_protocol_body.rs; no production or CI workflow files changed.

### FND-001

The new source-occurrence test adds a second occurrence and edits its region. None of the 12 criterion-tagged tests reorders role or other set-backed source declarations and checks that the node key and package ID remain unchanged. A reader or fixture that treats set order as meaningful would pass this suite.

### FND-002

The role-target test changes a target and checks only the final semantic refusal. It does not retain the original node key and package ID or compare either after rekeying; no other criterion-tagged test does. A key derivation that omits reference targets could pass this suite.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Source-map region swap reverses source order for two admitted roles; the public reader admits and node keys, clause key and package ID remain equal. |
| FND-002 | fixed | A protocol-reference carrier retargets from an admitted clause to an admitted role; both node key and package ID change and the public reader admits both. |

## Disposition verdict

**PASS at f17fa5851fa7c6c7decf7306e3c6d0f370e9398c** — the two recorded test-coverage gaps are fixed. The fix diff changes tests only; the lead owns the aggregate gate.
