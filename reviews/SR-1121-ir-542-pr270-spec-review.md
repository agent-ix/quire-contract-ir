---
id: SR-1121
title: "spec review of PR 270 (the inexact-number rule takes quire-canonical's tie-to-even spelling)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@0280d3aa38288301304c41e053f5f33f8b206224; git diff origin/main...HEAD (merge base cbcd790): FR-038 (the canonical-encoding paragraph note, FR-038-AC-109), TC-048 \"Inexact numbers\"; cross-checked against quire-spec-language origin/main 8244dd2b4a63b312ca36a8f254cdc096590eb9b9 (FR-056-AC-13 to AC-15 and the FR-056 number prose, FR-106-AC-11)"
review_set: subset
---
# SR-1121: spec review of PR 270

## Summary

Ticket: IR-542. This review checks the amended criterion's form, its
citations of QSL, and whether the rest of FR-038 still agrees with it.

- Base: the PR's merge base is cbcd790. origin/main has moved to c5fa773
  (#259). #259 changes only `spec/checked_package/matrix/tests.md` under
  `spec/`, plus code and tests. `git merge-tree` of the PR head with
  origin/main is clean. `make spec` on the merged tree gives the same result
  as on the head.
- Form: AC-109 is amended in place and is still a direct assertion with no
  "shall". No AC is added or renumbered. The FR-038 matrix row,
  `spec/checked_package/matrix/tests.md` and the TC-048 planned status are
  untouched. The TC-048 "Inexact numbers" paragraph gains the six tie cases,
  with the same expected outcomes as AC-109.
- QSL citation: 8244dd2b4a63b312ca36a8f254cdc096590eb9b9 exists. It is
  "QSL-219: refuse a model document number with no exact RFC 8785 spelling
  (#617)" and is currently the tip of quire-spec-language origin/main.
  FR-056-AC-13 is the inexact-integer criterion, and FR-056-AC-14 is the
  inexact-number criterion with exactly the six tie spellings the PR uses and
  the same outcomes. The FR-056 prose says what the PR says: the text that
  decides is the one quire-canonical writes, ties go to the even digit, and the
  texts are compared by digits and scale, never as doubles. FR-106-AC-11 is the
  authorized-change invocation clause (`post`, `result`, `created`, `deleted`).
  The PR cites these ids and the SHA and does not restate QSL's ordering
  (after FR-154 check 2), its linear-time clause or AC-15. That is right: the
  IR text asserts only what the IR reader must do.
- Invocation and snapshot: the IR reader reads no invocation or snapshot
  document. `crates/` has no such reader. `StateForm::Snapshot` is a node kind
  and `RunnerErrorCode::InvalidInvocation` is a conformance-runner CLI error,
  and neither is a document the checked-package reader admits. The note's
  "no counterpart here" holds.
- `make spec` was run in a detached throwaway worktree under
  /home/peter/dev/worktrees (since removed), on the PR head and on the head
  merged with origin/main c5fa773. `quire validate` passes on both, with 1
  grammar finding (FR-014 line 137, the baseline). Strict coverage gives
  23 unbacked rows and 0 contradicted on both, which matches the baseline.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-109 keeps its last clause: the reader crate declares `serde_json` with `float_roundtrip`, "so every decision above is the same whatever other crates in the build turn on". FR-038 prose l.602 still says "Today a model document is read as `serde_json` values". After #266, and in FR-038's own "A model document is a value inside a supplied document" section, a model document is read once with `quire_canonical::read`. That parses each number with core `str::parse::<f64>` and is never a `serde_json::Value`. The PR now says the deciding text is quire-canonical's. So none of AC-109's decisions passes through `serde_json`, and the manifest check guards nothing they depend on. The feature still matters for the package document's `serde_json` path (AC-111's admitted `0.1`). Move the clause to AC-111 or drop it, and correct l.602 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2139; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:602 |

## Verdict

Changes requested for FND-001, a one-sentence move plus a prose fix. The
amendment's substance is correct. The tie rule, the examples, the QSL
citations and the "no counterpart" note all hold when measured. The PR left
the `float_roundtrip` clause in place as "close to vestigial". With the
deciding text now named as quire-canonical's, the clause is no longer merely
vestigial. It says a `serde_json` feature secures decisions that the
specification's own reader section, and the code, make without `serde_json`.
It also leaves a test oracle that cannot fail for the reason it states.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@ea56b1196a65c2267d96ae1d4acf69945cb96339. The branch was rebased onto main c5fa773. `git range-diff` shows 0280d3a = 1b6ed14, so the patch is unchanged. ea56b11 is the fix commit. FND-002 is in text that was already present at 0280d3a, and the review pass missed it.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The FR-038 note says "so the two readers agree on every number". That is false for `1e400`/`-1e400`. QSL FR-056 (merged 8244dd2, l.86-99, AC-2, TC-145) says a document carrying "a number with no finite IEEE 754 double value (such as `1e400`)" does not parse, and refuses `stale_dependency`/`byte-digest-mismatch`, never `noncanonical_wire`. IR FR-038-AC-110 (from #265, restated by ea56b11) requires `noncanonical_wire`/`inexact-integer` with a `document_pointer` instead. QSpec FR-272's "however spelled" supports IR. QSL's "however spelled" in FR-056-AC-13 covers only parseable numbers. Either qualify the claim to finite numbers and record the `1e400` divergence as an open cross-repo question, or align AC-110 with QSL. Which behaviour wins is an owner/QSL decision, not this PR's | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:633; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2147 |
| FND-003 | low | AC-110's new parenthetical and the l.611-614 prose state an implementation constraint as part of the criterion: "the reader's scan must see the text before or around the read". That is design guidance, not an observable outcome, and it pulls against FR-038 l.557-558 ("there is one reader for the document"). The target is implementable, because `quire_canonical::read` aborts with `Malformed::NumberOutOfRange` at the literal's offset, and the pointer could come from a pre-scan or from an upstream quire-canonical change. That choice belongs to the code author. Keep the outcome and the labelled today's behaviour, and drop the mechanism, or move it to a note. Moot if FND-002 resolves by aligning with QSL | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2147; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:611 |

## New findings (disposition pass 2)

Reviewed at agent-ix/quire-contract-ir@fb44bf440cebcbd92456740f9d48b21383a6e6b8 (delta ea56b11..fb44bf4, through de3c027).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The FR-038 note states "QSL ruled otherwise (QSL ruling relayed 2026-10-03, the planner's relay …)" and spells out the ruled causes and mechanism. No checkable record of that ruling exists. QSL's merged FR-056 at 8244dd2 still says the opposite, the QSL-219 comments carry no ruling, and no QSL ticket for the FR-056 amendment or the quire-canonical change could be found. A relay is untrusted text, and a public spec should not record it as an established ruling. Nothing in IR depends on it: AC-110's outcome is IR's own (since #265) and is supported by QSpec FR-272. Cite the QSL ticket that tracks the FR-056 amendment and the quire-canonical change once it exists, or drop the ruling sentences and keep the recorded divergence. Non-blocking | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:634 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@ea56b1196a65c2267d96ae1d4acf69945cb96339, the delta 1b6ed14..ea56b11 plus the net change against main c5fa773. `make spec` was run in a detached throwaway worktree (/home/peter/dev/worktrees/ir-270-review-r1, since removed). Validate passes with 1 grammar finding (FR-014, the baseline), and strict coverage gives 23 unbacked and 0 contradicted. `git diff --check` is clean.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ea56b11: the `float_roundtrip` manifest clause and its TC-048 source-level check move to AC-111 (the package document read through `serde_json`). AC-109 now ends "the decision is made on the number's text read by `quire-canonical`, through no `serde_json` value". l.602 now reads "read once by `quire_canonical::read` … digested through `quire-canonical`". Verified at c5fa773 that `read_value` → `strict_parse` (serde_json Deserializer) → `require_canonical_bytes` is the package path, and that `admit_document` uses `quire_canonical::read`. The manifest has only `unbounded_depth` today, and AC-109..111 are marked planned in TC-048 ("no test exists until the IR-542 code change lands") and in the matrix row, which covers the manifest change |

Round 2, reviewed at agent-ix/quire-contract-ir@fb44bf440cebcbd92456740f9d48b21383a6e6b8, the delta ea56b11..fb44bf4. `make spec` was run in a detached throwaway worktree (/home/peter/dev/worktrees/ir-270-review-r2, since removed). Validate passes with 1 grammar finding (FR-014, the baseline), and strict coverage gives 23 unbacked and 0 contradicted. `git diff --check` against c5fa773 is clean.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-002 | fixed | fb44bf4: the note now reads "agree on every number with a finite IEEE 754 double". It records that QSL's merged FR-056 (AC-2, TC-145) refuses `1e400`/`-1e400` as unparseable with `byte-digest-mismatch`, which is accurate against 8244dd2. The qualification is needed while QSL's merged text stands and should be revisited when QSL's amendment merges. AC-110 keeps `1e400`/`-1e400` in the `inexact-integer` list, with `noncanonical_wire` and the `document_pointer`. Today's behaviour is labelled "(today's behaviour, not a requirement)" as `stale_dependency`/`byte-digest-mismatch`. The change from #265's "serde_json refuses at parse time" is correct: at locked quire-canonical b4bb97a, `read` parses the literal to an infinite `f64` and returns `Malformed::NumberOutOfRange`, and `admit_document` maps every read error except the byte limit to the mismatch refusal. AC-109 keeps `1e-400` as `inexact-number` |
| FND-003 | fixed | fb44bf4: AC-110 carries no mechanism. The "scan must see the text" sentence is removed from AC-110 and from the prose. The mechanism appears only in the note, labelled "a ruling, not a requirement of AC-110", pending the quire-canonical SHA (see FND-004 on its provenance) |
