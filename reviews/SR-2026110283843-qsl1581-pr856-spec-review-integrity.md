---
id: SR-2026110283843
title: "QSL-1581 IR carrier consistency review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; FR-052 exhaustion carrier and AC-10/26/30"
review_set: subset
---
## Summary

Checked the two live QSL owner paths against the proposed neutral exhaustion fields. The spine can report stage and setting; the retained composed lane cannot, so the new required-field rule conflicts with the requirement to preserve owner-native facts without fabrication.

## Verdict

**FAIL** — represent unavailable stage and setting explicitly or remove the composed producer before requiring them.

## Scope

Examined FR-052 neutral stop projection and AC-10/26/30, QSL FR-514 inventory cause, QSL FR-255 setting names, and `src/checking/composed/work.rs::Exhaustion` plus `src/checking/source_inventory.rs::exhaustion` on current QSL main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The active composed producer has no owner-native stage or setting, but `SourceTypeExhaustion` requires both as exact names; the specified projection cannot be implemented without fabricating data. | spec/model/functional/FR-052-source-obligation-correspondence.md:249 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The two owner paths now have explicit disjoint presence rules: exact Some stage/setting/observed for spine limits, None for the composed lane, which keeps measured prior/requested and native site. |
