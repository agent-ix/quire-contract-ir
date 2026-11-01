---
id: SR-202611010112
title: "IR-1240 PR 792 FR-048 failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; IR-1240 PR #792; spec/model/functional/FR-048-derive-structural-and-group-candidates.md"
review_set: subset
---

## Summary

Checked the callback boundary, identity meaning, and nested-path topology in the amended FR-048 obligations. The spec distinguishes whole-ID substitution from path-segment inference and keeps invalid replacement admission with the reader.

## Verdict

**PASS** — the changed obligation has a defined None/Some outcome, one visit per logical slot, no cascade, and an iterative deep-term bound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-048 Behavior, examined: a nested `node` path segment remains part of one whole ID; the mapper does not synthesize or validate a replacement.
- FR-048-AC-6, examined: None preserves, Some substitutes once, and a replacement is not remapped.
- FR-048-AC-7, examined: raw/typed mirrors share one callback result and unrelated text remains unchanged.
- FR-048-AC-8, context_only: deep traversal is iterative and malformed output is refused by the reader at its normal typed locus.

## Evidence

`CheckedNodePathSegment::Node` is a structural path member (`shared.rs:601`); the amended criterion treats it as a component of the enclosing `CheckedNodeId`. No applicable AssuranceProfile was found under `spec/`.
