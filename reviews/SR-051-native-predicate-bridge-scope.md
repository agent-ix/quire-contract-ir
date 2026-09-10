---
id: SR-051
title: "Scope-boundary review of native predicate to TL projection"
type: SpecReview
analysis: scope-boundary
scope: "issue #63 ownership against native frontend, Contract IR, tl-syntax, issue #64 and output-only issue #57 at cf4beaf15e35dfe276749637dbfb16c230070514"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-051: Scope-boundary review of native predicate to TL projection

## Summary

Native Quire retains authored language and producer authority; Contract IR owns
the typed bridge; TL receives only generated internal artifacts.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6323 | high | **FIXED:** Treating a selected set as formula-complete moved temporal parsing into Contract IR. Issue #63 maps explicit leaves; issue #64 owns generated-formula occurrence binding. | FR-025 Behavior; issue #64 | wrong-requirement |
| FND-6324 | high | **FIXED:** A local result profile could re-parent native result semantics. Contract IR emits a derived mapping receipt while source and availability authorities retain identity and bytes. | FR-025 Public v1 records | wrong-requirement |
| FND-6325 | high | **FIXED:** TL/FRETish artifacts could appear to be editable semantic authority. Native Quire is sole source, TL is internal derived input and FRETish is output-only. | FR-025 Description; Dependencies | wrong-requirement |

## Boundary Result

The bridge does not parse source, resolve names, evaluate predicates or TL,
read ambient state, invoke plugins, access the network, retain evidence, or
make a release decision. **PASS.**
