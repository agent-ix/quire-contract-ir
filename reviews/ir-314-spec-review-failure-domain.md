---
id: SR-594
title: "failure-domain review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@11c6b013a0c94ddbf76040cbe81142858935eae3; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-594: failure-domain review of PR 203

## Summary

Ticket: IR-314. Failure modes the restructure gate and identifier rules leave unstated, checked against the tests and scripts that read spec/ paths.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Gate rule 8 misses checks that silently shrink scope (validate_matrix_status.py) and does not require make test/make assurance. | spec/decisions/ADR-0056-spec-layout-convention.md:267-275 |
| FND-002 | medium | Unprefixed foreign IDs already in code (TC-280/281, TC-221, FR-322...) will bind to this repo's own IDs when next-free reaches them. | spec/decisions/ADR-0056-spec-layout-convention.md:160-167 |
| FND-003 | low | Rule 5 permits byte edits to FR-025/FR-026, whose bytes production code digests. | spec/decisions/ADR-0056-spec-layout-convention.md:258-261 |

## Finding Detail

**FND-001** (medium, confidence high, soundness; spec/decisions/ADR-0056-spec-layout-convention.md:267-275): Rule 8 only catches a gate that gains findings. scripts/validate_matrix_status.py:24,101-102 enumerates fixed SPEC_DIRECTORIES with a non-recursive glob; on a missing or partial directory list it silently reads fewer requirements and its criterion-citation check reports fewer failures, so a restructure that weakens it passes rule 8 and rule 7 ('still names an existing path'). Rule 8 also does not require make test or make assurance, which run the include_str!/include_bytes!/read_dir consumers and the sealed assurance/change-assurance.json (configuration: spec/index.md). Fix: require make ci green at head, and require each enumerating check to read the same document/ID set before and after.

**FND-002** (medium, confidence medium, soundness; spec/decisions/ADR-0056-spec-layout-convention.md:160-167): This repo's code already carries unprefixed foreign IDs that quire binds by ID alone: TC-280/TC-281 (QSpec vectors) in doc comments at crates/quire-contract-model/src/checked_package/v2/operations/model_member_vectors.rs:439-752, #[trace("TC-221", ...)] at tests/it/kani_replay.rs:306 (TC-221 is not declared here; quire-specification declares one), FR-322/FR-331/FR-340. The next-free rule looks only at local frontmatter, so when this repo mints its own TC-280 those tags back it: false coverage. Fix: require existing unprefixed foreign IDs to be prefixed (or put into a reserved block) before the next-free sequence reaches them.

**FND-003** (low, confidence medium, other; spec/decisions/ADR-0056-spec-layout-convention.md:258-261): src/temporal/admission.rs:282-284 and src/temporal/request.rs:315-317,322-324 digest the bytes of FR-025 and FR-026 (include_bytes! into BridgeDigest). Rule 5 allows link-target edits in moved files, which changes those digests silently. Neither file has relative links today, so this is latent; name digest-bound files as byte-frozen in rule 5.

## Scope

- `ADR-0056-gate` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Restructure gate 1-8: structural first (adds no ID), one writer, relocation map TSV old_path/new_path/old_id/new_id, ID-set equality, no meaning change, link check, embedded-path scan, gates unchanged.
- `ADR-0056-identifiers` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: IDs are flat and global per repository per family ... never carries a subsystem prefix ... never renumbered by a move ... An unprefixed ID names an artifact in the same repository ... ix://agent-ix/<repo>/<ID> ... next free ID ... A spec artifact is a document under spec/ outside spec/reviews/.
- `FR-345-AC-5` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: With SPEC_BASE set, a structural change that drops, adds or renumbers an ID, or renames a file with no row in an added relocation map, fails the check; the same moves with a complete, ID-preserving relocation map pass.

## Verdict

Embedded spec/ path consumers found by repo-wide search: tests/it/{governance_reconciliation,governance,ecosystem_model,identity,output_mapping,conformance,foundation,toolchain_policy}.rs, src/temporal/{request,admission}.rs (include_bytes! on the line after the macro), scripts/validate_matrix_status.py, scripts/assurance_chain.py (writes a hermetic probe spec/index.md, unaffected), tests/test_matrix_status.py and tests/test_native_orchestration.py (fixtures/reads), assurance/change-assurance.json (sources and configuration spec/index.md), README.md and CONTRIBUTING.md links. Gate rule 7's generic scan covers them by path string. TC-020 (tests/it/foundation.rs:51) reads spec/assurance/* markers, which the layout keeps in place: unaffected.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Adoption site list misses tests/it/support/checked_package.rs and Makefile:109-112 foreign IDs and four spec-path doc comments. | spec/decisions/ADR-0056-spec-layout-convention.md:362-370 |

**FND-004** (low, confidence high, coverage; spec/decisions/ADR-0056-spec-layout-convention.md:362-370): The 'as found at acceptance' site list is incomplete. git grep at 5a3d953 also finds unprefixed FR-322 in tests/it/support/checked_package.rs:321,556,577,581,1168 and FR-340 at :461, and TC-280/TC-281 plus FR-322-AC-36 in Makefile:109-112. The readers list also omits the spec-path doc comments in crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:33, crates/quire-contract-model/src/output_mapping.rs:176, tests/it/checked_package_v2_frame_bodies.rs:107 and tests/it/checked_package_v2_qsl_parameters.rs:42. Rules 7 and 9 still require them, so the risk is a restructure author trusting the list; add the sites or say the list is not exhaustive.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5a3d953 |
| FND-002 | fixed | 5a3d953 |
| FND-003 | fixed | 5a3d953 |

Correction to the original row's wording: TC-221 is this repository's own withdrawn test case (spec/contract/FR-031...:109, spec/contract-test-matrix.md:115), not only a quire-specification ID; the ADR now handles it as a withdrawn-ID trace tag to remove.
