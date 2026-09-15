---
id: SR-036
title: "Integrity review of bounded Kani K0"
type: SpecReview
analysis: integrity
scope: "FR-029, FR-030, FR-031, AD-002, AP-002, MP-002, TC-042"
review_set: all
---
# SR-036: Integrity review of bounded Kani K0

## Summary

The three requirements divide profile selection, finite input/outcomes, and dispatch/replay/provenance without contradicting the existing native source authority. Each acceptance criterion maps to TC-042.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- |
| FND-361 | low | No issue found: profile selection precedes input validation, which precedes dispatch and replay; the dependency graph is acyclic and the matrix identifies the planned verification population. | FR-029, FR-030, FR-031, TC-042 |
