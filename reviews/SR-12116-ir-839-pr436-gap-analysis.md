---
id: SR-12116
title: "IR-839 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@09e52d9bfbb7e41b1cffeaa000ae5461061dbbe8; spec/conformance/functional/FR-018-conformance-corpus.md, spec/conformance/functional/FR-020-json-conformance-interface.md, tests/it/conformance_mismatch_independence.rs, tests/it/conformance.rs, crates/quire-contract-model/src/conformance.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: references
---

## Summary

Ticket: IR-839, PR quire-contract-ir#436, head 09e52d9. The scope is FR-018-AC-2, the FR-020 mismatch-kind registry it depends on, the runner's `mismatch_kinds` comparison, and the tests now tagged to the criterion. Plan completion: not assessed.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-018-AC-2 | examined | Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing. |
| FR-020 mismatch-kind registry | examined | Mismatch kinds are `validity`, `diagnostics`, `canonical_bytes`, `canonical_digest`, `dependencies`, and `coverage`. |
| FR-020-AC-2 | context_only | Process fixtures pin exit 1 with all six mismatch kinds in fixed order and exit 2 for each of the six closed operational codes |
| FR-018-AC-1 | context_only | Running the corpus directory yields one matching row per input with non-empty trace targets, and the union of observed tokens equals the published inventory |
| crates/quire-contract-model/src/conformance.rs:2095-2115 | examined | `mismatch_kinds` compares `valid`, `diagnostics`, the canonical bytes and digest projections, `dependencies` and `coverage` structurally |

## Method

Starting from the criterion, I listed each mutation dimension it names and matched it to a test case. I compared the runner's kind set with FR-020, then ran `quire trace --id FR-018-AC-2 --json` and `make spec`. I ran nine runner mutants in a throwaway worktree with its own target dir (since removed); the results are in SR-12115.

| AC-2 dimension | Case | Kind asserted |
| --- | --- | --- |
| schema validity | `validity` | validity |
| diagnostic code | `diagnostic-code` | diagnostics |
| diagnostic path | `diagnostic-path` | diagnostics |
| diagnostic order | `tc_018_diagnostic_order_alone_is_reported_as_a_diagnostics_mismatch` | diagnostics |
| diagnostic span | `diagnostic-span` | diagnostics |
| diagnostic obligation | `diagnostic-obligation` | diagnostics |
| canonical byte | `canonical-bytes` | canonical_bytes |
| digest | `canonical-digest` | canonical_digest |
| dependency | `dependencies` | dependencies |
| coverage row | `coverage-row` | coverage |
| coverage reason | `coverage-reason` | coverage |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

No gap-analysis findings of its own. The one test-strength defect is SR-12115 FND-001.

- Every dimension FR-018-AC-2 names has its own one-mutation case, and the six asserted kinds are exactly FR-020's registry. The runner reports no kind outside the six. A missing kind, an extra kind, a merged kind or a relabelled kind is each killed by the new tests (SR-12115 mutant table).
- Sorting diagnostics before comparing survives the old combined test and is killed only by the new order test. Before this PR, the order conjunct of AC-2 could not fail.
- One test-strength hole remains, recorded once as SR-12115 FND-001 (medium) rather than duplicated here: a runner that compares only `diagnostics[0]` survives all three tests. Every diagnostic dimension is still covered at index 0.
- "Without message parsing" holds by construction: the closed fixture schema has no `message` member.
- The tests contain no stub or tautology. Each case asserts a value the runner computes from a real process run, and the order baseline is proven by an exit-0 control before the swap.
- Traceability: three `rust-trace-attribute` claims resolve for FR-018-AC-2. `make spec` gives strict 43/0 and 508/565 rows backed. Neither changes, because AC-2 was already backed.
- Outside this PR's scope: no shipped corpus fixture produces two or more diagnostics. So the published corpus that downstream tools run never exercises the ordered-diagnostics output FR-018 promises; only this in-test constructed input does. That is a corpus-content question for FR-018 and AC-1, not a gap in AC-2's tests.
