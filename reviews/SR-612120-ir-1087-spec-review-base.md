---
id: SR-612120
title: "IR-1087 PR #652 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

IR-1087 PR #652. Examined FR-035-AC-23..26, FR-038-AC-341, unchanged FR-035-AC-22 and TC-047/048 plus FR-019 inventory. Spec-only diff; no code or aggregate gate.

## Verdict

FAIL: unchanged AC-22 conflicts with the new plain-reader case in AC-25.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-25 admits a plain strict V1 reader term with no authenticated views, yet unchanged AC-22 requires that reader term to equal the producer/source CheckedTerm for every represented key. CheckedApplicationOperation and CheckedTerm derive Eq over views, so the plain read term (views=None) cannot equal the producer term (views=Some) for a represented operation. AC-26 only repairs the authenticated path. Define plain-read admission and typed lookup behavior so AC-22 remains literally satisfiable, and add a TC-047 operation-bearing plain-read equality/refusal control; do not silently narrow AC-22. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:248-252 |

## Coverage

All changed files and the unchanged AC-22 equality requirement examined. `quire validate --scope .` on the five changed files: 5/5 grammar-clean. `quoin advise --json` checked all five new AC methods; no mismatch. No backing test tags were added in this spec-only diff; the new criteria are marked Planned/UNRUN. The full computed matrix was not run for this scoped review. Plan completion: not assessed.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | AC-25 newly makes plain strict V1 reading refuse every otherwise valid canonical package with a represented application operation as operation_source_required. This violates unchanged FR-035-AC-15 (exact work limit admits), FR-035-AC-19 (strict reading returns kinds), QSpec FR-330-AC-12 (exact-limit input admitted), and FR-330-AC-9/11 (sole reader and closed diagnostic mapping). operation_source_required has no FR-271 catalog row. Preserve the valid canonical read contract and AC-22 equality without adding an uncatalogued refusal or silently weakening either requirement. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:162-166,256; quire-specification FR-330-AC-9/11/12 |

## Dispositions

Round 1, reviewed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Plain reading of operation-bearing bytes now refuses before returning an unequal term; unchanged AC-22 applies to admitted packages. The new refusal conflicts with existing reader obligations (FND-002). |
| FND-002 | still-open | The new refusal is absent from FR-271 and rejects valid canonical operation packages under exact limits; no compatible repair is in this head. |

Round 2, reviewed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Plain strict reading again admits valid canonical operation packages at exact limits. The new operation-source refusal and uncatalogued code are removed. AC-22 remains verbatim; represented-body CheckedTerm equality now excludes the private view cache while fallible accessors retain provenance. TC-047 covers plain admission, equality, nested operations and refusal precedence. |
