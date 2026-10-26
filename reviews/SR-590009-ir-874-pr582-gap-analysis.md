---
id: SR-590009
title: "IR-874 PR 582 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #582 frozen head 2b1dbad0d248ee68efc77562da22ffbc88906f4f against 4b407abc"
review_set: subset
---

## Summary

The changed public reader tests trace FR-038-AC-290 for receive/send, parallel independence, zero-progress repeat and a self-anchored await. Existing repeat admission tests trace AC-288 only. Distinct causal edge kinds and the positive progress variations of AC-290 lack matching tagged assertions.

## Verdict

**CONDITIONAL** — the changed feature has a meaningful public reader evidence gap.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-290 tests do not exercise an effect-before-attempt cycle, a compensation-forward-effect anchor, an await anchored at a later sibling, or positive progress with and without a repeat maximum and loop proof. Existing repeat admission tests assert the positive variants but tag only AC-288. | tests/it/checked_package_v2_protocol_body.rs:3522 |

## Scope

- **FR-038-AC-290 (examined)**: For each clause the reader constructs the QSpec FR-464 derived causal relation from control containment, sequencing, receive/send, effect/attempt and await edges, not source order or timestamps. A receive before its send or an await anchored at a later control refuses `invalid_package`/`definition-cycle` naming the cycle; a same pair on independent parallel branches admits. A repeat whose body can complete with no progress on a path refuses `unsupported_construct`/`declaration-form`, while a repeat with a real step admits with or without a maximum and loop proof.
- **FR-464-AC-9 (examined, QSpec)**: The derived causal relation is acyclic: a `receive` sequenced before the `send` it names, a send/receive pair in one sequence in the wrong order and an `await` anchored at a later control each refuse `invalid_package`/`definition-cycle` naming the controls on the cycle; the same pair on different branches of a `parallel` is admitted; a `repeat` back edge is not a cycle.
- **FR-038-AC-288 (context_only)**: Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks.
- **FR-038-AC-284 (context_only)**: The protocol step reports the first defect by QSpec FR-464 order.

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format tsv` completed: FR-038-AC-290 is tagged by three public-reader tests at lines 3525, 4840 and 4883. The tags do not establish the missing positive repeat and cross-edge cases. No repository-wide assurance claim is made from this scoped PR review. The changed production code has FR-038-AC-290 as owner and no new stub. Optional repository-wide semantic review was not requested; this review compared the changed public-reader assertions to the criterion. Host backoff barred Cargo, Rustc, Make and test jobs; no tests were started.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The updated TC-226 whole-read boundary checks `limit` and `consumed` at the new exact `M-1` but no longer asserts its first-unpaid-value pointer. Its retained pointer assertion now covers only the earlier Tree boundary, leaving FR-038-AC-149's whole-read `M-1` pointer obligation unverified. | tests/it/checked_package_v2_structural_keys.rs:2240 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 8fabf73f, AC-290 tags now cover existing bounded and unbounded repeat admissions and compensation-anchor admission; an added unbounded-with-proof repeat admits. Public tests newly assert effect-before-attempt and compensation-forward-effect causal cycles, and a later-sibling await cycle that wins over a conflicting guard. |
| FND-002 | still-open | At f683d452, TC-226's `read_at(M - 1)` outcome checks `limit_kind`, `limit` and `consumed` but does not assert `incomplete.path`, although FR-038-AC-149 requires the first unpaid value's pointer at this exact whole-read boundary. |

## Disposition verdict

**PASS at 8fabf73f** for the recorded gap-analysis finding. Test execution remains pending hosted Rust Checks because host backoff barred new local jobs.

**CONDITIONAL at f683d452** — FND-001 remains fixed. New FND-002 is open on the TC-226 exact-boundary pointer assertion.
