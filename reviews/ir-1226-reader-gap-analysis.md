---
id: SR-202611010320
title: "Traceability and evidence gap analysis of IR-1226 reader PR 812"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 812 operation-effect carrier and selected object-operation projection; FR-019-AC-21, FR-038-AC-406..408, FR-354-AC-10..11"
review_set: subset
---

## Summary

Ticket: IR-1226. Plan completion: not assessed. Queried `quoin matrix --repo . --json` on the frozen PR #812 worktree: all six in-scope IR criteria have static trace binders, with no run evidence attached. Inspected their test assertions and production paths, plus QSpec TC-977 vectors by reference. Optional semantic review was not requested. No plan was supplied.

## Scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-019-AC-21 | examined | Typed producer rows and borrowed admitted operation-effect views are available through the public model API. |
| FR-038-AC-406 | examined | OEC-01..07 admit and expose each effect kind, an empty effect and distinct owners/documents by reference. |
| FR-038-AC-407 | examined | OES-01..14 and OER-01..26 refuse with the exact code, cause and locus, and null/empty/present carrier states remain distinct. |
| FR-038-AC-408 | examined | Carrier rows and entries charge validation work before inspection; one below the required work returns incomplete at the first uncharged locus. |
| FR-354-AC-10 | examined | Ledger pay/payTaxed own operations expose ordered signature and producer effect. |
| FR-354-AC-11 | examined | Conflicting producer facts, including changed ordered parameter names/types or optional result under matching document/owner/member, refuse before a partial type view. |
| FR-322-AC-303 | context_only | The QSpec operation-effect carrier includes document, declaring node, operation, effect and provenance; it carries no parameter or result signature. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-354-AC-11's changed producer-signature branch has no representable producer input: the QSpec effect row and existing operation-member witness carry no parameter/result signature, while the tagged Ledger test changes none. The criterion cannot fail for that case. | spec/checked_package/functional/FR-354-project-admitted-runtime-context.md:178 |
| FND-002 | medium | FR-038-AC-408 is tagged only by the vector test, which never supplies a reduced validation-work limit; no test asserts incomplete at the first uncharged effect-entry locus, and production shape inspection precedes charging. | tests/conformance_qspec/operation_effects.rs:239 |

## Coverage

Computed matrix: FR-019-AC-21, FR-038-AC-406..408 and FR-354-AC-10..11 are statically tagged, but `quoin matrix` reports no run evidence for them on this worktree. The test bindings were inspected for behavior rather than counted as proof. Production coverage of the new carrier and projection was scoped to this PR diff; no repository-wide assurance PASS is claimed. Plan completion: not assessed. Optional semantic review: skipped because it was not requested.

## Verdict

FAIL for the in-scope criterion-to-evidence chain. Repair the impossible FR-354 signature-conflict claim according to the agreed QSpec/IR ownership, and add a fail-capable validation-limit test for FR-038-AC-408. Code review separately records implementation and exact-cause assertion defects.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-354-AC-11 now derives signature only from admission-retained OperationDecl and limits producer witness to effect; TC-977 negative vectors carry its trace tag. |
| FND-002 | fixed | A tagged bounded-work test asserts incomplete at the second duplicate effect entry one charge below the refusal threshold. |

## Disposition verdict

Round 1 at `8503368b7f2a15b8bc168530a67353bbd4bb1904`: all recorded findings fixed. Focused reader, TC-058, TC-977, string-edge and formatting checks passed according to the coder; the aggregate gate remains with the lead.
