---
id: SR-11800
title: IR-816 base spec review of receiver-scoped frame entries and deletes_scoped (FR-040-AC-17..29)
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir@9526be4cce3d637705b22ec4804705ff718dcb3a (PR 415, diff vs origin/main 32e433a); spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-053-checked-package-v2-frame-bodies.md, spec/checked_package/matrix/tests.md, spec/spec.md, spec/tests.md
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-816. Base checklist over the seven-file spec diff of PR 415 at 9526be4: FR-040's four-member frame body, field-entry `scope`, `deletes_scoped`, ordering, `conflicting-binding` rank and the state-step scoped-deletes check; FR-038's frame text; TC-056/TC-053; matrices. Each rule was checked against merged QSpec FR-340/FR-342 (quire-specification origin/main 9a1d3cf), QSL TC-1043 (quire-spec-language origin/main 24fa7b4a3) and the current reader (`v2/frame.rs`, `tests/it/checked_package_v2_frame_entries.rs`). Review only; no edits.

## Verdict

**NOT MERGEABLE**: one high and one medium finding, plus one low.

What holds:

- Each FR-040 rule matches merged QSpec FR-340: exactly four members (`modifies`, `creates`, `deletes`, `deletes_scoped`, all required, each possibly empty); `scope` exactly `receiver`|`every-object` on field entries only; `deletes_scoped` shapes `{kind: receiver}` / `{kind: parameter, name}`, receiver first then parameters by UTF-8 name bytes, no duplicate; order key declaration digest, name, scope (`every-object` first), relationship entries with empty name and no scope; `conflicting-binding` after every other meaning-join defect and before canonical order; first-defect order schema, stale key, frame step, state step; check 5 at the frame node per FR-342.
- Codes and causes used (`invalid_semantic_graph`, `invalid_model_binding`/`conflicting-binding`, `missing_declaration`/`missing-name`, `invalid_package`/`stale-node-key`, `ambiguous_declaration`/`ambiguous-name`) all already exist in the reader's taxonomy (`CheckedPackageRefusalCause::ConflictingBinding`, shared.rs:205; used by FR-346 and FR-038). No code is invented. (STD-001 is IR's own registry and does not carry checked-package codes; the authoritative pairing is QSpec FR-322 as stated in FR-038.)
- The AC-7 change (shared frame admitted) is justified by merged QSpec FR-342 ("Two anchors of one `context` that reference one `frame` node are admitted") and FR-342-AC-6, and by QSL TC-1043 step 2b. It breaks no other IR criterion's text.
- No fallback or compatibility reader for the old three-member body is specified; a body without `deletes_scoped` refuses (AC-19).
- AC-17..29 are free: `git log --all -S FR-040-AC-17` finds only 9526be4; open PRs #413 and #414 touch no FR-040 ids. No renumbering.
- TC-056 cites TC-1043 rows by id (W1-W6, X1-X9) and re-authors its cases; no wire bytes are copied.
- Gates measured: `quire validate --scope . "spec/**/*.md"` exits 0 with only module-level warnings. `quire coverage --strict` is 49 unbacked / 0 contradicted at both origin/main 32e433a (temporary worktree, removed) and 9526be4; FR-040 moves 16/16 to 16/29 rows. `quire matrix` differs only by the thirteen new untagged FR-040 rows.
- Outside IR, noted not a finding: QSL TC-1043 itself states that every row using a scoped member (W1, W2, W4, W6, I1-I17, X1-X9) depends on the filament-core-data semantic-IR frame schema admitting `modifies_receiver`/`deletes_receiver`/`deletes_parameters`; only step 2b is reachable today. IR's TC-056 re-authors its packages in-repo, so the IR code slice is not blocked by that, but a QSL-emitted oracle package for those rows does not exist yet.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-040 says the `conflicting-binding` refusal's "message names both entries and their scopes", "as for FR-346's conflicting bindings", and AC-23 asserts "both entries named". FR-346 (:246-253) states the opposite IR reading: the second entry's path only, "the first entry is not carried", because carrying it "needs a refusal member this repository does not have". `CheckedPackageRefusal` (shared.rs:346-375) has no message member and no Display, so no test can observe "both entries named". AC-23's last clause is unsatisfiable without inventing a refusal member, and the PR body repeats the wrong FR-346 attribution. Fix: drop the message claim from FR-040 and AC-23, say the first entry is not carried, as FR-346 does, and record carrying both as the same QSpec question. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:164-168, 322 |
| FND-002 | medium | AC-7's new clause "two anchors of one `context` that reference one `frame` node are both admitted" carries no PLANNED marker. TC-056 Status (:107) and the TC-056 matrix row (tests.md:41) still say AC-1..AC-13 are implemented. The test tagged FR-040-AC-7 (tests/it/checked_package_v2_frame_entries.rs:911-948, case "a shared frame") asserts `ambiguous_declaration`/`ambiguous-name`, the refusal the new text forbids. Coverage stays at 0 contradicted only because quire cannot see the semantic contradiction. AC-28 already states the same admission as PLANNED. Fix: remove the shared-frame clause from AC-7 and keep only the duplicate-pair refusal there, so AC-28 owns the admission and AC-7 stays true and implemented. Alternatively, mark AC-7 PARTIAL/PLANNED in its own row text. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:306 |
| FND-003 | low | The TC-056 row of the checked_package matrix, which this PR edits, still reads "AC-14 through AC-16 are PLANNED/UNRUN (IR-770), with no tagged implementation tests yet". The FR-040 row of the same file, TC-056 Status and spec/tests.md all say AC-14..16 are implemented (IR-770). Correct the stale clause while the cell is being edited. | spec/checked_package/matrix/tests.md:41 |

## Examined units

```yaml
scope:
- id: FR-040-AC-7
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: '... A duplicate (`context`, `operation`) pair refuses as `ambiguous_declaration`/`ambiguous-name` at the second anchor in ascending node-id digest order; two anchors of one `context` that reference one `frame` node are both admitted.'
- id: FR-040-AC-17
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits.
- id: FR-040-AC-18
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `field` entry with no `scope`, a `field` entry with a `scope` other than `receiver` or `every-object`, and a `relationship` entry carrying `scope` each refuse `invalid_semantic_graph` with no cause, located at that entry, before the frame step.
- id: FR-040-AC-19
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame body with no `deletes_scoped` member refuses `invalid_semantic_graph` with no cause, located at the frame node's `body`, before the frame step.
- id: FR-040-AC-20
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `deletes_scoped` entry of a `kind` other than `receiver` or `parameter`, a `parameter` entry with no `name`, an entry carrying a further member, and an array repeating an entry each refuse `invalid_semantic_graph` with no cause, located at that entry or member, before the frame step.
- id: FR-040-AC-21
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `deletes_scoped` that does not hold the `receiver` entry first and the `parameter` entries ascending by name as UTF-8 bytes refuses `invalid_semantic_graph` with no cause, located at the frame node; `receiver`, `parameter a`, `parameter b` admits and `parameter a`, `receiver`, `parameter b` refuses.
- id: FR-040-AC-22
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). The `modifies` entries of one declaring node ascend by name, then by scope with `every-object` first; entries (`step`, `receiver`) then (`value`, `every-object`) admit and the reverse order refuses `invalid_semantic_graph` with no cause, located at the frame node.
- id: FR-040-AC-23
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). Two `field` entries sharing a declaration and name and differing in scope refuse `invalid_model_binding`/`conflicting-binding`, with the later entry in array order as the path, the declaring node key as the locus and both entries named, whichever order they are listed in.
- id: FR-040-AC-24
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame holding a `conflicting-binding` defect and a canonical-order defect reports the `conflicting-binding` and not the `invalid_semantic_graph` of the misordered array.
- id: FR-040-AC-25
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame holding a `conflicting-binding` defect and any other meaning-join defect reports the other meaning-join defect.
- id: FR-040-AC-26
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `scope` or a `deletes_scoped` entry changed in a frame node left under its old key refuses `invalid_package`/`stale-node-key` before the frame step.
- id: FR-040-AC-27
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `parameter` entry of a frame's `deletes_scoped` naming no reference parameter of the operation an anchor referencing the frame resolves refuses `missing_declaration`/`missing-name`, located at the frame node, in the state step after that anchor's own joins hold, and not in the frame step.
- id: FR-040-AC-28
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). Two anchors of one `context` referencing one `frame` node both admit, and the frame's `parameter` entries are checked against each anchor's operation, the first failure in ascending anchor node-id digest order being reported.
- id: FR-040-AC-29
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A package carrying a frame-step defect and a `parameter` entry that fails the state step's scoped-deletes check reports the frame-step defect.
- id: FR-040-AC-1
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `modifies` array holding a `relationship` entry and a `field` entry, each naming an eligible declared dependency, admits; an all-empty frame body of four empty members admits; a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` with no cause, located at that entry.
```
