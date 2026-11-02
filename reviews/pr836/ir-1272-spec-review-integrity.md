---
id: SR-2026110101304
title: "IR-1272 PR 836 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1272 PR 836; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Checked atomicity, consistency, failure locus, and testability for FR-038-AC-435/436 against existing FR-038-AC-281/284 and merged QSpec FR-464/TC-946.

## Verdict

PASS. AC-435 isolates the protocol-result type rule and V13; AC-436 isolates cross-family reference placement and V14. The two criteria do not conflict with earlier graph/state precedence and each has a discriminating negative control.

## Reviewed units

- FR-038-AC-435: examined. Planned until IR-1272 lands. Every admitted `quire.op.protocol.control` body on a protocol member has `result_type` and node `semantic_type` equal to the package's Boolean scalar-type node; equality of those two references to another type is insufficient. A resolved non-Boolean type refuses `ill_typed`/`operator-ineligible` at the containing protocol node with no admitted package. TC-048 reads the merged QSpec STD-609 protocol-node mutation manifest and base fixture by reference, applies V13 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/64`, with that node as structural locus.
- FR-038-AC-436: examined. Planned until IR-1272 lands. A state-clause body reference to a protocol member node other than a `protocol_clause` or `protocol_reference` carrier refuses `ill_typed`/`operator-ineligible` at the containing state-clause node with no admitted package, even when the referenced member is Boolean typed. The protocol step applies this placement rule after earlier graph and state defects. TC-048 reads the same merged manifest and base fixture by reference, applies V14 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/54`, with that node as structural locus.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
