---
id: SR-12086
title: "IR-838 FR-014-AC-4 recheck test gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@313c53552ceaad178fde05ff1d9d91f99530ef42; tests/it/fr014_recheck.rs, tests/it/main.rs, spec/model/functional/FR-014-expression-semantics.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
---

## Summary

Ticket: IR-838, PR quire-contract-ir#433, head 313c535. This is a planless gap analysis, so plan completion was not assessed. Everything was measured with quire 0.36.2 (engine 0.50.2) and quoin 0.28.3.

`quire coverage --scope . --strict --format json` at head: 508/565 minted targets backed, 43 unbacked rows, 0 status lies. FR-014 is 8/9. This matches origin/main, because on main FR-014-AC-4 is already credited by three legacy doc lines.

Slice A (#432 at 5231586, open) removes those three `/// FR-014-AC-4.` lines from `tests/it/expression.rs`. Simulated by applying the #432 diff in a scratch worktree:

| Tree | Coverage | FR-014 | FR-014-AC-4 in `quire matrix --format tsv` |
| --- | --- | --- | --- |
| origin/main | 508/565 | 8/9 | tagged, 3 legacy binders |
| head 313c535 | 508/565 | 8/9 | tagged, 3 legacy binders plus `tests/it/fr014_recheck.rs:374` |
| main + #432 | 507/565 | 7/9 | untagged |
| head + #432 | 508/565 | 8/9 | tagged, only `tests/it/fr014_recheck.rs:374` |

So this test restores the row when slice A lands. The two PRs touch different files and apply cleanly together. In every tree the matrix has 531 criterion rows and only the FR-014-AC-4 row changes.

`quire trace --id FR-014-AC-4 --json` at head: four `verifies` claims. Three are `rust-doc-comment-id`, provenance `legacy` (the three tc_016 tests in `expression.rs`). One is `rust-trace-attribute`, provenance `canonical` (the new test, line 374). The `//!` module line is a `mention` citation only.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-014-AC-4 | examined | Rechecking identical declarations, context, expression, and expected type produces structurally equal typed output, ordered diagnostics, and dependency identities. |
| TC-016 | context_only | Verification column of FR-014-AC-4 names TC-016; the new test carries both ids |
| tests/it/fr014_recheck.rs | examined | `#[trace("TC-016", "FR-014-AC-4")]` on the single recheck test |
| tests/it/expression.rs legacy lines 227, 475, 956 | context_only | `/// FR-014-AC-4.` doc lines that #432 removes |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

CLEAN for gap analysis. The new binding is canonical, the row stays backed after slice A, and no other row's status changes. No production code changed, so there is no code without an owning requirement.

The one partial leg, the order part of "ordered diagnostics", is recorded once, as SR-12085 FND-001. Once that fix lands, the binding covers every clause of the AC. Until then the claim is honest on typed output, dependency identities and diagnostic content, but not on diagnostic order.

The ticket asks for a mutant that makes the second check differ, shown killed. The PR body lists three kills. Re-measured independently in SR-12085: 20 of 21 mutation kinds killed (14 of 15 in each of the repeat-input and per-call trigger modes, plus 6 environment, parity and stale-cache variants). The one survivor is the diagnostic-reorder mutant.
