---
id: SR-1081
title: "gap analysis of PR 259 after rebase (IR-535 selections bind by identity)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@22ed70872d8e5cc42fcd68c882428399facec766; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, tests/it/checked_package_v2_reader.rs, tests/it/checked_package_v2_dependency_selections.rs, tests/it/checked_package_v2_model_members.rs, tests/conformance_qspec/main.rs"
review_set: subset
---
# SR-1081: gap analysis of PR 259 after rebase

## Summary

Ticket: IR-535. Reviewed head 22ed70872d8e5cc42fcd68c882428399facec766 against
origin/main cbcd790a8fc9d07c83c7c7f6238023ed85fa0bad. This review is planless, so plan
completion is not assessed. It replaces SR-821, which reviewed the pre-rebase head.

Spec. FR-038's criteria and prose are already on main (IR-534, #252). This PR changes only
the matrix: the FR-038 row and the TC-048 row in `spec/checked_package/matrix/tests.md`.
The FR-040 and TC-056 rows are untouched, and no count is re-added. I read AC-62, AC-63
and AC-64 and the amended AC-2, AC-10, AC-19, AC-20, AC-27, AC-31, AC-32 and AC-45. Each
clause maps to a tagged test that asserts the whole code, cause and pointer:

- AC-62 `tc_048_a_model_row_is_exactly_identity_digest_domain_and_digest`. It covers the
  version member in the lock only, in the preimage only, and in both beside a well-formed
  row, plus each member absent or of the wrong kind.
- AC-63 `tc_048_a_dependency_entry_carrying_version_is_an_unknown_member`. It covers the
  same three positions and the old shape `{identity, version}` giving `malformed_wire`.
- AC-64 is tagged on three tests: the dependency bind test (`missing_import` and
  `byte-digest-mismatch`), the "needs no package version" test, and the owner-join test
  (unnamed owner gives `invalid_semantic_graph`).
- AC-20 `tc_048_model_selection_same_identity_different_digest_refuses_as_stale_dependency`.
  It covers both orders, the four "before any read" evidence cases, an earlier missing
  row, and class 2 outranking from either entry.
- AC-27 and AC-45 (lock-row clause) are on the "needs no package version" test.

I found no stale "earlier shape" or "planned" text for IR-535 in the FR-038 row, the
TC-048 row or the TC-048 "Selections bind by identity" section. The FR-038 prose names four
classes and states the invariant. The diff changes no AC ids or numbering. The matrix ranges still skip AC-16, AC-34 and the
retired AC-66, as they do on main.

Counts, from my own `quire coverage --strict` runs:

| | Coverage | FR-038 | Unbacked | Contradicted |
| --- | --- | --- | --- | --- |
| head | 220/261 | 98/108 | 23 | 0 |
| origin/main | 217/261 | 95/108 | 23 | 0 |

The +3 is AC-62, AC-63 and AC-64. The 10 FR-038 rows still unbacked are AC-81 to AC-86,
AC-88 and AC-109 to AC-111, all planned on main.

QSpec. Its merged `proposals/checked-package-v2/schema.json` at 974fb24 has `ModelRef`
`{identity, digest_domain, digest}` and `DependencySelection` `{identity, package_id}`.
Neither has a `version`, and both set `additionalProperties: false`.
`node-identity-preimage.schema.json` defines no selection rows. So there is no HIGH
mismatch. QSL's TC-253 gap note ("selections still carry version") does not describe
QSpec's merged schema. All six positive fixtures carry empty `model_selections` and
`dependency_selections`, so AC-107 (three fixtures) does not exercise rows.

IR's own `schemas/` and `corpus/` do not mention selections, so neither needed a change.

Consumers, read-only:

- QSL main a7017bea pins IR at afb01a2 (#253). Its breakage on repoint:
  - `qsl-package/src/emit.rs:843` and `:980` build `CheckedDependencySelection` and
    `CheckedDomainPackageRef` with `version`, which is a compile error.
  - The module doc at `emit.rs:25-29` is stale.
  - The `checked_v2/tests.rs` helper `dependency_selection` (line 70) builds entries with
    `version`. The tests at lines 776 and 1951 would then draw `unknown_member` at
    `/identity_preimage/dependency_selections/0/version` instead of their expected
    refusals.
  - Every QSL package with selections gets a new `package_id`.
  - `checked_v2.rs:977` uses QSL's own `selection.version`, not IR's, so it does not break.
- CG main aba2403: no use of the changed IR types. Its fixtures carry empty arrays. Its
  `DependencyLock`/`DependencyEntryWire` `version` is a QSL replay wire, not IR, so this PR
  breaks nothing in CG.
- quire-driver main fc53a75: no reference.

Status of SR-821:

- FND-001 (FR-040/TC-056 rows): resolved by #250 on main.
- FND-002 (AC-64 and AC-45 tags): fixed.
- FND-003 (PR body): fixed. The body now names the compile break and the `package_id`
  change. FND-002 below covers its QSL list going stale.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No test in this repo pins a `package_id` over non-empty selections against an outside value. The preimage change is checked only by the in-repo JSON hash (`refresh_identity`), and AC-107's QSpec fixtures all have empty selection arrays. QSpec publishes `dependency-selection-vectors.json`, with a recorded package_id 0606043a… over version-free entries and five entry mutations with outcomes. A throwaway probe shows this reader derives that value. Nothing keeps it that way: a reader and fixture that drifted together would still pass. This is a candidate follow-up under `conformance-qspec`, not a blocker | tests/conformance_qspec/main.rs:28-33, tests/it/support/checked_package.rs:185-210 |
| FND-002 | low | The PR body's lockstep list for QSL is stale against QSL main a7017bea. It cites `emit.rs:890-895` and `:754-758`, which are now lines 980 and 843. It says `checked_v2.rs:970-974` passes a version to `insert_dependency_package`, but QSL main already calls it with two arguments. It says `emit.rs:508` still builds `NominalOwner::Model { version }`, which QSL has already fixed. It omits the `checked_v2/tests.rs` fixtures that will refuse `unknown_member` after the repoint. Untrusted PR text is the planner's lockstep checklist, so it should be refreshed before the QSL pair is scheduled | quire-contract-ir#259 PR body |

## Verdict

The backing is real. AC-62, AC-63 and AC-64 and the amended clauses are implemented, and
each is backed by a test that a targeted mutant fails (SR-1080). The matrix flips are true,
and the counts match main plus 3. The wire matches QSpec's merged schema. Two low
findings, neither blocking.

## Dispositions

Round 1, reviewed at ae9e3f16b7af1e5f06cfb8e04c543a19735a7518. I re-measured the PR
body's QSL list against quire-spec-language f6c3974811f6bd8cf6475230efd48676cc4eae36,
whose files emit.rs, checked_v2.rs, checked_v2/tests.rs and Cargo.lock equal current QSL
origin/main 6f314877. Each item holds:

- `emit.rs:843` and `:980` set `version`.
- `emit.rs:25-29` describes rows with a version.
- `checked_v2/tests.rs:70` builds entries with `version`, and the calls at 778-779 and
  1951-1952 use it.
- `checked_v2.rs:956` calls the two-argument `insert_dependency_package`.
- `checked_v2.rs:977` reads QSL's own `selection.version`.
- Cargo.lock pins IR at afb01a23d1e047feff3159d96551663702187957.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | deferred | Routed to IR-552: a conformance-qspec test over QSpec's dependency-selection-vectors.json. The reader matches QSpec's recorded package_id today (SR-1080 probe), so it is not needed for this PR. As of this round, IR-552's description and comments do not mention the vector, so the routing must be written into IR-552 for the deferral to stick |
| FND-002 | fixed | ae9e3f16b7af1e5f06cfb8e04c543a19735a7518 (PR body refreshed; no repo change) |
