---
id: SR-1406
title: "EARS review of PR 290 (FR-043)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@0c24bb3bd1059fcf390657a17086e5b9c72a9e9e; spec/core/functional/FR-043-one-lock-entry-per-crate.md (statement and FR-043-AC-1 to FR-043-AC-4)"
review_set: subset
---
# SR-1406: EARS review of PR 290

## Summary

Ticket: IR-581. FR-043 is the one new requirement statement. Its ACs are direct
assertions in this repository's convention (a concrete fixture, an observable pass or
fail, the crate named in the output), which is fine; `quire validate` raises no grammar
warning for FR-043. The statement itself bundles three `SHALL` clauses with three
different subjects, one of which (the lock) is a state rather than a system response.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-043's statement is compound: "Cargo.lock SHALL hold ...", "make deny SHALL fail when it holds two" and "The check SHALL be run by one owned tool ..." are three requirements with three subjects; the first states a lock property, not a response. Split into an event-driven clause ("When the lock holds two entries of one first-party crate, `make deny` shall fail and name the crate") and a ubiquitous one ("The repository shall carry no copy of the check") | spec/core/functional/FR-043-one-lock-entry-per-crate.md:17 |

## Verdict

One low finding; the ACs conform to the repository's AC convention.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | adb9fc2 |
