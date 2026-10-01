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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The fix round restored "parse strict JSON once", but the reader now makes two passes over the text: the iterative syntax, duplicate and depth scan, which re-reads each scalar and member name with serde_json, and then the value parse. As written, the requirement no longer describes the code | spec/contract/FR-038-consume-checked-package-v2.md:140-141 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f1d923 |
| FND-002 | fixed | 7f1d923 |

- FND-001: The "reads as 128" sentence is gone. The paragraph now says a document within the caller's limit "is read whatever that limit is", which agrees with FR-038-AC-3 and FR-322-AC-6.
- FND-002: The stack-mechanism and memory-bound clauses are removed. The paragraph states only observable behaviour.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | The Reading paragraph now states "a caller limit above that reads as 16,384". FR-038-AC-3 in the same file still says, unqualified, "Exact ... depth ... limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter", and that is false for any caller limit above 16,384. Upstream FR-322 on agent-ix/quire-specification origin/main 4634f5f also says "`read` receives limits from its caller" (line 319), "Exact selected resource limits are admitted" (line 652) and AC-6 "Exact selected limits admit the boundary vector", with no reader maximum | spec/contract/FR-038-consume-checked-package-v2.md:149-151 |

### New finding detail (disposition pass 2)

- FND-004: The prose describes the code accurately. AC-3 and FR-322 do not allow it. Required changes:
  - In this PR, qualify FR-038-AC-3: "Exact byte, depth (up to 16,384), node, ... limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit charged and the consumed counter".
  - Upstream FR-322 (quire-specification): at line 319, permit a reader-declared maximum for `depth`, so that a caller limit above it is charged at that maximum and `incomplete.limit` reports the limit charged. At line 652 and in AC-6, change "Exact selected (resource) limits" to "... up to any reader-declared maximum". An alternative is to refuse an over-maximum limit as a configuration error instead of reading it silently as the maximum.
  - QSL `qsl-package/src/checked_v2.rs`: replace `SERDE_JSON_RECURSION_LIMIT = 128` with a reference to `quire_contract_ir::CheckedPackageReadLimits::MAXIMUM_DEPTH`. Update the `V2ReadLimits.depth`, `enforced()` and `effective_limits` docs, and the module doc's "refused, not Incomplete" bullet. Replace the test `depth_far_past_the_default_limit_is_refused_as_malformed_wire`, which now gets `incomplete(Depth, ...)`.

## Dispositions (disposition pass 2)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-003 | fixed | ae300d3 |

- FND-003: "parse strict JSON once" became "read the document as strict JSON", which no longer claims a pass count.

## Dispositions (disposition pass 3)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-004 | deferred | AC contradiction recorded as a known deviation (FR-038 Reading prose, lines 151-153, and the PR body); upstream amendment STD-125 |

- FND-004: The contradiction remains but is now explicit, not silent: "This ceiling is a known deviation from FR-322 and FR-038-AC-3, which charge the caller's limit as given, pending the upstream amendment tracked as STD-125; it is not a settled rule." That wording is accurate and not misleading. AC-3 stays as written, so the AC remains the authority and the code is the declared exception. Residual for whoever routes STD-125: once it lands, AC-3 needs re-qualifying. The trace tags that bind deviation-asserting tests to AC-3 are SR-624 FND-011.
