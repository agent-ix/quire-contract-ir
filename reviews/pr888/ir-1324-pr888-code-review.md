---
id: SR-20261104133001
title: "IR-1324 PR 888 code-review specification review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 888; FR-352 AC-12 through AC-15"
review_set: subset
---

## Summary

Independent code-review review of frozen PR #888 at e5941b6f3c02a4e1f1100329c02a6f602650832c against measured IR main 35a96493cd3a87ef6ca5d5fe5b9446c94a8f6f69.

## Verdict

PASS. The spec diff introduces no source code, copied artifact, hash, digest, pin, or compatibility layer; its four planned criteria stay within the admitted package projection boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: examined; id: FR-352-AC-12; excerpt: PLANNED/UNRUN (IR-1324). For an admitted selected record with a plain `Collection<Integer>` member, the public record view returns a witnessed Exact `MemberId`, the original IR field identity and `Presence::Required`, and an Exact collection whose kind, integer element and cardinality bound equal the admitted member. Distinct sequence, set, bag and ordered-set kinds remain distinct, as do unbounded and `[0,3]` cardinality. The complete view can be formed from checked-package bytes and selected producer rows without a source recheck.
- role: examined; id: FR-352-AC-13; excerpt: PLANNED/UNRUN (IR-1324). An admitted selected record containing `Collection<Option<Integer>>` returns an Exact collection with the admitted kind and bound and an Exact `Option<Integer>` element. An optional field with a collection or another nested `Option` reports `Presence::Optional` after consuming exactly its outer presence layer and retains all inner wrappers in its Exact value type. The already supported optional `Option<String>` field still reports `Presence::Optional` and Exact `String`.
- role: examined; id: FR-352-AC-14; excerpt: PLANNED/UNRUN (IR-1324). A nested member reference to a witnessed record in a selected cyclic nominal graph retains `Composite` with that record's original Exact `NodeKey` and terminates without expanding the cycle. A missing, foreign, conflicting or wrong-form nested identity refuses with its native typed cause and first referenced declaration; an unrepresentable nested leaf refuses as `UnsupportedExactType`. Neither refusal exposes a partial record view.
- role: examined; id: FR-352-AC-15; excerpt: PLANNED/UNRUN (IR-1324). A selected record with deeply nested admitted Option and Collection members projects using bounded iterative work and a small host stack. At the exact work allowance it returns the complete view; one less refuses with typed work exhaustion, charged work and first uncharged member locus. Equal admitted inputs and allowances, including a package clone, return equal views or equal first stops.
- role: examined; id: FR-352; excerpt: For a selected record field, Contract IR SHALL project the complete admission-retained member type into the corresponding Exact `ValueType`.

## Verification and Limits

Checked the one-file frozen diff, FR-352 in context, the current IR projection shape, and Exact ValueType variants. Quire validation was run on the changed FR. The review verifies requirement wording and does not claim code implementation or QSL parity.
