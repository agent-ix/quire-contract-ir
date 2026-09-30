---
id: SR-585
title: "PR #202 evidence review: TC-046, TC-054..057, TC-221..223 and coverage claims"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/ (29 files, git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-221
    type: reviews
---
# SR-585: PR #202 evidence review: TC-046, TC-054..057, TC-221..223 and coverage claims

## Summary

Evidence and verification-method analysis of the PR #202 test cases and matrix rows.

Ticket: IR-22.

Coverage claims were re-measured with my own runs of quire 0.33.0 (engine 0.47.1). `quire coverage --scope . --strict` reports 21 unbacked rows and 0 contradicted statuses, and 39 unbacked and 0 contradicted. Both runs exit 1, as `--strict` does with unbacked rows. `make spec` exits 2 at both revisions, and both runs report the same "2 document(s) failed structural validation" from the MP-001 and MP-002 frontmatter, so the PR introduces no new structural failure. The 18 added unbacked rows are the planned FR-037, FR-039, FR-040, FR-038-AC-40, TC-046 and TC-054 to TC-057 rows. The PR's claims match. The paths the PR adds or fixes under `tests/`, `src/` and `crates/` all exist at origin/main. TC-020's four AD-001 markers (`type: ArchitectureDescription`, `## System Boundary`, `## Risks`, `owner: kreneskyp`) are present at this head (AD-001:4, 6, 34, 212). I confirmed this by reading the file. I did not run `cargo test --test it foundation`: it needs a full git-dependency build, and the assertion is plain substring matching.

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

**FAIL** on two medium findings. TC-057 does not say which of the five published arrays it re-derives, and no test consumes the 67 published mutation oracles. TC-056 promises exact codes that FR-040's criteria never name.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-40 and TC-057 say "every vector" of node-identity-vectors.json, but the file has five arrays: vectors (17), operation_vectors (21, which include function-call), invalid_mutations (13), operation_mutations (24) and frame_mutations (30, in the FR-340 FrameModifiesEntry shape). TC-057 names "nominal and application preimage encoders" but no array, and no test case reads the 67 published mutation cases. TC-056 authors its own frame mutations instead. Failure scenario: TC-057 passes on the 17 nominal vectors alone, and FR-040's frame refusals are checked only against expectations this repository wrote itself. That is the tautology risk the PR itself names in TC-057's Description. | spec/contract/FR-038-consume-checked-package-v2.md:647 |
| FND-002 | medium | TC-056's Expected Results require "exactly the code, cause and RFC 6901 locus its criterion names". Several FR-040 criteria name none and say only "refuse(s)": AC-5 (occurrence role), AC-7 (reordered bindings, non-text operation, missing binding, role), AC-8 (other identity, operator class, member kind, clause value, term shape, role) and all of AC-11. QSpec fixes these cases at schema level only. Contract IR's reader maps a schema-shape failure to invalid_semantic_graph (checked_package/v2/mod.rs:1426-1450). The criteria should state that mapping and the locus. Failure scenario: the TC-056 author invents the expected code and locus. | spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:190,192,193,196 |
| FND-003 | low | FR-040-AC-7 leaves out the operation-resolution cases of QSpec FR-342-AC-4: an undeclared name and a field's name refuse missing-name, an unselected version refuses missing-selection, and (Sub, "size") resolves to Sub's redefinition. FR-040's Behavior also drops "with redefinitions applied". Behavior names the refusals, but no criterion or test case backs them. | spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:192 |
| FND-004 | low | The TC-221 row stays ✅ against the retired FR-031-AC-4. Seventeen #[trace("TC-221", "FR-031-AC-4")] tests in tests/it/kani_replay.rs still trace a criterion that is no longer declared, and quire coverage does not flag them (FR-031 3/3). The withdrawn TC-054 row still counts as one of the 39 unbacked rows. Both are informational until witness.rs and replay.rs are deleted. The code PR that deletes them should delete these traces too. | spec/contract-test-matrix.md:89 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The strict coverage count shows false green for the rows behind the original HIGH. quire coverage --strict reports 37 unbacked and 0 contradicted. Neither the TC-223 row nor FR-030-AC-4 appears in the unbacked list, because the legacy #[trace("TC-223", "FR-031-AC-5")] test in tests/it/kani_shared.rs:251 backs the TC-223 row. That test verifies the superseded map onto KaniProviderResult. quire's own per-file line disagrees: FR-030 3/4. The matrix prose is honest (both rows are planned, and it names the legacy test). The aggregate gate number is not. Recommended fix: in the code PR that implements TC-223, retag or delete the legacy trace in the same commit. No change is needed in this PR. | spec/contract-test-matrix.md:90 |
| FND-006 | low | Withdrawn test cases are treated inconsistently. TC-054 and TC-046 moved to the new "Withdrawn Test Cases" section (matrix:105). TC-221 is also withdrawn, but it keeps its row in the Test Case Summary table (89) and its Coverage Design row (128). Keeping TC-221 in the table is defensible while 17 tests still trace it. The Withdrawn section should list it too, or say why it is absent. Nothing is hidden: TC-046 and TC-054 have status: withdrawn, and no test or source file traces either one. | spec/contract-test-matrix.md:89,105,128 |

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | resolved |
| FND-002 | fixed | resolved |
| FND-003 | fixed | resolved |
| FND-004 | fixed | resolved |

Round 2.

| FND | outcome | reason |
| --- | --- | --- |
| FND-005 | deferred | This belongs to the Contract IR code PR that implements TC-223. That PR must retag or delete the legacy #[trace("TC-223", "FR-031-AC-5")] test in tests/it/kani_shared.rs:251 in the same commit, so that quire's strict count stops crediting TC-223 and FR-030-AC-4. The team leader recorded it on IR-22. The matrix prose already discloses the legacy test, and no spec text in this PR is wrong. |
| FND-006 | fixed | resolved |
