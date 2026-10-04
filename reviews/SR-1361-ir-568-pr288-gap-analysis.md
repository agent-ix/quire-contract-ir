---
id: SR-1361
title: "PR #288 FR-020-AC-3 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@9adfb5a0259d84da4fc68471f886b56f7e4a73f6; FR-020-AC-3, TC-018; tests/it/conformance.rs, corpus/contract-v0.1/canonical (read only), spec/conformance/matrix/tests.md, spec/core/matrix/tests.md, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-018
    type: reviews
---
# SR-1361: PR #288 FR-020-AC-3 gap analysis

## Summary

Ticket: IR-568. PR agent-ix/quire-contract-ir#288. Plan completion: not assessed.

The PR adds no production code. It backs the last clause of FR-020-AC-3: "every
canonical file of the published corpus whose object holds one of the eight members
equals the expected bytes written in the test, with the eight members only as
strings". The other clauses were already backed on main by the TC-018 tests and are
unchanged.

Clause by clause, for the changed clause:

- **Every holding file equals written-out bytes.** Met.
  `tc_018_the_recorded_canonical_files_spell_the_eight_members_as_strings`
  (`#[trace("TC-018", "FR-020-AC-3", "FR-016-AC-5")]`) compares all 47 files byte for
  byte with expectations composed from fragments written in the test.
- **The 47 files are exactly the holding files.** Met. The test's set was compared
  with an independent `json.load` walk over the 78 canonical files, and the two sets
  are equal. The test also fails when a holding file has no expectation, so a new
  file cannot slip past.
- **The members only as strings.** Met. The scan of all 78 files is kept: it refuses
  a digit or `-` after any of the eight member names. The independent walk also found
  no JSON number in any of the members.
- **The published corpus runs exit 0 with every fixture a `match`.** Met. `make corpus`
  ran 107 fixtures, all `match`, exit 0, and
  `tc_018_the_corpus_runs_deterministically_and_every_fixture_matches` passes.

The status claims are now true:

- FR-020-AC-3 no longer says "only partly".
- The FR-020 row in spec/conformance/matrix/tests.md, StR-002 in
  spec/core/matrix/tests.md and the Conformance line in spec/tests.md read
  implemented.
- A grep finds IR-568 only in the historical review file SR-1157. No spec, plan or
  test file mentions it.
- `quire coverage --strict` reports FR-020 at 3/3, and it is not among the 23 unbacked
  baseline rows.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. FR-020-AC-3 is fully backed at this head, and the three matrix rows and the AC
text match the code.
