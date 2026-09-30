---
id: SR-628
title: "integrity review of PR 232 (removed requirements and their references)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@122c21f6cacb25222b35b6629d19ed90e541d81a; spec/assurance/AD-001-*, spec/contract/FR-028-*, spec/contract/FR-031-*, spec/interface/FR-019-*, spec/decisions/ADR-0056-*; cross-repo citations in agent-ix/quire-specification and agent-ix/quire-contract-codegen origin/main"
review_set: subset
---
# SR-628: integrity review of PR 232

## Summary

Ticket: IR-448. This review checks internal consistency after the removals, and every citation of the removed ids across the ecosystem. The PR deletes FR-039, FR-019-AC-5 and FR-037-AC-6 on the ground that they are false today. The same false-today claims remain as present-tense description in paragraphs the PR edited. Seven documents in two other repos still cite the removed IR ids.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Present-tense claims the PR's own rationale calls false today are kept, including in a sentence the PR edited; FR-019 now contradicts AD-001 on glob re-exports | spec/assurance/AD-001-contract-ir-architecture.md:42-49 |
| FND-002 | medium | QSpec FR-196, FR-197, ADR-005 and codegen FR-014, FR-018, FR-021, FR-022, src/kani_obligations.rs and six test doc comments still cite IR FR-036, FR-037 or TC-045, which this PR deletes | quire-specification spec/functional/ir-proof/FR-196-generate-kani-contracts.md:12 |
| FND-003 | low | ADR-0056 uses TC-058 as its example of a padded id; TC-058 no longer exists | spec/decisions/ADR-0056-spec-layout-convention.md:138 |

## Finding Detail

- FND-001: several paragraphs still state the removed rules as current fact.
  - AD-001:42-43: "Its crate root re-exports each public item by name from its module, and the items it re-exports are exactly FR-019's public item table". The model root has seven glob re-exports.
  - AD-001:47-49: "... and it re-exports no `quire-contract-model` item". `src/lib.rs:12` is `pub use quire_contract_model::*`. The PR edited this sentence to drop "(FR-039)" and left the false clause in place.
  - FR-031:58: "the root crate carries no family lowering". `src/kani/arithmetic.rs`, `collections.rs` and `objects.rs` exist.
  - FR-031:73: "Contract IR defines no counterexample packet, witness, replay source ...". `src/kani/witness.rs` and `replay.rs` exist.
  - FR-028:28 says the root package "re-exports none of its items" as fact. FR-028's own Status admits AC-2 is planned.
  - FR-019:53 now says only "The crate root re-exports them from their modules". AD-001:155-156 still requires "by name ... with no glob re-export", so the two documents disagree.

  Fix: phrase these as target state, the way the PR's own AD-001 Replay ownership rewrite does ("target is to ..."), and restore the no-glob target to FR-019 as prose, not as an AC.
- FND-002: the citations were measured on each repo's origin/main.
  - quire-specification: FR-196:12 and ADR-005:9 (`depends_on`) point at `ix://agent-ix/quire-contract-ir/FR-036`. FR-197:12 points at `.../FR-037`.
  - quire-contract-codegen: FR-014:12, FR-018:12 and FR-021:14 point at `ix://agent-ix/quire-contract-ir/FR-036`. FR-022:59 says "under the Contract IR FR-036 vocabulary". `src/kani_obligations.rs:5` cites it. `tests/it/kani_obligations.rs` has doc comments at :1, :487, :631, :834, :1129, :1271 and :1427, each reading "Upstream: agent-ix/quire-contract-ir FR-036-AC-n, TC-045".
  - QSL ADR-013:903 mentions "(IR FR-039)" as history, which is harmless.

  None of these fails this repo's CI, but after merge they point at nothing. FR-036's content, the per-item `supported`/`requires_bound`/`unsupported`/`invalid_request` disposition vocabulary, is the vocabulary codegen says it implements. It should be re-homed (codegen FR-019/FR-015 is the natural owner) and the links repointed. File Linear follow-ups for QSpec and codegen. The owner should approve removing a requirement that another repo's ADR `depends_on`.
- FND-003: the id is illustrative, and the ADR's point still holds. Optionally swap in a live id.

## Scope

- spec/assurance/AD-001 root-crate paragraph and Replay ownership, examined (FND-001). The Replay ownership rewrite itself is accurate.
- spec/assurance/AD-003 risk row, examined: the rewrite is accurate.
- spec/contract/FR-028 Status edit, examined: "re-export, which AC-2 excludes" is correct.
- spec/contract/FR-031 Dependencies and Status edits, examined: the edits are accurate; the surrounding present-tense text is FND-001.
- spec/interface/FR-019 surface paragraph, examined (FND-001).
- The ecosystem grep for FR-036, FR-037, FR-039, FR-019-AC-5, TC-045, TC-055 and TC-058 of this repo, examined (FND-002). The FR-019-AC-5 hits in quire-specification are that repo's own FR-019, not this one.

## Verdict

Changes requested at medium. The removals are internally reachable-clean, but they leave inconsistent present-tense claims here and dangling upstream references in two repos.
