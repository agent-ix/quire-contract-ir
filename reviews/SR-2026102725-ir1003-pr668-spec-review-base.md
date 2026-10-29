---
id: SR-2026102725
title: "IR-1003 concrete-root swap specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 668; FR-061-AC-3/7; TC-062; QSpec FR-375-AC-33/34 and TC-310 SR-01..13"
review_set: subset
---

## Summary

Reviewed the edited IR criteria and matrix against QSpec. The refusal stages, row-empty model admission, protocol catalog comparison, and graph-case status of SR-01..13 agree with the source contract. The matrix lacks a population-key precedence control.

## Verdict

**FAIL** until FND-001 is fixed in this PR. No applicable AssuranceProfile with an enforced spec-review operation was found in the changed module.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-062 does not test a foreign `populations[].concrete` key before another foreign row key, so the first array in AC-3's mandatory refusal order has no discriminating oracle. Add a population-versus-object or population-versus-step control with the exact population-key refusal. | spec/checked_package/matrix/TC-062-refinement-reader-writer.md:64 |

## Coverage

| Unit | Role | Result |
| --- | --- | --- |
| FR-061-AC-3 | examined | QSpec FR-375-AC-33 refusal stages and loci agree. |
| FR-061-AC-7 | examined | Row-empty model and protocol admission agree with SR-06/07. |
| TC-062 two-selection controls | examined | Missing population precedence control; other stated controls agree with SR-01..13. |
| FR-375-AC-34 | context_only | Abstract-equals-concrete admission is specified upstream; its wire mechanism remains pending. |

No new hash, digest, pin, or tracking record was introduced by the diff. QSpec SR-01..13 are graph cases, so TC-062 correctly requires IR-owned wire packages.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At rebased PR, TC-062 requires a foreign `populations[0].concrete` before foreign object and step keys and the exact `invalid_model_binding`/`malformed-declaration` refusal at `/body/populations/0/concrete`. |
