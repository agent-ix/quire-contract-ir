---
id: SR-2026102991
title: "IR-1164 PR #722 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; FR-038 AC136/142/370-373, TC-227, matrix/tests.md"
review_set: subset
---

## Summary

TC-227 expected results contradict the new typed accessor error controls.

## Verdict

CONDITIONAL — TC-227 expected-result text must include the new typed errors.

## Examined units

FR-038-AC-136, AC-142, AC-370 through AC-373 and TC-227 were examined; QSpec FR-322-AC-30 was context. The matrix row was checked for Planned status.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-227 still says every accessor error is a variant AC-139 names, excluding the new MissingDeclaringNode and AmbiguousDeclaringNode errors | spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md:67 |
