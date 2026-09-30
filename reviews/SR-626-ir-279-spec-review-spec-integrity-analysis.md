---
id: SR-626
title: "spec integrity review of PR 186 (FR-038 Reading paragraph)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@cfb1b713e5048e036b762bb297a795657758fb95; spec/contract/FR-038-consume-checked-package-v2.md"
review_set: subset
---
# SR-626: spec integrity review of PR 186

## Summary

Ticket: IR-279. This review covers the FR-038 "Reading" paragraph edited by PR 186 (spec/contract/FR-038-consume-checked-package-v2.md:138-155). It checks that paragraph against FR-038-AC-3 and FR-038-AC-26 in the same file, and against FR-322 on agent-ix/quire-specification origin/main 4634f5f. The new depth unit ("`[]` is depth 1 and `[1]` and `{"a":1}` are depth 2") is exact and matches the code and the literal-depth test. The stated order (syntax and member defects refuse before depth is charged) matches FR-322 lines 345-350. Dropping "parse strict JSON once" is consistent with FR-322, which says nothing about parse count.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new sentence "The reader admits no document deeper than 128, the default limit; a larger caller limit reads as 128" contradicts FR-038-AC-3 in the same file ("Exact ... depth ... limits admit a package") and upstream FR-322 (line 319, line 652, AC-6). A local FR narrows the upstream interface FR, and nothing upstream records that change | spec/contract/FR-038-consume-checked-package-v2.md:152-154 |
| FND-002 | low | The paragraph puts implementation mechanism into the requirement ("builds no value and runs on a stack grown onto the heap") and states "memory bounded by the byte limit", which is true only as a vacuous linear bound (SR-624 FND-002 measures about 190x). No test can fail it | spec/contract/FR-038-consume-checked-package-v2.md:146-148 |

## Finding Detail

- FND-001: Resolve it in one of two ways. Honour the caller's limit and delete the sentence. Or raise an upstream FR-322 change that allows a reader maximum and names the `limit` reported, then qualify FR-038-AC-3 to match. The prose and the AC must not disagree inside one FR.
- FND-002: State the observable property (syntax and member refusals precede depth at any depth, and the parser's own cap never decides the outcome). Leave the stack mechanism to the code comment, and either drop the memory clause or state a bound that can be tested.

## Verdict

The depth unit and the refusal order are clear improvements. FND-001 is a real contradiction and blocks merge until the owner picks between honouring the caller's limit and an upstream amendment.
