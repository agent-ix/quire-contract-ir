---
id: SR-632
title: "spec review of PR 233 matrix edits (FR-037, FR-039)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@6a04978b0e34cb4d8e1a179a58ae52dc19fde2df; spec/contract-test-matrix.md"
review_set: base
---
# SR-632: spec review of PR 233 matrix edits

## Summary

Ticket: IR-347. Integrity review of the two edited Test Matrix status cells (FR-037 row line 40, FR-039 row line 41) against FR-037-AC-6, FR-039 and the code at head. No requirement, acceptance criterion, verification cell or matrix row is added or removed; both rows stay planned with no invented tag. FR-037's new text is accurate: `src/kani/` has no `replay` or `witness` module. `make spec` at head: validate passes, coverage reports the same 17 unbacked rows and 0 contradicted statuses as origin/main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-039 status cell now says the root crate only still re-exports the model and the codegen-owned family lowerings, but `src/kani/mod.rs:23` still exports `KaniProviderRecord` and `KaniProviderResult`, which FR-039's "Items QSL owns" section lists; the cell understates the remaining FR-039-AC-3 gap | spec/contract-test-matrix.md:41 |

## Finding Detail

- FND-001: Keep a clause for the QSL-owned items still exported, for example "...and exports the family lowerings codegen owns and the QSL-owned `KaniProviderRecord`/`KaniProviderResult`". The row stays planned, so no status is contradicted.

## Verdict

Approve with one low status-prose fix. Requirement statements and ACs are unchanged, so EARS, dependency and object sub-analyses do not apply.
