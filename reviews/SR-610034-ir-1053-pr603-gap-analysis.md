---
id: SR-610034
title: "IR-1053 PR #603 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #603 at f50585f52177269bbeb7405b1846d02b6ba97c8d; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, refinement.rs; tests/it/checked_package_v2_protocol_body.rs, checked_package_v2_refinement.rs; FR-064-AC-2/4/5, FR-038-AC-325"
review_set: subset
---

## Summary

Audited the changed owner and class-catalog behavior against its criterion tags and public reader tests. The positive second-selected-object retarget vector was replaced by a negative interface-owner vector, leaving a required acceptance path without evidence.

## Verdict

**CONDITIONAL** — one medium evidence gap requires a test in this PR. The full memory/drain catalog remains open under IR-1034 and does not receive a passing claim.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The test still tags FR-064-AC-2 but retargets to a selected `systems_interface` and asserts refusal; no test retargets a clause to a second valid selected `object_type`, rebuilds keys, then asserts admission and the changed clause/package identities. This leaves AC-2's positive path and the ticket's two-declaration vector unverified. | FR-064-AC-2; `tests/it/checked_package_v2_protocol_body.rs:1538-1571` | correct-requirement-no-evidence |

## Coverage

- FR-064-AC-2 (examined): A clause's `quire.application-node/v1` preimage contains `owner_model` as its local node id; an otherwise identical non-clause application preimage omits it. Retargeting the edge to a second valid local model and recomputing dependencies, clause key, downstream keys and package id changes the clause key and package id; retaining the old clause key while recomputing the envelope identity refuses `invalid_package`/`stale-node-key` at that clause's `node_id`.
- FR-064-AC-4 (examined): The changed test asserts wrong-kind selected interface refusal; unselected-object typed refusal is not exercised.
- FR-064-AC-5 (examined): The no-step owned class test asserts admission, missing class, wrong node and foreign class refusal through the public reader; new typed writer readback asserts package and refinement node identities.
- FR-038-AC-325 (examined): This PR covers non-memory owner/class joins only. IR-1034 must provide source-default memory/drain implementation and evidence; the ticket remains open.
- The computed matrix command did not return criterion rows during this review; trace tags and test assertions were manually inspected. Plan completion: not assessed. Optional repository-wide semantic review was not requested.
