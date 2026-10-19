---
id: "SR-13230"
title: "IR PR 479 spec review: ContractPackage reader refusal criteria FR-035-AC-12 to AC-14"
type: SpecReview
analysis: base
review_set: subset
scope: "agent-ix/quire-contract-ir@f5229f01c640aa772fe7ba1c109a3e94dfe75b26; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13230: IR PR 479 spec review

## Summary

Ticket: IR-893. PR: agent-ix/quire-contract-ir#479 at f5229f0. Spec-only (three Markdown files).

The PR adds FR-035-AC-12 (missing top-level `source_map`), AC-13 (leading whitespace) and
AC-14 (first `feature_accounting` record names a different requested node). It adds TC-047
procedure paragraphs for AC-12 to AC-14 and for FR-035-AC-2 (stale own identity, stale
selected dependency), adds FR-035-AC-2 to TC-047's Traces To, and widens the FR-035 matrix row
to AC-14. AC-1 to AC-11 are unchanged.

Each claimed behaviour was reproduced against the code with throwaway probe tests in a reviewer
worktree, over the `mixed_fixture()` wire from `tests/it/complete_v1_contract_package.rs`. The
reader is `ContractPackageWireV1::read` / `admit` in
`crates/quire-contract-model/src/checked_package/v2/contract_package.rs:905-971`.

- AC-12: removing `source_map` gives `Refused`, `InvalidPackage`, path `Some("")`, cause
  `MissingMember`. The same result holds for each of the six required top-level members,
  including `contract_version`, which takes a separate branch at line 955. Confirmed.
- AC-13: leading space, tab, LF, CR, or two spaces each give `NoncanonicalWire` with
  `path: None`, `cause: None`. Trailing space gives the same result. Confirmed.
- AC-14: setting `feature_accounting[0].node_id` to `requested_items[1..=3]` gives
  `InvalidPackage`, `/feature_accounting`, `InvalidValue`, whether the identity is rehashed or
  stale (`validate_accounting` runs before `validate_identity`, lines 966-967). Index 4 repeats
  the first node (`dddd`), so it is not "other than" and is admitted, which is consistent.
  Confirmed.
- AC-2 cases: stale own digest gives `InvalidPackage`, `/identity/digest`, `InvalidValue`.
  The stale selected dependency (selection digest zeroed, identity rehashed, original
  dependency supplied) gives `StaleDependency`,
  `/package/source/lock/dependency_selections/0`, `ContentMismatch`. Confirmed.
- "Admits no ContractPackageV1": `Refused` is a distinct `ContractPackageReadResult` variant
  from `Admitted`, so each refusal holds by construction.

The precondition probes found two over-general statements (findings below):

- A canonical package with no `source_map` and a wrong string `contract_version` returns
  `UnknownContractVersion` at `/contract_version`, not `invalid_package`.
- The AC-14 mismatch combined with a dropped `source_map.nodes` entry returns `/source_map`.
  Combined with an empty source region, it returns
  `/source_map/nodes/0/occurrences/0/regions/0/start`.

Examined: FR-035-AC-2, AC-5, AC-10, AC-11, AC-12, AC-13, AC-14, the TC-047 procedure, Expected
Results and Status, and the tests.md FR-035 and TC-047 rows. FR-035-AC-1, AC-3, AC-4 and AC-6
to AC-9 were read for context only.

Informational, not a finding against this PR: the concurrent IR PR #480 (IR-889) also adds
FR-035-AC-12 and FR-035-AC-13, with different text. The ids collide. The lead merges #479
first and renumbers #480.

## Verdict

CHANGES REQUESTED: two medium findings. Each new criterion states the reader's real code,
path and cause. Each can fail (a trimming reader, a per-index path or a different cause would
fail them), and each class is stated by exactly one new criterion. AC-2 is not reworded. Its
TC-047 reconciliation is honest: the existing stale-case assertions are still tagged AC-5, and
the PR says so. TC-044 stays PLANNED/UNRUN. The ids are the next free ones on main. The two
findings are missing "otherwise valid" preconditions. TC-047's own procedure already uses
"otherwise valid" for AC-12, so the fix is one phrase in each criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-035-AC-12 has no "otherwise valid" precondition. It says that any canonical package omitting `source_map` returns `invalid_package`/`""`/`missing_member`. A canonical package with no `source_map` and a different string `contract_version` returns `unknown_contract_version` at `/contract_version`, because the version is checked before `validate_shapes` (contract_package.rs:944-957). A byte limit also returns `Incomplete(Bytes)` first. As written, AC-12 contradicts AC-10's outcome on that input, and a correct reader fails it. AC-10, AC-11, AC-13 and TC-047's own AC-12 step all say "otherwise valid" | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:163; crates/quire-contract-model/src/checked_package/v2/contract_package.rs:944-957 |
| FND-002 | medium | FR-035-AC-14's precondition, "canonical ... has a valid recomputed identity", does not exclude the earlier refusals. With the same first-record mismatch plus a dropped `source_map.nodes` entry, the reader returns `/source_map` (`validate_correspondence` runs before `validate_accounting`). With an empty region, it returns `/source_map/nodes/0/occurrences/0/regions/0/start`. The identity clause is also not what decides the outcome: accounting is validated before identity, and a stale identity still yields `/feature_accounting`. The criterion should say "otherwise valid canonical" (the identity may be recomputed or not) | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:165; crates/quire-contract-model/src/checked_package/v2/contract_package.rs:964-967 |
