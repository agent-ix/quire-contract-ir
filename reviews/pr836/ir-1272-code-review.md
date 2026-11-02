---
id: SR-2026110101302
title: "IR-1272 PR 836 diff hygiene review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1272 PR 836; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed frozen PR #836 head 7c51b4a5. The diff adds only two planned acceptance-criterion rows to FR-038; no production code, tests, schema, copied fixture, or workflow changes appear in this PR.

## Verdict

PASS. The diff is scoped to the ticket, whitespace-clean, and adds no hash, digest, pin, or copied upstream artifact. Rust review and gap analysis are inapplicable to this spec-only PR.

## Reviewed units

- FR-038-AC-435: examined. Planned until IR-1272 lands. Every admitted `quire.op.protocol.control` body on a protocol member has `result_type` and node `semantic_type` equal to the package's Boolean scalar-type node; equality of those two references to another type is insufficient. A resolved non-Boolean type refuses `ill_typed`/`operator-ineligible` at the containing protocol node with no admitted package. TC-048 reads the merged QSpec STD-609 protocol-node mutation manifest and base fixture by reference, applies V13 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/64`, with that node as structural locus.
- FR-038-AC-436: examined. Planned until IR-1272 lands. A state-clause body reference to a protocol member node other than a `protocol_clause` or `protocol_reference` carrier refuses `ill_typed`/`operator-ineligible` at the containing state-clause node with no admitted package, even when the referenced member is Boolean typed. The protocol step applies this placement rule after earlier graph and state defects. TC-048 reads the same merged manifest and base fixture by reference, applies V14 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/54`, with that node as structural locus.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
