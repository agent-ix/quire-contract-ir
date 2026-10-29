---
id: SR-612135
title: "Specification integrity review of IR-1059 PR 663"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md"
review_set: subset
---

## Summary

Reviewed IR PR #663 against QSpec origin/main. The changed FR-038 and FR-069 obligations preserve the published TK-01..35, TW-00..26 and NK-00..07 outcomes, stage order, and authoritative checkout boundary. No applicable AssuranceProfile was found.

## Verdict

**PASS** — No defect found in the changed requirements or their QSpec alignment.

## Scope

- FR-038-AC-127 (examined): The derived-shape re-derivation runs after the graph-shape, application-key and nominal-key stages; owner joins precede structural passes, while a stale nominal key wins over an owner defect.
- FR-038-AC-343 (examined): The public V2 reader replays NK-00..07, checks exact code, cause and RFC 6901 locus, and orders application, nominal, owner-join and structural key stages.
- FR-069-AC-6 (examined): Every published TK-01..35 and TW-00..26 case returns exact admission or refusal code, cause and location at its specified stage, and new published cases are replayed.
- FR-069-AC-8 (examined): Conformance loads QSpec artifacts from QUIRE_SPECIFICATION_DIR and runs every published tuple/union intake and wire case, including admitted controls.
- FR-069 Behavior (examined): Conformance SHALL replay every published intake_cases entry and every wire_cases entry; added cases are not silently skipped by a fixed count or range.
- QSpec FR-322-AC-160..162 (context_only): Nominal key stage follows application-key and precedes structural/owner join; NK-00..07 bind exact outcomes and dependent-reference deferral.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
