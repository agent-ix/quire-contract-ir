---
id: SR-1177
title: "AC-shape review of PR 279 (FR-038-AC-119, AC-104)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@48f27d676fad56fa5f7e7d69fefb39c254e21880; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-119, the amended FR-038-AC-104 and the new timed-form prose"
review_set: subset
---
# SR-1177: AC-shape review of PR 279

## Summary

Ticket: IR-551. FR-038's ACs are direct assertions, not EARS `shall` sentences.
The change follows that convention:

- No added line uses `shall`.
- AC-119 carries the 🚧 planned marker, and the prose carries "🚧 Planned
  (IR-551 code)".
- No AC row names a tracker state.
- Every refusal names its code, its cause and its pointer.
- `quire validate` reports the grammar baseline unchanged: 376/377 docs clean,
  with the one finding in FR-014 line 137, which is already on main. AC-119
  trips no grammar rule.

AC-119 is compound, which is an atomicity finding. It is recorded once, as
SR-1176 FND-006, and is not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

The form is clean. The AC shape matches FR-038's convention.

## Disposition pass 1

Re-checked at 9d1879016a15023a9f55fa8185950066e705b4b7. AC-119 to AC-122 keep
FR-038's direct-assertion form:

- no `shall`;
- the 🚧 planned marker on each row;
- no tracker state;
- a code, cause and pointer for each refusal.

The grammar baseline is unchanged: 376/377, FR-014 only. This file remains clean.
