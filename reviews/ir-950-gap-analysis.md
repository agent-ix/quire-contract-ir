---
id: SR-1396
title: "IR-950 corpus generator gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-950; FR-018-AC-1 through FR-018-AC-3; src/bin/quire-contract-corpus-generator.rs; src/bin/quire-contract-corpus-generator/cases.rs; src/bin/quire-contract-corpus-generator/ordered_json.rs; tests/it/conformance.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
---

## Summary

The new generator is owned by FR-018's explicit checked-in generator paragraph. The regeneration test binds FR-018-AC-1 and compares every generated path and byte with the checked-in corpus; no new trace or reverse ownership gap was found in the PR diff.

## Verdict

**PASS for the IR-950 delta** — FR-018-AC-1 through AC-3 are tagged. This does not assert a repository-wide gap-analysis PASS; the computed matrix still reports 24 untagged criteria outside this PR's changed behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

The computed `quire matrix --scope . --format json` reports 239 tagged, 24 untagged, and 2 method-without-symbol criteria. FR-018-AC-1, AC-2, and AC-3 are tagged. The new test's tag resolves to FR-018-AC-1 and TC-018. The source surface examined includes the generator CLI, its case declaration table, ordered JSON serialization, the runner handoff, README invocation, and the regeneration test. No new unowned behavior, source stub, test stub, or trace inflation was found. The 24 untagged criteria are outside the changed FR-018 surface and are not dispositioned by this PR review. Optional semantic review was not requested.

## Disposition-round scope update

At fe0e4973, the additional `#[trace("FR-018-AC-1", "TC-018")]` test for failed regeneration compares every existing corpus file path and byte after a missing runner. It is a correct binding of the generator's scratch preservation behavior to FR-018-AC-1. The code change derives the deep trace's canonical requirement identity from the package; it adds no new criterion or unowned public surface.
