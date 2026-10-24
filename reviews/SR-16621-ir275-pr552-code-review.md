---
id: SR-16621
title: "IR-275 PR 552 code review with Rust lane"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@558592b7cadd8ea9ed4fa8546240fb33465b29a5; PR #552 fifteen changed Rust and test files in checked_package and tests/it; IR-275 AC-2; FR-349-AC-4/5; FR-038-AC-2/97/117"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-349", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---
# SR-16621: IR-275 PR 552 code review with Rust lane

## Summary

Independent code review with Rust lane of the frozen PR head. The diff adds a closed WireTerm discriminator and TermField projection, ports the flat, temporal, structural, hybrid and derived-key readers, and adds exact-refusal controls. IR-275 AC-2 was checked literally against production source. The branch was not edited and the author owns the final aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Three live raw term-member lookups remain outside WireTerm, so IR-275 AC-2 is still false on this head | crates/quire-contract-model/src/checked_package/common.rs:1224-1236; crates/quire-contract-model/src/checked_package/v2/operations.rs:625-629; crates/quire-contract-model/src/checked_package/v2/timed_model.rs:66-72 |

## Finding Detail

- FND-001: The dependency-reference validator still accepts a raw Value and calls term.get("package") and term.get("node"). The type-argument operation checker reads application.node.body.get("result_type") on a RawSemanticNodeV2. The malformed timed-model binding fallback calls value.as_object()?.get("value") on a term, so the new view is bypassed for that refusal-order path. These are production reader paths, not test fixtures or metadata lookups. The literal ticket criterion forbids non-test IR or CG term reads through Value::get on string keys, and the PR claims one projection for the raw term. Route these paths through the closed projection while preserving their present refusal order and pointers.

## Scope and Evidence

- Examined all fifteen changed files and their nearby callers for admitted versus raw term interpretation, malformed input handling, recursion and work bounds. No CI workflow, copied dependency artifact, manifest or gate changed.
- IR-275 AC-2: CheckedSemanticNodeV2 defaults to CheckedTerm and CG main production source was previously audited clean; the three residual IR paths above prevent closure on this PR head.
- FR-349-AC-4 and FR-038-AC-2/97: compared staged flat-wire checking before identity and the added missing, nonstring, unknown tag and invalid interval controls. No additional concrete refusal change was found from diff inspection. The author's QSpec adverse and focused receipts remain author evidence; this review ran no full gate.
- FR-349-AC-5 and FR-038-AC-117: the 100000-node chain is across a flat graph; this change adds no depth cap, and the existing iterative walkers remain. Recursive typed conversion is reached after flat validation, which limits nested term grammar and input depth. No new concrete stack or work-bound failure was established.

## Verdict

FAIL until FND-001 is fixed and AC-2 can be re-audited on the final merged IR and CG mains.

## Dispositions

At PR #552 head `f0e1cbd01d1d218b1c4abe11700b13611a05e64b`, same-reviewer round 1 inspected the four-file delta from frozen reviewed head `558592b7cadd8ea9ed4fa8546240fb33465b29a5`. The diff is formatting-clean and adds no new raw term interpretation path. No aggregate gate was run by this reviewer.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Dependency-reference package/node now read TermField from the existing WireTerm; type-argument result_type comes from the closed CheckedTermView application arm; timed-model malformed-binding fallback reads TermField::Value from WireTerm. The only remaining non-test direct term discriminator get is the intended WireTerm constructor. |

Narrow code and Rust verdict: PASS at `f0e1cbd01d1d218b1c4abe11700b13611a05e64b`; the final gate and merged-main AC-2 audit remain separate.
