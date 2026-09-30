---
id: SR-584
title: "PR #202 integrity review: QSL-owned replay, root API, V2 frame/state spec"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/ (29 files, git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-053
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-344
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-039
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: reviews
---
# SR-584: PR #202 integrity review: QSL-owned replay, root API, V2 frame/state spec

## Summary

Integrity analysis (completeness, consistency, atomicity) of the spec-only diff (29 files under `spec/`), checked against the code and against the binding owner ruling: QSL owns the replay and proof types in `qsl-replay`, Contract IR deletes its copies, codegen keeps the Kani transcript parser, replay runs only through `qsl_replay::replay`, and `runtime::execute` is retired.

Ticket: IR-22 (also IR-27, IR-89 per the PR title).

Confirmed at this head: no `runtime::execute`, `CheckedPackage::call`, `#243` or `complete-V1 executor` text remains anywhere under `spec/` (the only surviving names are the deletion lists in AD-001 and FR-039). FR-031-AC-3 and AC-4 and FR-037-AC-1 through AC-5 are retired, each with a reason. Every `qsl_replay` name the PR cites (`TerminalValue` with `Proved { success_checks }`, `ProofRefusalCause`, `IncompleteCause`, `UnavailabilityCause::{SolverAbsent, BackendAbsent}`, `InconclusiveCause::KaniVacuousProof`, `TerminalRecord`, `Witness`, `ReplaySource`, `WitnessEnvelope`, `ReplayRequest`, `ReplayResult`, `ObligationIdentity`, `replay`) exists in `qsl-replay` at the QSL revision the root crate already pins. FR-039's `bridge` and `kani` rows match `src/` exactly, and its "Items QSL owns" list matches `src/kani/{replay,witness,outcome}.rs` exactly. AD-001's four versioned contracts match the code (`SchemaVersion` 1.0/1.1 with one registered 1.0-to-1.1 migration, `quire.checked-package/v2`, `quire.contract-ir.contract-package/v1` / `lowered-node/v1`, `kani-bounded/1`). OQ-1 to OQ-3 are stated as open questions with options, and OQ-1's and OQ-2's factual premises hold (QSL depends on the model crate under the `quire-contract-ir` alias; codegen's `bounded_kani_corpus.rs` consumes the family lowerings). ADs and APs stay `proposed`. No ID collision exists in `spec/` and cross-repo references are prefixed rather than renumbered.

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

**FAIL**. One high finding: FR-031's outcome map cannot be implemented as written, because FR-031, FR-039 and AD-001 call it total while FR-031 leaves two kinds unmapped, and the zero-check case AC-5 requires is the case the current outcome type routes to an unmapped kind. Three medium findings leave unchanged files (TC-053, FR-344, TC-222) contradicting the rewritten FR-038 and the new FR-040, and FR-039's inventory table cannot back its own "exactly" criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-031's outcome map is not implementable as written. FR-031:34-35, FR-039:78 and AD-001:82 require every KaniOutcome to map to exactly one TerminalValue through a total, exhaustive map. FR-031:71-74 leaves Unavailable and Inconclusive with no TerminalValue until OQ-3 is decided. Proved{success_checks} needs a check count, but KaniOutcome (src/kani/outcome.rs:92-101) carries none. KaniOutcome::proved_from_checks(0) builds kind Inconclusive with code kani_vacuous_proof, and that is exactly the kind FR-031 leaves unmapped, so the zero-check case AC-5 requires to map to Proved{0} cannot reach that value. Failure scenario: the TC-223 coder has to pick one of three workarounds. (1) An Option return, which breaks "total". (2) An invented arm for the two kinds, which silently decides OQ-3. (3) A count field on FR-030's KaniOutcome, which no requirement states. | spec/contract/FR-031-bounded-kani-dispatch-replay-provenance.md:59-74 |
| FND-002 | medium | TC-053 is unchanged but contradicts the rewritten FR-038-AC-12 and the matrix TC-053 row. It still enumerates "six eligible triples", including the modifies relationship/field_declaration pairs that FR-040 now owns. It still replays "every published frame_mutations vector" ("All 26"), although QSpec now publishes 30 in the FrameModifiesEntry shape. Its status is still "Blocked" on #166, while the matrix row says implemented, with locally authored cases. Failure scenario: a reader of the TC file and a reader of the matrix get two different test methods and two different statuses for one row. | spec/contract/TC-053-checked-package-v2-frame-bodies.md:21-54 |
| FND-003 | medium | FR-344:57-59 still says FR-038 validates a model node against the "closed eighteen-meaning list". TC-222 case 3 (TC-222:35-40) builds its extra-member regression on semantic_form field_declaration, which it calls "an existing admitted form". FR-040-AC-6 refuses field_declaration. Failure scenario: once FR-040 lands, TC-222 case 3 either refuses on the form instead of on the extra member it exists to pin, or it passes for the wrong reason. Either way the regression stops discriminating. | spec/contract/FR-344-admit-or-refuse-the-adr-002-2-0-0-members.md:57-59 |
| FND-004 | medium | FR-039-AC-1 requires an inventory that "fails on an added or missing item", but the Public items table is not an exact list. Several rows use category phrases: "the decision and cause types", "the admission, correspondence, join and decision types", "the PROFILE family of profile constants", "the manifest module" and "the profile and schema constants". The temporal row also omits ExpectedTemporalJoin and ExpectedTemporalProjection, which are public at origin/main (src/temporal/mod.rs:22). The bridge and kani rows match exactly. Failure scenario: TC-055 either fails on day one on the temporal row, or the author decides for themselves what each category phrase contains. | spec/interface/FR-039-root-crate-public-interface.md:70-78,109 |
| FND-005 | low | AD-001 OQ-1 says the model crate has "`pub use binding::*` and seven more", which means eight glob re-exports. crates/quire-contract-model/src/lib.rs:31-47 has seven globs in total (binding, canonical, checked_package, coverage, expression, identity, output_mapping) plus two explicit lists (conformance, limits). The count is off by one. | spec/assurance/AD-001-contract-ir-architecture.md:155 |
| FND-006 | low | MP-002 is unchanged and still measures "native replay" agreement as part of Contract IR's bounded-Kani parity. It is still the control_ref for AP-002's impact-unreplayable-counterexample, which this PR rewords to qsl_replay::replay. After the ruling, the replay agreement MP-002 counts is QSL's ReplayResult as codegen records it. The procedure should say so. | spec/assurance/MP-002-bounded-kani-parity.md:19,74,78 |
| FND-007 | low | FR-037-AC-6 and TC-046 duplicate FR-039-AC-2 and AC-3 and TC-055. Both pairs inventory the public API for QSL-owned replay types and search src/ for an executor call, so two test cases verify one property. | spec/contract/FR-037-canonical-backend-replay-and-qualification.md:57 |
| FND-008 | low | The TC-053 cases row contains a stray ".;" left over from an edit: "...regardless of which defect class the other carries.; the cases above are locally authored". | spec/contract-test-matrix.md:123 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | low | Some wording still says every outcome has a terminal value, although FR-031's map is no longer total. FR-039 Outputs (62-64) says typed KaniOutcomes, "each with its QSL TerminalValue". The TC-223 title and matrix row (TC-223:3, matrix:90) say "Every Kani outcome kind maps to its one QSL terminal value". TC-055:28-30 says to compile a probe that "binds the kani outcome map's result to qsl_replay::TerminalValue". The map now returns a TerminalValue or a typed absence, so that probe does not build as written. Failure scenario: the TC-055 author writes a probe that fails to compile, or narrows the map's type to make it build. | spec/interface/FR-039-root-crate-public-interface.md:62-64 |
| FND-010 | low | Cross-repo, for information only. FR-030 now says the zero-check rule is "the one shared rule every caller that classifies a Kani run routes through". FR-030-AC-4 also says no public constructor builds a proved outcome without a count of at least one. Codegen origin/main calls KaniOutcome::proved(..) with no count at src/bounded_kani_corpus.rs:346 and src/bounded_kani_replay.rs:74. Its classify_run does route through proved_from_checks (src/kani_execution.rs:942). The IR code PR that implements AC-4 breaks those two call sites. Those callers need a coordinated codegen change. No change is needed in this PR. | spec/contract/FR-030-bounded-kani-domain-and-outcomes.md:33,44 |

## Dispositions

Round 1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | resolved |
| FND-002 | fixed | resolved |
| FND-003 | fixed | resolved |
| FND-004 | fixed | resolved |
| FND-005 | fixed | resolved |
| FND-006 | fixed | resolved |
| FND-007 | fixed | resolved |
| FND-008 | fixed | resolved |

Round 2.

| FND | outcome | reason |
| --- | --- | --- |
| FND-009 | fixed | resolved |
| FND-010 | deferred | This belongs to the Contract IR code PR that implements FR-030-AC-4 and TC-223. That PR adds the counted proved constructor, and it has to update codegen's count-less KaniOutcome::proved callers (src/bounded_kani_corpus.rs:346, src/bounded_kani_replay.rs:74) at the same time. The team leader recorded it on IR-22. No spec text in this PR is wrong. |
