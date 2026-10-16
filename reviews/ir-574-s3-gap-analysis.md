---
id: SR-11791
title: "IR-574 Slice 3 probe and coverage corpus bounds gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #414 diff origin/main...6741c65: tests/it/conformance.rs, spec/conformance/matrix/tests.md, spec/core/matrix/tests.md; read against FR-011-AC-3, FR-012-AC-6, FR-020-AC-3, FR-023 prose, spec/tests.md"
review_set: subset
---

# IR-574 Slice 3 probe and coverage corpus bounds gap analysis

## Summary

Ticket: IR-574. PR: quire-contract-ir#414 at 6741c658514c97043e489c5b74775a4ba04b886a. This is a planless gap analysis, scoped to the PR diff and the criteria it claims. Plan completion: not assessed.

The PR adds one test and makes no production or schema change. The obligation it backs is that probe and coverage corpus inputs refuse an over-2^53 revision or byte offset as `invalid_corpus` at the fixture schema, before any typed decoding. That obligation is stated in two places:

- FR-012-AC-6, for byte offsets, with a pointer to FR-020-AC-3;
- the FR-023 prose ("The same stage refuses the same value in the probe and coverage inputs of the conformance corpus (FR-020-AC-3) ...").

The computed matrix (`quoin matrix --json`, quoin 0.28.3) binds the new test to FR-011-AC-3, FR-012-AC-6 and FR-020-AC-3, all with static status `tagged` and no run evidence. That is the expected state for a tagged test that has not been run under evidence capture.

Test-to-code agreement was measured by mutation in a throwaway worktree. Mutations that raised, removed, lowered, retyped or loosened the bound for a single member were all killed (details in SR-11790). The schema bound is what decides each refused case, and the test is not passing on some unrelated schema mismatch.

## Verdict

**PASS for this PR, mergeable.** The PR does not introduce either finding. Both are in the spec text and summary outside the diff, and should be deferred to a spec follow-up rather than block this tests-only slice.

- **Backed.** FR-012-AC-6's probe and coverage clause is backed for offsets (`source.start.byte_offset` and `target_span.end.byte_offset`). The FR-023 prose obligation is backed for revisions and offsets: the probe clause-reference revision and the trace source, target-span and target revisions.
- **The FR-011-AC-3 and FR-012-AC-6 tags.** The direct decode half of the test (ClauseRef, RequirementRef, SourceSpan with serde) does exercise those ACs' registered codes. TC-015's existing tests already cover the named decoders, so this adds evidence without overclaiming.
- **The matrix flips match the test.** FR-020 is the only row whose status claim grows, and FR-012 gains TC-018 while staying 🚧.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The probe and coverage revision half of the obligation has no owning AC statement. FR-020-AC-3, which the test and the FR-020 matrix row cite, speaks of "the package schema" and decoder fixtures and never names probe or coverage inputs. FR-011-AC-3, unlike FR-012-AC-6, has no clause routing an over-2^53 revision in probe or coverage inputs to `invalid_corpus` at the fixture schema. The four revision rows of `tc_018_probe_and_coverage_bounds_fail_at_the_fixture_schema` are traced to FR-020-AC-3, but they back an obligation stated only in FR-023 prose. If a future edit dropped the bound for those members, it would not contradict any AC text. This is a pre-existing spec gap and is not introduced by this PR. | spec/conformance/functional/FR-020-json-conformance-interface.md:136; spec/core/functional/FR-011-package-identity.md:58; spec/model/functional/FR-023-executable-projection-binding.md:77-79 |
| FND-002 | low | The `spec/tests.md` summary still calls FR-023-AC-6 planned. The Core row says "Whole projections meet the published schema first (`invalid_wire_format` at `projection`, planned FR-023-AC-6)", and the Model row says "FR-023-AC-6 is planned (IR-574)". The core matrix FR-011 and FR-012 rows that this PR touches say "implemented in TC-035" (#412). This is pre-existing and not touched by this PR, but it is IR-574 matrix bookkeeping that a reader of the summary trips on. | spec/tests.md:12-13 |

## Examined scope

```yaml
scope:
- id: FR-020-AC-3
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: "the package schema accepts a revision and a byte offset of `9007199254740992` and rejects `9007199254740993`; a decoder fixture that supplies a JSON number in one of the eight members straight to the decoder returns `invalid_wire_format` and its expectation carries that code, while the same number in a valid-operation input is `invalid_corpus` at the schema check"
- id: FR-012-AC-6
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: "Source spans reject ... a byte offset above 9007199254740992 ... using `invalid_source_span` ... and the probe and coverage inputs of the conformance corpus refuse it `invalid_corpus` at the fixture schema (FR-020-AC-3)."
- id: FR-011-AC-3
  path: spec/core/functional/FR-011-package-identity.md
  role: examined
  excerpt: "zero source or requirement revisions and a revision above 9007199254740992 (`invalid_source_revision`, `invalid_requirement_revision`, through the constructors, the package decoder, the expression operation decoder and the binding expression decoder; a whole executable projection is FR-023-AC-6's)"
- id: FR-023
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: context_only
  excerpt: "The same stage refuses the same value in the probe and coverage inputs of the conformance corpus (FR-020-AC-3), where it is an `invalid_corpus` failure of the run and never reaches a decoder."
- id: tc_018_probe_and_coverage_bounds_fail_at_the_fixture_schema
  path: tests/it/conformance.rs
  role: examined
  excerpt: "six members x [2^53+1, u64::MAX] -> invalid_corpus at fixtures.<id>.input, detail 'instance does not match schema'; direct ClauseRef/RequirementRef/SourceSpan decode -> registered code; 2^53 passes the schema stage"
- id: spec/tests.md#Core-Model
  path: spec/tests.md
  role: examined
  excerpt: "Whole projections meet the published schema first (`invalid_wire_format` at `projection`, planned FR-023-AC-6); ... FR-023-AC-6 is planned (IR-574)."
```
