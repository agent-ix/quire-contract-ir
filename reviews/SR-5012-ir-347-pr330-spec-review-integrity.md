---
id: SR-5012
title: "Integrity review of FR-019 public items table"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@d61f55e1db42a3ea0a55484d3d4873feb2ac60ea; spec/model/functional/FR-019-rust-library-interface.md:161-163; FR-019-AC-5; FR-038; AD-001; source export chain"
review_set: subset
---

## Summary

Reviewed the frozen PR #330 table diff and the default-feature public export chain. All 16 added V2 types are public model-root exports owned by FR-038; none are fault-injection-only. Ticket: IR-347.

## Scope

- Examined FR-019 Public items V2 reader addition and scalar/composite operand rows (lines 161–163).
- Read FR-019-AC-5, FR-038, AD-001, and the checked-package V2 re-export chain as context.

## Verdict

**PASS** — The changed table has no review finding. FR-019-AC-5 and TC-058 remain planned in the existing model matrix; the unchanged root bridge glob is the later implementation slice.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
