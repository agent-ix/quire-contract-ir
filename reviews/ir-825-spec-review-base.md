---
id: SR-11840
title: "Base spec review of IR-825 FR-052 status reclassification and AC-24 split (PR #421)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-052-source-obligation-correspondence.md (FR-052-AC-1..AC-24 table and status prose), spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md (TC-452 row); context: tests/it/source_obligation.rs, tests/it/public_interface.rs, crates/quire-contract-model/src/source_obligation.rs, spec/model/functional/FR-019-rust-library-interface.md:191"
review_set: subset
---

## Summary

Ticket: IR-825. PR agent-ix/quire-contract-ir#421 at e829123, diff
`origin/main...HEAD` (merges cleanly with origin/main 149e0f7; the one
intervening main commit touches FR-038 only).

AC table text was compared on origin/main and the branch. AC-1, AC-2, AC-5,
AC-6, AC-7, AC-9, AC-10 and AC-11 rows are byte-identical. AC-3 loses only the
conflicting-inventory-facts clause and gains a pointer to the new AC-24.
AC-17, AC-18 and AC-19 change only their trailing disclaimer sentence.

Per-AC judgement on synthetic failability. AC-1 and AC-5 name the neutral
public surface, a default-feature consumer and FR-019's inventory. AC-2 names
a typed fixture; its "complete IR V2 outcome" is a `Complete` outcome, which a
synthetic enumerator issues. AC-3 names census dispositions. AC-6 names census
errors for missing and `Err`-terminated outcomes; its last sentence ("No Quire
`Complete` fixture counts as owner evidence") is a disclaimer, not an
observable, and does not bar a synthetic Quire fixture from being scaffolding.
A synthetic test can fail each of these, and none names an owner enumerator
or owner scope. AC-7 (owner scopes), AC-9 (native enumerator, PLAT-1165), AC-10
(owner-reported causes) and AC-11 (sentence 2: owner adapter tests) correctly
stay Planned/owner-native. AC-11's sentence-by-sentence handling is accurate:
its first sentence is the AC-13 boundary.

The AC-2 tension between many-to-many and an unmapped function with only two
functions is real, and the TC resolves it with two assertion sets over one
inventory. That is a sound reading. No owner-issued-completeness or
four-owner credit is claimed anywhere, no status flips to Implemented, and the
PR body states plainly that the change loosens the decision and why.

Coverage was measured on origin/main in a temporary worktree (since removed):
47 unbacked, 0 contradicted. The branch has 48 unbacked (+FR-052-AC-24) and 0
contradicted. `quire validate` passes 919/919. FR-052-AC-24 does not occur in
`git log --all -S` before this branch's commit, and #421 is the only open PR.

Defects are in TC-452's plan for the code slice. Its AC-1/AC-5 doctest cannot
be traced by `quire matrix`. One test named for AC-3's exact-duplicate clause
cannot fail that clause. AC-3's same-ID conflicting-provenance clause has no
listed test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-452 backs FR-052-AC-1 and AC-5 with a default-feature doctest on `build_source_obligation_census` "traced to both criteria", but `quire matrix` binds a `Trace:` tag only to a `#[test]` fn. Reproduced in a probe worktree: a `/// Trace: FR-052-AC-1, FR-052-AC-5` line in that fn's rustdoc (before or after the code block), and a `// Trace:` line inside the doctest, both leave AC-1/AC-5 `untagged`. The same line on a non-test helper is also untagged, while on a `#[test]` fn it binds. Repo precedent (FR-030-AC-6's `compile_fail` doctests) traces the AC through a `#[test]` in tests/it. As written, the code slice cannot move AC-1/AC-5 off `untagged`. Name a traced `#[test]` external consumer: tests/it, as AC-13/AC-14 already do, or a model-crate `tests/` target run with `-p quire-contract-model` for strict default features. The doctest can remain as extra evidence. | spec/model/matrix/TC-452-source-obligation-correspondence.md:182-197 |
| FND-002 | medium | TC-452 lists `assertion_conflicts_and_relation_dedup_are_deterministic` for AC-3's exact-duplicate-assertion clause, but that test cannot fail the clause. Its inputs `a, b, a` share one relation and one provenance, so with step 1 (exact-duplicate collapse) removed, step 4's relation dedup still yields the asserted single row. Only `rejected_assertions_collapse_duplicates_and_sort_with_reason_precedence`, where the repeated dangling `a` must appear once in `rejected_assertions()`, observes the collapse. Drop the first test from that bullet. | spec/model/matrix/TC-452-source-obligation-correspondence.md:236-237; tests/it/source_obligation.rs:854-858 |
| FND-003 | medium | AC-3's "conflicting assertion provenance" mirrors FR-052's "Repeated assertion identity with conflicting ... provenance shall remain inspectable with `ConflictingProvenance`" (step 2, provenance branch). TC-452 maps it only to `assertion_conflicts_and_relation_dedup_are_deterministic`, whose `a`/`b` are distinct IDs on one relation (step 4, AC-22). No existing test has one `SourceAssertionId` whose members share a relation and differ only in provenance. `assertion_reason_precedence_...` differs in both, so it yields `ConflictingRelation`. Add this as a third new assertion before tracing AC-3: one assertion ID, same relation, two provenances, both members rejected `ConflictingProvenance`, and no accepted row. | spec/model/matrix/TC-452-source-obligation-correspondence.md:240-251; spec/model/functional/FR-052-source-obligation-correspondence.md:312-316 |
| FND-004 | low | TC-452's existing Test Procedure was not reconciled with the reclassification. Line 40 frames the four inputs as "native APIs or typed adapters". Lines 127-143 then exercise the AC-6 omissions and `Err` stops and the AC-2/AC-3 fixture inside that flow, and they still list "conflicting inventory facts", now AC-24. For `quire-code-rs` they say "a selected module parse failure", while the new section uses `ModuleUnavailable`. A reader cannot tell which procedure backs AC-2, AC-3 and AC-6. | spec/model/matrix/TC-452-source-obligation-correspondence.md:40,127-143 |
| FND-005 | low | FR-019's `source_obligation` inventory row lists the FR-052 criteria the exports serve: AC-4, AC-12..15 and AC-17..23. Once AC-1/AC-5 are backed, that row will be stale. TC-452's code-slice list does not tell the slice to add FR-052-AC-1 and AC-5 there. tc_058 compares item names only, so nothing would catch it. | spec/model/functional/FR-019-rust-library-interface.md:191; tests/it/public_interface.rs:189-208 |

## Verdict

Not mergeable as is. The status reclassification itself meets the planner's
conditions. The AC table is unchanged except for the AC-3/AC-24 split and the
AC-17/18/19 disclaimers. The disclaimers now say exactly what they do and do
not back: AC-17 not by itself AC-2, AC-18 not by itself AC-6 and no AC-7
owner-native credit, AC-19 not by itself AC-3. No Implemented flips occur.

Fix FND-001 to FND-003 before merge, because TC-452 is the binding plan for
the code slice. Under that plan, AC-1/AC-5 would stay untagged, and AC-3 would
be traced to a test that cannot fail one of its clauses while missing another
clause entirely. FND-004 and FND-005 are small consistency fixes in the same
PR. See SR-11841 for the AC-24 split.
