---
id: SR-2026110101303
title: "IR-1272 PR 836 base specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1272 PR 836; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed the two new FR-038 acceptance criteria against merged QSpec main 6c45e0bc FR-464, FR-322-AC-23, TC-946 V13/V14, and the published mutation manifest. Quire validates the edited artifact; its only grammar warning is inherited at unchanged line 5507. No applicable AssuranceProfile was found under spec/.

## Verdict

PASS. Both criteria have unique IDs, observable adverse outcomes, exact code/cause/node loci, and Test verification. The Planned state is explicit. The text cites the merged source by reference and does not invent a primary document pointer or narrow the existing protocol requirements.

## Reviewed units

- FR-038-AC-435: examined. Planned until IR-1272 lands. Every admitted `quire.op.protocol.control` body on a protocol member has `result_type` and node `semantic_type` equal to the package's Boolean scalar-type node; equality of those two references to another type is insufficient. A resolved non-Boolean type refuses `ill_typed`/`operator-ineligible` at the containing protocol node with no admitted package. TC-048 reads the merged QSpec STD-609 protocol-node mutation manifest and base fixture by reference, applies V13 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/64`, with that node as structural locus.
- FR-038-AC-436: examined. Planned until IR-1272 lands. A state-clause body reference to a protocol member node other than a `protocol_clause` or `protocol_reference` carrier refuses `ill_typed`/`operator-ineligible` at the containing state-clause node with no admitted package, even when the referenced member is Boolean typed. The protocol step applies this placement rule after earlier graph and state defects. TC-048 reads the same merged manifest and base fixture by reference, applies V14 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/54`, with that node as structural locus.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
