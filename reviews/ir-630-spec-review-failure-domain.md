---
id: SR-7124
title: "IR-630 PR 346 failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@f84b9b278b3547da072d2da7b63af2674725c00f; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md"
review_set: subset
---

## Summary

Ticket: IR-630. The changed clauses cover graph self-reference, model-owner admission, and distinct table-versus-graph exhaustion loci. No additional failure-domain omission was found in this diff.

## Verdict

**PASS** — No findings in this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-18 | examined | For a keyed input that reaches the graph-reference stage, a self-typed node's own body-root `literal.type` naming itself is exempt from the reference-cycle check and needs no declared `recursion_group`. The same self-reference nested in an `aggregate` member, `binding` value or `application` argument is not exempt and refuses as `invalid_semantic_graph` at `/semantic_graph/nodes/<n>` (the node object lacking `recursion_group`), as do a self-typed node's `reference` body and `application.result_type` naming itself. Public admission authenticates content-derived structural and applicat |
| FR-038-AC-139 | examined | Absence and refusal semantics (IR-628; implemented, ungated). `UnknownNode` for an id whose digest is no node of the graph; `NotModelObjectType` for an admitted `scalar_type` node, a `systems_interface` declaration node and a `relationship` declaration node; a selected `model`/`object_type` declaration whose retained body is changed from `aggregate{[]}` refuses admission as `invalid_package`/`stale-node-key`, and an unselected model owner refuses at owner joining before the accessor is available; `AmbiguousField` carrying `x` for an object type whose document gives two unhidden effec |
| FR-038-AC-144 | examined | Resolution is charged once, at admission (IR-628; implemented, ungated). Cost model of items 8 and 9, with the arithmetic. For a chain of N object types, the first declaring one field and each later one declaring one field and extending the previous, the charge is 1 for the first type and k + 1 for the type at depth k >= 2 (one own field, one supertype edge, k - 1 copied entries), in all N(N + 1)/2 + N - 1: 13 for N = 4 (1 + 3 + 4 + 5) and 43 for N = 8. A crate-internal test reads the table-building counter and asserts 13 and 43. For the same chains, the smallest whole-read `work` li |
| TC-048 | examined | Changed AC-18 graph-reference test procedure. |
| TC-227 | examined | Changed AC-139 and AC-144 accessor and budget test procedure. |
