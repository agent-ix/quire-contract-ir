---
id: SR-15325
title: "IR-571 scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038, FR-346; spec/checked_package/matrix/TC-225; current TC-225 implementation; QSpec FR-154-AC-14"
review_set: subset
---

## Summary

QSpec owns declaration validation; IR owns the selected-document public refusal and frame-step ordering. The changed files keep those responsibilities explicit.

## Verdict

**PASS** — QSpec owns declaration validation; IR owns the selected-document public refusal and frame-step ordering. The changed files keep those responsibilities explicit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
