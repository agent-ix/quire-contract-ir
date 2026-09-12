---
id: SR-058
title: "Scope-boundary review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: scope-boundary
scope: "FR-026, issues 52/57/63/64, native authorities, and TL authorities"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: reviews
---
# SR-058: Scope-boundary review of issue 64

## Summary

The scope review assigns deterministic projection and result joining to
Contract IR while leaving authored syntax, semantic evaluation, observations,
progress, evidence, and release authority with their owners. Snapshot
`558c4dc` contains no alternate authored language or foreign runtime path.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6461 | high | **Fixed:** the earlier roadmap could be read as permitting authored TL or FRETish input. FR-026 makes native Quire the sole editable formal-clause source; TL is internal and issue #57 is output-only. | FR-026 Description and Dependencies; ADR-0053 | wrong-requirement |
| FND-6462 | high | **Fixed:** consuming TL trace/request/result contracts could imply evaluator ownership. Contract IR constructs derived artifacts and joins supplied results but does not parse TL, evaluate a predicate/formula, read runtime state, invoke plugins, or access a network. | FR-026 Outputs; purity boundary | missing-requirement |
| FND-6463 | medium | **Fixed:** progress, completeness, clock, capture, and correction semantics were partially shared. Their authorities now publish and authenticate assertions; FR-026 only verifies exact bindings and correspondence. | FR-026 Inputs, Public v1 records, Dependencies | wrong-requirement |

## Responsibility Allocation

| Responsibility | Owner |
|---|---|
| Authored temporal clause and native meaning | Quire specification/frontend |
| Predicate projection and total Boolean values | Contract IR FR-025 |
| Observation, clock, capture, progress, completeness, corrections | Native owning authorities and public readers |
| TL formula/semantic/trace/request/evaluator/result contracts | tl-syntax and tl-mltl |
| Exact projection and cross-result join | Contract IR FR-026 |
| FRETish rendering | issue #57 output-only consumer |
| Evidence retention and release decisions | Quoin/Engineering Assurance and release owners |

Any first-party production or qualification logic added for FR-026 must be
Rust. No Java, Node, Electron, or new Python semantic path is admitted.

## Result

**PASS after remediation.** Responsibilities are allocated without creating a
second source language or duplicating semantic and evidence authorities.
