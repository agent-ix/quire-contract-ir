---
id: SR-16610
title: "IR-275 PR 548 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@797923c6d3f3d4a2b483363c3ca94b7959167134; PR #548 typed V2 body, producer codec, candidate APIs and tests"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-349", type: references }
---
# SR-16610: IR-275 PR 548 code and Rust review

## Summary

Independent diff-scoped review of the exact draft head against IR main `0dfdea6`. Examined the typed term variants, raw admission boundary, producer canonical encoder, candidate identity helpers, all changed reader call sites and targeted tests. No author edits. IR-275 AC-2 and FR-349-AC-1 through AC-5 were examined.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The selected-population candidate path converts each term back to `Value` and selects `members` and `target` with string keys, leaving the IR-275 AC-2 migration incomplete and making this path's behavior depend on JSON serialization. | `crates/quire-contract-model/src/checked_package/selected_population.rs:441` |
| FND-002 | low | The typed `ApplicationIdentityContext` retains a full raw graph copy, contradicting its public constant-storage promise. | `crates/quire-contract-model/src/checked_package/v2/application_identity.rs:43` |

## Coverage

- FR-349-AC-1/2: the admitted `CheckedSemanticNodeV2.body` is `CheckedTerm`; `decode_admitted` follows flat validation and covers literal, reference, application, aggregate, binding, dependency reference, frame, abstraction relation and extension form. Nested variants and operator/literal payloads preserve admitted shape. The changed reader call sites generally match typed terms. `population_object_type` remains a raw candidate route used by `operations.rs:2471`; it serializes a generic body twice and reads `members`/`target` with `Value::get`, contrary to IR-275 AC-2's no term-key-read criterion.
- FR-349-AC-3: graph and borrowed identity projection use the same generic `Body`; `Encode` writes typed body members directly. Existing fixture checks compare canonical package bytes and digest inputs. The raw wire duplicate identity body is outside this PR's wire change.
- FR-349-AC-4/5: flat and body validation precede typed construction; focused refusal and exact byte-limit tests exercise that ordering. Typed conversion occurs only after `validate` and drops the raw body iteratively.
- Candidate identity API: `CandidateBody for CheckedTerm` converts all nodes to raw `Value` before `ApplicationIdentityContext` stores them in an owned `Cow`; the documented retained storage is therefore linear in graph size for typed callers. The byte limit applies later to canonical output, not this copy.
- Author reports focused model/integration checks passing. This reviewer did not run the final aggregate gate, which is reserved for the reviewed head.

## Verdict

CONDITIONAL. Fix the two findings in this PR and request a same-reviewer disposition before the final gate.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At `dbeef5e1dc67cd3fc8b357d928608c11f34f111a`, the raw candidate body decodes as `CheckedTerm` and selects `Aggregate` then `Reference` variants. The function no longer serializes an admitted typed body or reads term keys through `Value::get`; the existing raw pre-admission boundary remains explicit. |
| FND-002 | fixed | The public context and constructor docs now distinguish borrowed raw drafts from owned linear-size typed projections, matching the `Cow` implementation. No storage behavior was changed or falsely promised. |

The same-reviewer disposition examined only the two-file fix delta from `797923c6d3f3d4a2b483363c3ca94b7959167134`. The author reports 14 focused reaches-population tests plus the typed-term unit tests, Clippy, format and diff checks passing; this review did not run a full gate. Current verdict: PASS for the reviewed PR diff, pending its one final-head gate.

## Narrow carry after final-gate fixture repair

Reviewed `5131cf5c..879d5b149c85694d5d9a052f23fe90d5a38825a4` as the same reviewer. The changed `model_members` and `scalar_operands` helpers now dispatch raw `Value` and admitted `CheckedTerm` directly; they no longer serialize typed bodies to JSON to read them. `CheckedTerm::reference_targets` returns an ordered `Vec<&CheckedNodeId>` and reuses the unchanged private traversal, so it preserves the visited identities and excludes type IDs as documented. The typed and raw branches of the new private helper traits are closed and explicit.

The TC-048 source guard's `v2/mod.rs` allowance rises from two to three because `CheckedSemanticNodeV2::try_to_raw` contains one producer candidate projection; the other two calls remain the reader's lock/preimage first-difference diagnostics. `intake.rs` remains zero and `identity.rs` remains one. The projection feeds the pre-read candidate algorithm; the canonical digest still uses `quire-canonical`. The two TC-451 fixture edits select the raw generic body for malformed-body encoder controls, leaving typed positive controls intact. `git diff --check` passed on this exact delta. Author-reported four prior failing integration cases, helper unit, scoped Clippy and format checks passed; the final full gate remains the author's next step. No new finding. Prior FND-001 and FND-002 remain fixed. Review verdict: PASS at `879d5b149c85694d5d9a052f23fe90d5a38825a4`.
