---
id: SR-037
title: "Dependency review of bounded Kani K0"
type: SpecReview
analysis: dependency
scope: "FR-029, FR-030, FR-031, AD-002, AP-002, MP-002, TC-042"
review_set: all
---
# SR-037: Dependency review of bounded Kani K0

## Summary

The required order is FR-029 profile/index enablement, FR-030 finite ABI/outcome enablement, then FR-031 family dispatch/generation/replay features. The dependency edges are explicit and acyclic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- |
| FND-371 | low | No issue found: the shared profile/index and typed outcome vocabulary are enablement prerequisites; definedness/arithmetic, object/graph, collection/query, and replay integration can be separately tasked only after them. | FR-029, FR-030, FR-031 |
