---
id: SR-8460
title: "code-review review of IR-769 admitted scalar result context"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@99732d6ea7306b1eab8aa12db1587ff50f35c016; spec/checked_package/functional/FR-049-admitted-scalar-result-context.md, spec/checked_package/matrix/TC-447-admitted-scalar-result-context.md, spec/spec.md"
review_set: subset
---

## Summary

Reviewed IR-769 at the pinned head. Examined FR-049-AC-1 through AC-6, TC-447 and the spec index; FR-035 and FR-038 supplied context.

## Verdict

**CONDITIONAL** — The finding below requires a spec correction.

## Examined units

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-049 | examined | When an admitted `ContractPackageV1` is queried for one exact requested scalar expression, Contract IR SHALL expose typed reverse consumer context sufficient to identify its result bound without requiring the consumer to search a checked-package graph. |
| FR-049-AC-1 | examined | A reader-admitted QSL package from `function f using v(a: Int[0, 9]): Int[0, 10] pure { a + 1 }`, requested by the exact `quire.op.integer.add` expression key, returns a lowered context whose qualifying `numeric.narrow` conversion identifies result bound `Int[0, 10]`; the operand's `Int[0, 9]` bound remains separately present and is not returned as the result bound. Independent QSL source and range assertions establish both intervals. |
| FR-049-AC-2 | examined | An admitted scalar expression with no qualifying local conversion returns `Unnarrowed` with no result-bound key; one valid conversion returns `Narrowed` with the exact conversion and target keys, while duplicate valid conversions to that same target retain one distinct bound and all conversion keys in typed-key order. |
| FR-049-AC-3 | examined | Two qualifying conversions to distinct target bounds, or one qualifying conversion plus another local body-reference consumer, return `AmbiguousBound` without choosing an operand bound or any one target; a sole conversion with wrong bound form, unreadable target or mismatched scalar type returns `MissingBound`, and an otherwise nonqualifying reference cannot be promoted to a narrowing. |
| FR-049-AC-4 | examined | A non-lowered requested key returns its exact typed disposition; an unrequested or absent key returns typed absence, including when the key occurs in another root's closure. An inconsistent post-admission body/source edge or exhausted work limit returns the existing typed view error with no partial context. |
| FR-049-AC-5 | examined | The result identifies the exact requested key, source package and lowering profile, and returns admitted source-map occurrences for its expression, conversion and bound. A selected external reference retains its selected package/key pair; an identically keyed local node cannot satisfy it, and an unavailable selected package or target is refused at reader admission rather than yielding a guessed local bound. |
| FR-049-AC-6 | examined | Querying all outcomes leaves the six FR-330 wire fields and canonical identity byte-for-byte unchanged; the preexisting `reached_closure` key set does not grow to include a reverse consumer, and CG can consume the typed context without an original checked graph or a second lowering pass. |
| TC-447 | examined | Verify FR-049-AC-1 through FR-049-AC-6 against the public reader-admitted QSL-to-IR path and focused adversarial consumer shapes. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The stray word “references;” breaks the consumer-classification sentence and should be removed. | spec/checked_package/functional/FR-049-admitted-scalar-result-context.md:69 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | d254e62b1456441ba2728f46a1efe208d562bfbc — The stray word is gone; the classification sentence now ends at “knowledge.” |
