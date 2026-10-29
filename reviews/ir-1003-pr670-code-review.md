---
id: SR-2026102729
title: "IR-1003 PR 670 Rust code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 670; refinement.rs and checked_package_v2_refinement.rs; FR-061-AC-3/7"
review_set: subset
---

## Summary

Reviewed the PR 670 production and test diff against IR FR-061-AC-3/7 and QSpec FR-375-AC-33/34. The new owner check covers population, object and history declarations and operation anchors. The existing catalog comparison retains the extra-versus-omitted class tie-break. No QSpec file or fixture was copied into IR. Focused test execution was pending the shared build lock at review time.

## Verdict

**FAIL** until FND-001 is fixed in this PR. A catalog-backed step can name a non-anchor concrete declaration from another selected model and reach catalog comparison without the mandated row-key refusal.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Catalog-backed steps check the selected concrete identity only when `source_node` is an operation anchor. A foreign protocol declaration or frame that matches the listed catalog node reaches the later catalog comparison, which refuses at the catalog entry or array instead of `steps[i].source_node`. Check every declared concrete source node's owner before catalog comparison. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:1692 |

## Coverage

| Unit | Role | Result |
| --- | --- | --- |
| FR-061-AC-3 | examined | Population, object, operation anchor, history, assume and catalog order inspected; non-anchor step owner join missing. |
| FR-061-AC-7 | examined | Row-empty model-subject swap retains distinct node id and public read/write/read control. |
| FR-375-AC-33 | examined | QSpec requires every first foreign concrete declaration or anchor key to refuse before catalog comparison. |
| FR-375-AC-34 | context_only | Reflexive admission is specified upstream; no new wire mechanism was added. |
| TC-062 | examined | IR test-matrix row and changed tests inspected. |

Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At PR, every catalog-backed declared `steps[].source_node` enters `require_concrete_class_source` before catalog comparison. It checks operation anchors through their declaring type, model declarations through their owner, and protocol members through their closest declaring clause's model owner. A new foreign protocol-class step test asserts `invalid_model_binding`/`malformed-declaration` at `steps/0/source_node`. Focused execution and hosted checks were pending at disposition time. |
