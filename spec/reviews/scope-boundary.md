---
id: SR-040
title: "Scope-boundary review of bounded Kani K0"
type: SpecReview
analysis: scope-boundary
scope: "FR-029, FR-030, FR-031, AD-002, AP-002, MP-002, TC-042"
review_set: all
---
# SR-040: Scope-boundary review of bounded Kani K0

## Summary

Contract IR owns bounded-profile selection, ABI validation, dispatch, artifacts, typed outcomes, and replay agreement. Quire owns native meaning and checked clauses; Kani is an externally selected executable; native `runtime::execute` is the independent replay oracle.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- |
| FND-401 | low | No issue found: the design does not grant Contract IR source-language authority, does not treat Kani as an unbounded semantic authority, and does not make a replay result a release decision. | AD-002, FR-029, FR-031 |
