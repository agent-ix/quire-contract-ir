---
id: SR-590
title: "PR #202 risk and complexity review"
type: SpecReview
analysis: risk-complexity
scope: "agent-ix/quire-contract-ir@5c4a5059d0b8965687195b23534cbd90e4791442; spec/ (29 files, git diff origin/main...HEAD)"
review_set: subset
---
# SR-590: PR #202 risk and complexity review

## Summary

Risk and volatility analysis.

Ticket: IR-22.

The main volatility is OQ-2 (moving the family lowerings to codegen) and OQ-1 (glob versus explicit re-exports). Each changes FR-039's table, and FR-039 already flags both with a placement note. OQ-3 depends on a QSL change to `TerminalValue`. This is recorded as an open question with options rather than as a gate, so Contract IR work does not wait on QSL. FR-040 restates QSpec text that was accepted recently (the QSpec origin/main head is `e56756f`, STD-111). If QSpec revises FR-340, FR-341 or FR-342, FR-040 drifts. The mitigation is that TC-056 reads QSpec's published fixtures at run time under `make qspec-vectors` rather than copying them.

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
