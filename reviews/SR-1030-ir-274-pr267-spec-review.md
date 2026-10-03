---
id: SR-1030
title: "spec review of PR 267: a byte-limit failed record names its limit kind and values (IR-274, planner IR-546)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@c4f2a24706520a10bbaeb165ae9eef1a14624c3d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Canonical encoding of the wire types; Every identity digest is computed through quire-canonical; AC-91, AC-95; Dependencies), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/assurance/AD-005-qsl-consumption-seam.md, read against FR-035, AD-004, spec/checked_package/matrix/tests.md and schemas/"
review_set: subset
---
# SR-1030: spec review of PR 267

## Summary

Ticket: IR-274 (planner ticket IR-546). The PR answers SR-1016 FND-002 (what a
byte-ceiling `failed` record carries) and FND-003 (stale prose about
quire-canonical #7). Diff: `git diff origin/main...HEAD`, base ebea678, three
files.

What I measured:

- `CheckedPackageLimit` exists in `checked_package/shared.rs` and is closed,
  with `Bytes` and `Work` variants. `CheckedPackageIncomplete` uses it.
  `CompleteLoweringRecordV2::Failed` is `{node_id, limit, consumed}`, with no
  kind field. That holds on main and on PR 266's head 476e9b4. So "planned"
  is true.
- The record type is not `Serialize`. No schema or corpus file under
  `schemas/` or `corpus/` describes lowering records, and FR-016, FR-020 and
  FR-034 do not give their field shape. AD-004 only lists the record kinds. So
  no schema or corpus file needs the same change. This is a Rust model change,
  not a wire change.
- quire-canonical #7 is MERGED as b4bb97a5fe0a946e9d980e6466c7ecf95c6e62f1,
  which is origin/main's head. `LimitExceeded.required` exists and is
  documented "Always greater than `bound`". `Writer::count` enforces that for
  `CanonicalBytes`.
- The FR-038 matrix row in `tests.md` still says "(quire-canonical #7) is open
  and pending merge ... code change A is blocked on it". The author is right
  that the row belongs to PR 266. At 476e9b4, though, PR 266 has not fixed it
  yet: SR-1016 FND-001 is still open there. PR 267 and PR 266 merge without
  conflict (`git merge-tree`).
- AC-91 and AC-95 state outcomes directly. Numbering is unchanged (AC-89 to
  AC-95). The TC-048 procedure matches AC-95. The `tests.md` counts are
  unchanged and still true: AC-89 to AC-95 are 🚧 planned.
- `make spec` was run in a detached throwaway worktree under
  /home/peter/dev/worktrees, since removed. Results: validate exit 0, one
  grammar finding (FR-014 `ac:vague-response`), and `--strict` 23 unbacked
  with 0 contradicted. That is the baseline.
- `git diff --check` is clean. No line outside the three intended edits
  changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Stale #7-pending prose is still in FR-038 after the fix. "Canonical encoding of the wire types" says "a `Value` member to the encode of a `Value` (this repository's walker until `quire-canonical`'s `Encode` for `serde_json::Value` lands, then that `Encode`)". It also says "Once the upstream `Encode` lands, this repository holds no walker ... Until then the walker is the one encoder of a `Value` and FR-038-AC-77 and FR-038-AC-78 are verified against it". Dependencies says "including its pending `Encode` for `serde_json::Value`". All three contradict the amended sentence 50 lines earlier: "this repository has no walker of its own: `value_to_vec` ... is deleted". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:389-396, 1608-1609 |
| FND-002 | medium | Decision (b) breaks sibling independence, and the spec never says so where that rule is stated. When the package is over the ceiling, every requested record becomes `failed` for `bytes`, so a request that would lower on its own flips because of its siblings. FR-035 Behavior says the lowerer "shall not ... affect a sibling disposition". FR-035-AC-3 and FR-038-AC-6 say sibling records are independent and unchanged. The new paragraph restates the package-wide failure as normative ("no requested record is left `lowered`") but gives no exception in FR-035 or FR-038-AC-6. State the carve-out there, or in FR-038 with a cross-reference. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:466-470; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:64-66 |
| FND-003 | low | The new paragraph was inserted mid-paragraph. It ends "...`consumed` 0." and runs straight into "`NominalIdentityPreimage::digest`, which a caller may call ... (FR-038-AC-90)" with no blank line. That sentence belonged to the previous paragraph about encodes under a ceiling, so it now reads as part of the failed-record paragraph. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:473-479 |
| FND-004 | low | Two statements are true only if PR 266 merges first. FR-038 says "Planned: the record today ... reports a package over the ceiling as `failed` with the work limit and `consumed` 0". AD-005:131 says "IR's `value_to_vec` is deleted". Both describe PR 266's code. On main (ebea678) the package encode is a serde_json `.expect` with no ceiling, and `value_to_vec` exists. Merge PR 266 first, or drop "today". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:473-474; spec/assurance/AD-005-qsl-consumption-seam.md:131 |

## Verdict

Changes requested, for FND-001 and FND-002. Decisions (a), (b) and (d) are
sound and well-formed:

- (a) reuses the existing closed `CheckedPackageLimit`.
- The work and bytes rule "consumed > limit" holds against the code: `failed`
  fires only at `work > work_limit`, and `required > bound`.
- No schema or corpus file needs the change.
- AC-95 is amended consistently with the prose and TC-048.

FND-001 is the item 2 sweep the PR set out to finish but did not.
Decision (c) is examined in SR-1031.

## New findings (disposition pass 1)

Reviewed at ec21b9c00b9e3838115e19dc8e35eb6ee4313a33. Delta checked:
c4f2a24..ec21b9c.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The exception's stated reason is wrong. FR-035 Behavior says the package "has no siblings", and FR-038 says "a package that could not be produced has no siblings". The records are still siblings. What the exception does is override their dispositions, because no package exists to carry any `lowered` node. Reword both, for example: "every requested record is `failed`, because no package exists to carry a lowered node". | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:66-69; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:484-485 |
| FND-006 | low | FR prose now carries matrix status: "FR-038-AC-95 is planned and has no test yet". PR 266 (249acbc) adds the AC-95 tests and updates only the matrix, so this sentence goes stale as soon as PR 266 merges. Status belongs in `tests.md`. Drop the sentence. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:492 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ec21b9c. I re-grepped `spec/` for pending, until, lands, walker, value_to_vec and "blocked on". The only #7-pending text left is the FR-038 row in `tests.md` line 15, which PR 266 owns, and PR 266 at 249acbc rewrites it ("merged as b4bb97a5 ... deleted from `encode.rs`"). The two PRs merge without conflict. |
| FND-002 | fixed | ec21b9c. The exception is now stated in FR-035 Behavior, FR-035-AC-3 and FR-038-AC-6, each pointing to FR-038-AC-95. FR-038 keeps per-node independence explicitly (`invalid_input`, `unsupported`, `requires_bound`, and a work or byte `failed` of one node). FR-035's matrix row stays ✅ truthfully, because the exception only narrows AC-3. The reason given in the wording is FND-005. |
| FND-003 | fixed | ec21b9c. The failed-record paragraph now follows the AC-90 sentence, which stands alone as its own paragraph. |
| FND-004 | fixed | ec21b9c. "today" is gone. AD-005 now reads "IR holds no walker of its own (FR-038-AC-91)", a target statement. |

## Dispositions (round 2)

Reviewed at ac907bbe858ba81730ae3e214e0f66988e159420 (delta ec21b9c..ac907bb).
No new findings this round.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | ac907bb. FR-035 now says "every requested record is `failed`, because no package exists to carry a lowered node". FR-038 says the same, and adds "which overrides the disposition each record would have had alone". A grep for "has no siblings" across spec/ finds nothing. |
| FND-006 | fixed | ac907bb. The sentence "FR-038-AC-95 is planned and has no test yet" is removed. A grep for "no test yet" in FR-038 finds nothing. |
