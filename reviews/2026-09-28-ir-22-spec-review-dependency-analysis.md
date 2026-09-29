---
id: SR-588
title: "PR #202 dependency review: relationship edges and the qsl-replay dependency"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir@5c4a5059d0b8965687195b23534cbd90e4791442; spec/ (29 files, git diff origin/main...HEAD)"
review_set: subset
---
# SR-588: PR #202 dependency review: relationship edges and the qsl-replay dependency

## Summary

Dependency analysis of the new `relationships:` edges and the dependency claims.

Ticket: IR-22.

FR-029 to FR-038 and FR-344 each gain `traces_to StR-001`. The StR-001 matrix row now spans FR-025 to FR-040 and FR-344, which is consistent. FR-039 and FR-040 each `depends_on` only same-repo requirements. Cross-repo edges are `references` only, so no cross-repo blocking edge is created. `qsl-replay` exists at the QSL revision the root crate already pins (`9395be4`) with every type the spec names, so AD-001's claim that this adds no repository edge holds. The dependency split between enablement and feature work is sound: FR-039 enables the code PR that deletes the copies, and FR-040 extends FR-038's reader without reopening it.

Scope examined (all 29 changed files):

- `spec/assurance/AD-001-contract-ir-architecture.md`
- `spec/assurance/AD-002-bounded-kani-architecture.md`
- `spec/assurance/AD-003-complete-v1-backend-delivery.md`
- `spec/assurance/AP-002-bounded-kani.md`
- `spec/contract-test-matrix.md`
- `spec/contract/FR-029-versioned-bounded-kani-profile.md`
- `spec/contract/FR-030-bounded-kani-domain-and-outcomes.md`
- `spec/contract/FR-031-bounded-kani-dispatch-replay-provenance.md`
- `spec/contract/FR-032-admit-output-mapping-request.md`
- `spec/contract/FR-033-account-for-output-obligations.md`
- `spec/contract/FR-034-assemble-output-package-atomically.md`
- `spec/contract/FR-035-complete-v1-contract-package-lowering.md`
- `spec/contract/FR-036-exact-backend-negotiation-and-emission.md`
- `spec/contract/FR-037-canonical-backend-replay-and-qualification.md`
- `spec/contract/FR-038-consume-checked-package-v2.md`
- `spec/contract/FR-040-admit-frame-entries-and-state-clauses.md`
- `spec/contract/FR-344-admit-or-refuse-the-adr-002-2-0-0-members.md`
- `spec/contract/TC-046-canonical-backend-replay.md`
- `spec/contract/TC-048-checked-package-v2-strict-reader.md`
- `spec/contract/TC-050-checked-package-v2-lowering.md`
- `spec/contract/TC-052-checked-package-v2-lowering-vocabulary.md`
- `spec/contract/TC-054-kani-counterexample-replay-crossing.md`
- `spec/contract/TC-055-root-crate-public-interface.md`
- `spec/contract/TC-056-checked-package-v2-frame-entries-and-state-clauses.md`
- `spec/contract/TC-057-qspec-node-identity-vectors.md`
- `spec/contract/TC-223-kani-outcome-fr331-result-map.md`
- `spec/index.md`
- `spec/interface/FR-039-root-crate-public-interface.md`
- `spec/reviews/scope-boundary.md`

## Verdict

**PASS**. No findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
