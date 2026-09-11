---
id: SR-050
title: "Risk and complexity review of native predicate to TL projection"
type: SpecReview
analysis: risk-complexity
scope: "cross-repository canonical wire, 10k populations, strict readers, failure injection and dependency volatility at cf4beaf15e35dfe276749637dbfb16c230070514"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-050: Risk and complexity review of native predicate to TL projection

## Summary

Risk is **high** and volatility is **high**. The principal drivers are multiple
domain-separated canonical identities, a three-artifact public join, 10,000-item
boundaries, result/availability producer authority, and unreleased upstream
contract selections.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6320 | high | **FIXED:** Embedded 100,000-item outputs exceeded Contract IR limits. The bridge now uses 10,000 items, bounded sibling artifacts and a 64 MiB preflight. | FR-025 bounds; AC-6 | wrong-requirement |
| FND-6321 | medium | **FIXED:** Identity and reader failures lacked independent oracles. Snapshot, real-reader and failpoint suites are separate from properties. | SUR-001 SUITE-005..008 | correct-requirement-no-evidence |
| FND-6322 | high | **FIXED:** A convenience parser/evaluator/foreign-runtime adapter could enlarge qualification scope. Inputs are supplied validated values and first-party implementation remains Rust. | FR-025 purity and Dependencies | wrong-requirement |

## Verdict

The residual risk is explicit and implementation remains gated on stable
contract selections. **PASS for specification.**
