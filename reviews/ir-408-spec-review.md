---
id: SR-599
title: "spec review of PR 219 (ceremony sweep, spec and docs)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/**, README.md, CONTRIBUTING.md, schemas/README.md, corpus/contract-v0.1/README.md (git diff origin/main...HEAD)"
review_set: subset
---
# SR-599: spec review of PR 219

## Summary

Ticket: IR-408. This review covers the spec and docs diff of PR 219, which deletes AA-001, AP-001, AP-002, MP-001, MP-002, PGM-01, SUR-001, ADR-0055, NFR-005, TC-046 and TC-054, and strips issue/PR/SHA references, version, licence and publication keys, and tool/version tracking from the remaining files. It is judged against the owner's ruling: tracking ceremony is deleted, and identity and content digests stay. No live spec artifact (outside spec/reviews/) references a deleted ID, and no relative link points at a deleted file. FR-034 and STD-003 agree. `quire validate` passes (241/241). Coverage has the same 22 unbacked rows as origin/main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | spec/reviews/base.md keeps a relationship to the deleted NFR-005, and spec/reviews/ (still under spec/) carries SHAs and issue refs, against ADR-0056 layout rule 8 | spec/reviews/base.md:11 |
| FND-002 | low | Review and plan records keep relationship targets to deleted PGM-01, MP-001, MP-002, AP-001, NFR-005 and ADR-0055 | reviews/SR-035-rust-1.98.1-code-review.md:9 |
| FND-003 | low | The AD-001 risk "Self-model authority confusion" was deleted whole, dropping three controls that are not ceremony | spec/assurance/AD-001-contract-ir-architecture.md:183-193 |
| FND-004 | low | FR-036 dropped the whole "provider shall identify package, claim, domain, bounds..." sentence, including the content-identity parts | spec/contract/FR-036-exact-backend-negotiation-and-emission.md:51 |
| FND-005 | low | The rewritten schemas/README.md domain-schema table omits two live files in schemas/ | schemas/README.md:5-13 |

## Finding Detail

- FND-001: `spec/reviews/base.md:11` has `target: ix://agent-ix/quire-contract-ir/NFR-005`, and the PR deletes NFR-005. The file sits under `spec/`, and ADR-0056 layout rule 8 says "`spec/reviews/` does not exist". The directory also holds commit SHAs and issue numbers (for example `spec/reviews/kani-status-reconciliation/base.md:35-39`). This was already on main, but the dangling target is new. `quire validate` does not resolve ix:// targets, so no gate reports it. Fix: drop the NFR-005 relationship entry and prose at spec/reviews/base.md:11,54,78. Then either delete spec/reviews/ or move it to reviews/. That second step can be a follow-up.
- FND-002: these review and plan files keep ix:// relationships to IDs this PR deletes: reviews/REV-001-pgm01-composite.md:6, reviews/SR-001-pgm01-gap-analysis.md:9, reviews/SR-004-contract-risk.md:9 (AP-001), reviews/SR-005-contract-evidence.md:9 (MP-001), reviews/SR-032-drop-legacy-evidence-code-review.md:13, reviews/SR-033-drop-legacy-evidence-gap-analysis.md:13, reviews/SR-035-rust-1.98.1-code-review.md:9,11 (NFR-005, ADR-0055), reviews/2026-09-28-ir-22-spec-review-integrity-analysis.md:19 (MP-002), and plan/PLAN-001-pgm01/TASK-001-specification.md:9. Fix: delete the records whose only subject was deleted ceremony (REV-001, SR-001, SR-032, SR-033, SR-035, PLAN-001), or drop those relationship entries. This is out of the PR's stated spec/docs scope, so a follow-up is acceptable.
- FND-003: origin/main AD-001 listed this risk: "Self-model authority confusion: controlled by exact external manifest selection, constructor-private checked views, absent acceptance fields, and no conversion from model/proposal output into executable owner inputs." Only "exact external manifest selection" is selection/pin ceremony. The other three are domain safety controls. Fix: restore the bullet as "Self-model authority confusion: controlled by constructor-private checked views, absent acceptance fields, and no conversion from model/proposal output into executable owner inputs."
- FND-004: the removed sentence was "The provider shall identify package, claim, domain, bounds, options, toolchain, and dependencies." Toolchain and options are tool tracking. Package, claim, domain and bounds identification is content identity, which the ruling keeps. No FR-036 AC depended on the sentence, so nothing is orphaned. Fix: either restore "The provider shall identify package, claim, domain and bounds in each emitted artifact", or add a Dependencies line saying QSpec FR-196 owns artifact identification.
- FND-005: `schemas/` holds `contract-executable-projection-v1.schema.json`, which FR-023 uses and which references the fixture schema, and `conformance-trace-map-v1.json`, which FR-018:75 calls "the owned ... registry". The new table lists only the fixture and package schemas, and says "The runner reads both". Fix: add a row for each file, with its owning FR.

## Scope

- `DEL-PGM-01` (spec/program/PGM-01-governance.md), examined: governance, evidence, licensing and provenance policy (deleted).
- `DEL-AA-AP-MP` (spec/assurance/AA-001, AP-001, AP-002, MP-001, MP-002), examined: the assurance chain and measurement plans (deleted).
- `DEL-SUR-001` (spec/evidence/suites.md), examined: evidence suite registry SUITE-001..008 (deleted).
- `DEL-ADR-0055-NFR-005` (spec/decisions/ADR-0055, spec/nonfunctional/NFR-005), examined: Rust 1.98.1 baseline pin (deleted; rust-version stays in Cargo.toml).
- `DEL-TC-046-TC-054` (spec/contract/TC-046, TC-054), examined: withdrawn TC tombstones (deleted).
- `FR-034` (spec/contract/FR-034-assemble-output-package-atomically.md), examined: generator identity is the owner only; a structural observation reference names the package, observer owner and accepted/refused outcome.
- `FR-034-AC-4` (same), examined: Mutating source/profile/generator owner/record/limit/target-byte identity inputs changes or invalidates package identity, while path/time/locale/display/observer changes do not.
- `FR-034-AC-5` (same), examined: Structural observer acceptance, refusal, absence, and observer owner remain downstream references and cannot establish native truth or mapping preservation.
- `STD-003-invalid_generator` (spec/contract/STD-003-output-mapping-refusal-registry.md), examined: The declared generator owner is empty, unbounded, or outside visible ASCII.
- `STD-003-invalid_observer` (same), examined: The declared observer owner is empty, unbounded, or outside visible ASCII.
- `FR-019-yaml` (spec/interface/FR-019-rust-library-interface.md), examined: version and compatibility keys removed; ObserverResultDigest removed from the output_mapping items row.
- `FR-029-AC-1` (spec/contract/FR-029-versioned-bounded-kani-profile.md), examined: A selected profile binds its versioned lowering, harness, oracle/strategy, replay, bounds, and capability-matrix identities before lowering.
- `FR-036` (spec/contract/FR-036-exact-backend-negotiation-and-emission.md), examined: negotiate exact per-item capability, domains, and bounds before it emits an artifact or result.
- `TC-045` (spec/contract/TC-045-exact-backend-negotiation.md), examined: mutate each bound, encoding, capability, and requested claim.
- `TC-055` (spec/contract/TC-055-root-crate-public-interface.md), examined: Run the TC-041 and TC-042 negative corpora under catch_unwind (matches FR-039-AC-4).
- `FR-011` (spec/contract/FR-011-package-identity.md), examined: Dependencies None.; ACs unchanged in meaning.
- `FR-012`, `FR-013`, `FR-014`, `FR-016`, `FR-018`, `FR-023`, `FR-030`, `FR-031`, `FR-035`, `FR-037`, `FR-038`, `FR-344`, `FR-020`, `FR-039`, examined: issue/PR/SHA/date references and v0.1 prose removed; no AC meaning changed.
- `STD-001` (spec/contract/STD-001-diagnostic-registry.md), examined: issue-numbered sections renamed by subject; codes unchanged; PGM-01 upstream dependency removed.
- `NFR-001`, `NFR-002`, `NFR-003`, `StR-001`, `StR-003`, examined: issue notes and PGM-01 dependencies removed.
- `AD-001`, `AD-002`, examined: AP-001/AP-002 realizes edges removed; release-decision prose removed.
- `ADR-0053`, `ADR-0054`, `ADR-0056`, examined: owner-decision records, issue links, withdrawn-TC and adoption rules removed.
- `TM-002` (spec/contract-test-matrix.md), examined: issues/11 relationship, PR refs and Withdrawn Test Cases section removed.
- `spec/index.md`, examined: PGM-01 scope and references removed; requirement range corrected to existing FRs.
- `README.md`, `CONTRIBUTING.md`, `schemas/README.md`, `corpus/contract-v0.1/README.md`, examined: provenance and release-decision directives removed.

## Verdict

Mergeable once FND-001 is fixed. The rest are low and can go in a follow-up. The sweep follows the owner's ruling. Identity digests (FR-016 canonical digests, FR-034 raw target digest and package identity, FR-038 lock digests) stay, and so does the conformance runner's `--version`. No live spec artifact outside spec/reviews/ references a deleted ID. FR-034 and STD-003 agree: both carry the generator owner only and the observer owner only, and `invalid_observer` is kept as an owner bound. The FR-019 public-items row no longer lists `ObserverResultDigest`, as expected pending PR #218. `make spec`: validate passes (241/241). On origin/main it fails on MP-001 and MP-002, so this PR fixes that. Coverage reports the same 22 unbacked rows as origin/main (quire coverage run directly there), with 155 backed on both sides. Total rows go from 185 to 178, and no backed row is lost.
