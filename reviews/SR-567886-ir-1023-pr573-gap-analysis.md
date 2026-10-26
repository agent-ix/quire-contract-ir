---
id: SR-567886
title: "Gap analysis of IR-1023 QSpec refusal word mapping"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-285; tests/conformance_qspec/main.rs:404"
review_set: subset
---

## Summary

Checked the changed QSpec conformance mapping against the owning FR-038 criterion, the existing integration assertion, and the computed Test Matrix. The PR adds only the missing wire word; it changes no test assertion, fixture, requirement or production reader.

## Verdict

**PASS for the changed mapping** — the criterion and existing test assertion support the word, and no new reverse specification gap or hollow assertion was found. The wider QSpec fixture suite has independent admission failures and is not credited as passing here. This verdict makes no repository-wide coverage claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed.

FR-038-AC-285 names `wrong-export`; the existing protocol-body integration test checks a `WrongExport` refusal. The computed Test Matrix marks FR-038-AC-285 `untagged` on this base; that pre-existing trace gap spans the broader bundled criterion and is outside this one-line PR. No coverage credit is claimed for it here. The PR neither adds nor removes trace tags. Optional semantic review was not requested; the check of this one changed mapping was scoped to its owning criterion and concrete test.
