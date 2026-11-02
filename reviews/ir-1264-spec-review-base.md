---
id: SR-202611010118
title: "IR-1264 PR 817 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1264 PR #817; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md"
review_set: subset
---

## Summary

Reviewed FR-038 owner presence, AC-417, and the FR-347 scope correction against QSpec FR-322 and TC-281. No applicable AssuranceProfile was found under `spec/`; only architecture descriptions are present.

## Verdict

FR-038 owner prose is incomplete against the closed QSpec table.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038 says every unlisted node omits owner, excluding QSpec-required model-owned forms | FR-038:2607-2612; QSpec FR-322:1568 |

## Coverage

- FR-038 owner-presence paragraph, role: examined.
- FR-038-AC-420, role: examined.
- FR-347 Scope paragraph, role: examined.
- QSpec FR-322 owner table and AC-68, role: context_only.
- QSpec TC-281 EV-01 and EV-02, role: context_only.

## Evidence

The two changed requirement documents pass scoped Quire validation and `git diff --check`. AC-420 is planned until IR-1264 lands and has no code trace tag yet. The diff adds no hash, digest, pin, or tracking record.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038 now applies the full QSpec OwnerPresenceRule table rather than a partial list, and states the event-type split explicitly. |
