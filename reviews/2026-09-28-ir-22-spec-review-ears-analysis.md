---
id: SR-586
title: "PR #202 EARS and wording review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@5c4a5059d0b8965687195b23534cbd90e4791442; spec/ (29 files, git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: reviews
---
# SR-586: PR #202 EARS and wording review

## Summary

EARS conformance and wording review of the new and rewritten requirement statements: FR-031, FR-037, FR-038 (edited criteria), FR-039 and FR-040.

Ticket: IR-22.

`make spec` at this head reports 249/249 documents grammar-clean with 0 grammar findings. The new statements use the event-driven (FR-037, FR-040) and ubiquitous (FR-031, FR-039) forms. No new requirement text contains a ticket id or PR number (grep of the added lines). No new text names an unsupported alternative outside AD-001's open questions, where the options are required.

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

**PASS** with two low findings about wording that describes the change itself.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | "so the ruling adds no repository edge" refers to a decision that is not in the spec. Better: "so taking qsl-replay adds no repository edge". | spec/interface/FR-039-root-crate-public-interface.md:82 |
| FND-002 | low | "Contract IR deletes its own copies: ..." describes a pending change. Once the code PR lands, it is a list of names that no longer exist. FR-039's "Items QSL owns" list carries weight (FR-039-AC-3 probes each name). AD-001 could state the ownership and point to FR-039. | spec/assurance/AD-001-contract-ir-architecture.md:77-80 |

## Dispositions

Round 1, reviewed at `24b077801d0dfed89cae39372dbb14afddd5d832`.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 24b0778 |
| FND-002 | fixed | 24b0778 |
