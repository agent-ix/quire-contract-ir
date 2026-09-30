---
id: SR-629
title: "PR #231 FR-038 spec integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@a567ba99e8601849c77984f19c723a000e664fca; spec/contract/FR-038-consume-checked-package-v2.md (requires_bound prose, lines 541-600, AC-8, AC-39)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---
# SR-629: PR #231 FR-038 spec integrity review

## Summary

Ticket: IR-283 (and IR-284). PR agent-ix/quire-contract-ir#231.
Spec-review with the spec-integrity-analysis sub-analysis, run over the
FR-038 prose rewrite.

Examined:

- The normative lowering statement (line 541-546).
- The unbounded-type paragraph (line 570-576).
- The new position paragraph (line 578-584).
- The new recursion paragraph (line 586-589).
- The typed-at paragraph (line 591-598).
- FR-038-AC-8 and FR-038-AC-39.

Also checked against upstream: QSpec FR-322 (`recursion_group`), QSpec
FR-143, QSL ADR-014 §4 (quire-specification origin/main 4634f5f,
quire-spec-language origin/main a2837b6f). `quire validate` on FR-038 is
clean.

## Verdict

**CHANGES REQUESTED.**

The position paragraph is correct, and the recursion paragraph matches
ADR-014 §4 and FR-143. But the rewrite leaves FR-038 contradicting itself in
two places. It also makes normative the value-path masking that SR-627
FND-001 reports.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038 contradicts itself on recursive composites. Line 572-574, unchanged, says `option`, `record`, `tuple`, `alias` and `reference` never raise `requires_bound`. FR-038-AC-8 says each form of those two families other than the eight unbounded forms does not raise it. The new line 586-589 says any `scalar_type` or `composite_type` with a `recursion_group`, a record included, raises it. | spec/contract/FR-038-consume-checked-package-v2.md:586-589; spec/contract/FR-038-consume-checked-package-v2.md:572-574; spec/contract/FR-038-consume-checked-package-v2.md:623 |
| FND-002 | medium | The normative "shall" statement was not updated. It still defines `requires_bound` as a reachable unbounded numeric, text or collection type, typed at by some reachable node, with no reachable `bounded_domain` typed by it. That is the closure-wide masking rule. It does not cover the composite-position rule or the recursion rule, which the prose now adds. | spec/contract/FR-038-consume-checked-package-v2.md:541-546 |
| FND-003 | medium | The new sentence "A type that a value, expression or the requested node is typed at is bounded when the closure holds a reachable `bounded_domain`..." makes normative the closure-wide masking for value positions. Under it, the outer `+` of `(x + 1) + n` over `x: Int[0,9], n: Integer` lowers. That contradicts QSL ADR-014 §4 and FR-097 (the record is `Unbounded` at `n`) and IR-283's own acceptance. | spec/contract/FR-038-consume-checked-package-v2.md:574-576 |
| FND-004 | low | "raises `requires_bound` naming the least such node key" reads as the least recursive node. The general rule (line 565-567) and the code name the least node over every offending type in the closure, so when an unranged `integer` key sorts below the recursive type, the integer is named. The paragraph should defer to the general least-key rule. | spec/contract/FR-038-consume-checked-package-v2.md:588-589 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The fallback rule says a type "typed at by the requested node, a `literal` value or an `expression` but ... named at no position" is bounded by any reachable domain over it. The code applies that fallback to every non-position typed use: a function's result type, a `bounded_domain`'s base, a claim, other `value` forms, and a type named through `dependencies`. The sentence leaves those uses without a stated rule. Replacing the list with "any other reachable node" would match the code. | spec/contract/FR-038-consume-checked-package-v2.md:597-601 |

## Dispositions

Round 1, reviewed at edb862819349fae5ba5bf2a19ea565ab1a3f96dd.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | edb8628 |
| FND-002 | fixed | edb8628 |
| FND-003 | fixed | edb8628 |
| FND-004 | fixed | edb8628 |
