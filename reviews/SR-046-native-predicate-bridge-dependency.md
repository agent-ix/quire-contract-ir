---
id: SR-046
title: "Dependency review of native predicate to TL projection"
type: SpecReview
analysis: dependency
scope: "FR-025 at cf4beaf15e35dfe276749637dbfb16c230070514; FR-012/014/015/016; native FR-019/048/095 candidate; tl-syntax FR-007; issues #52/#64/#57"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-046: Dependency review of native predicate to TL projection

## Summary

The reviewed dependency graph separates current specification from future
admission and names every upstream authority and target artifact required
before implementation can start.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6306 | high | **FIXED:** Moving native and TL branch/profile names were treated as sufficient. Every selection now binds contract, package version, repository, immutable commit and schema digest; candidates remain unaccepted. | FR-025 ContractSelection; Dependencies | wrong-requirement |
| FND-6307 | high | **FIXED:** Result availability and producer-result projection authorities were absent. Exact assertion bytes and accepted availability, source-result and mapping contracts are now blocking prerequisites. | FR-025 Inputs; Dependencies | wrong-requirement |
| FND-6308 | medium | **FIXED:** FR-016 canonical rules lacked a dependency edge. The normative edge and dependency entry are present. | FR-025 relationships; Dependencies | wrong-requirement |
| FND-6326 | high | **FIXED:** Requiring one whole FR-023 `BoundClause` stranded inline native `holds(expr)` leaves. FR-025 now consumes an authority-verified checked leaf under its parent subject and requires the native FR-048/FR-095 contract to publish its reader. | FR-025 Inputs; Behavior; Dependencies | wrong-requirement |
| FND-6327 | medium | **FIXED:** Matrix and suite readiness omitted source-result/mapping and availability blockers. All readiness surfaces now name the complete prerequisite set. | contract-test-matrix; SUR-001 | wrong-requirement |
| FND-6328 | medium | **FIXED AS GATE:** No published artifact grounded signal-catalog `schema_digest`. Target admission is now blocked until tl-syntax publishes canonical schema bytes; Rust/Serde shape alone is insufficient. | FR-025 Dependencies; TC-038 | wrong-requirement |

## Dependency Order

Issue #63 supplies the predicate projection contract before issue #64 binds a
generated temporal formula. Issue #57 remains output-only. No target artifact
or external runtime becomes a source authority. **PASS for specification; the
implementation gate remains closed.**
