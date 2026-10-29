---
id: SR-16611
title: "IR-275 PR 548 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #548 FR-349 AC-1 through AC-5; IR-275 AC-2 and AC-5"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-349", type: references }
---
# SR-16611: IR-275 PR 548 incremental gap analysis

## Summary

Planless, diff-scoped check of requirement tags, typed body tests, canonical identity controls and refusal/limit evidence. IR-275 AC-2 and FR-349-AC-1 through AC-5 were examined.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-349-AC-1/2: tagged tests cover admitted typed roots, nested arguments and bindings, dependency references, frame, abstraction relation and extension bodies. Changed tests assert retained variants rather than only serializing them.
- FR-349-AC-3: typed term canonical bytes are compared with the source JSON for the wire arms; admitted graph and identity preimage fixture checks preserve package ID; the producer envelope tests compare exact canonical bytes.
- FR-349-AC-4/5: tagged malformed body and refusal precedence tests remain, and a focused small-stack test admits the deepest valid typed term at the exact byte ceiling and refuses one byte below it.
- IR-275 AC-2: the remaining selected-population JSON term-key path is reported in SR-16610 rather than duplicated here. Source inspection did not identify an untagged new criterion in the changed tests.
- No plan was supplied. Plan completion: not assessed. This is an incremental PR-diff claim, not a full repository assurance verdict.

## Verdict

PASS for incremental trace/test coverage, subject to SR-16610's code findings.

## Narrow carry after final-gate fixture repair

Same-reviewer diff-scoped inspection of `..`: two TC-451 encoder fixtures now deliberately deserialize malformed bodies as raw `Value`, preserving their original byte/refusal assertions; the typed producer test still checks ordered reference targets and substitution through the new public `reference_targets` API. The TC-048 static source guard accounts for exactly one added pre-read candidate projection and retains its existing no-local-encoder checks. No new trace or acceptance gap in the changed tests. Incremental gap verdict remains PASS; plan completion remains not assessed.
