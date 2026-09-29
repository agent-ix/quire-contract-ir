---
id: SR-587
title: "PR #202 scope-boundary review against the QSL-owned replay ruling"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@5c4a5059d0b8965687195b23534cbd90e4791442; spec/ (29 files, git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: reviews
---
# SR-587: PR #202 scope-boundary review against the QSL-owned replay ruling

## Summary

Scope-boundary and responsibility-allocation review against the binding owner ruling.

Ticket: IR-22.

Ruling conformance at this head:

- QSL owns Witness, ReplaySource, the counterexample envelope, the FR-331 terminal record and ObligationIdentity in `qsl-replay`. Stated in AD-001, FR-031, FR-037 and FR-039.
- IR deletes its copies. FR-039 "Items QSL owns" lists all 18 current public names, and the list matches `src/kani/mod.rs` at `48ab5dc`.
- CG keeps Kani transcript parsing in its backend adapter. Stated in AD-001, AD-002, AD-003 and FR-031.
- Replay goes through `qsl_replay::replay` from the codegen replay adapter. Stated in FR-031, FR-037, AD-002 and AD-003.
- `runtime::execute` is retired. No reference remains in `spec/`, and FR-039-AC-2 forbids `quire_spec_language::runtime`.

OQ-1 to OQ-3 are real open questions and are not decided silently. FR-031 explicitly leaves the two OQ-3 kinds unmapped, although the rest of FR-031 contradicts that (SR-584 FND-001).

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

**PASS**. The allocation matches the ruling. Three low findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-037's Description places a shall on QSL and codegen: "the codegen replay adapter builds ... and calls qsl_replay::replay". Contract IR cannot verify that. Its verifiable content is AC-6 alone. Reword the statement to the Contract IR boundary, and cite QSL and codegen for the rest. | spec/contract/FR-037-canonical-backend-replay-and-qualification.md:27-32 |
| FND-002 | low | Cross-repo, for information only. QSL's ADR-011 E9 row (quire-spec-language spec/decisions/ADR-011 at origin/main, line 262) still describes the replay request as "the IR packet plus the #231 envelope members". FR-031, FR-037 and AD-002 cite E9 as their authority. QSL's text lags the ruling. Contract IR needs no change, and no gate should wait on QSL. | spec/contract/FR-031-bounded-kani-dispatch-replay-provenance.md:39 |
| FND-003 | low | FR-040 references ix://agent-ix/quire-specification/FR-341, but QSpec origin/main has two FR-341 files: interfaces/FR-341 (state clause body) and temporal/FR-341 (infinite-trace result disposition). The reference is ambiguous in QSpec's graph. The collision is QSpec's, so this is for information only. | spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:12 |

## Dispositions

Round 1, reviewed at `24b077801d0dfed89cae39372dbb14afddd5d832`.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 24b0778 |
| FND-002 | accepted-no-change | The finding concerns QSL's ADR-011 E9 text. That text is QSL's, and fixing it is QSL's work. Contract IR's citation is correct under the ruling, and QSL gates do not wait on downstream repos. |
| FND-003 | fixed | 24b0778 |
