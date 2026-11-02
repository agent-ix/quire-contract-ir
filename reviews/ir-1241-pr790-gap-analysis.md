---
id: SR-202610310149
title: "Gap analysis of IR-1241 QSpec conformance harness PR 790"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 790 frozen head c7acd764; tests/conformance_qspec/{dimension_equivalence,extension,hybrid,main,model_tuple_union,node_index,protocol_vectors,reaches_population,selection_vectors,timed_model}.rs and tests/it/checked_package_v2_owners.rs; IR-1241"
review_set: subset
---

## Summary

Ticket: IR-1241. Scoped the computed Test Matrix, changed tests, and the QSpec vector read path to this harness repair. No production source or requirement changed. Existing trace tags bind the changed test functions to their criteria, and the fixed helper paths exercise reader admission or exact refusal rather than asserting only their own constructed values. No applicable AssuranceProfile was found under `spec/`.

## Verdict

PASS for the scoped PR diff at c7acd764. The exact-head QSpec run has three attributed reaches failures owned by IR-1249. Both XP tests pass after the typed-operand harness update; the TC-229 node-57 refusal is an exact-base failure owned by IR-1251. This is not a whole-repository coverage claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format tsv` completed on the earlier frozen PR tree (894 records including header); FR-035-AC-9 is tagged by two changed `selection_vectors.rs` tests and a separate integration test. The new TC-229 edit retains its `#[trace("TC-229", "FR-038-AC-304", "FR-038-AC-305", "FR-038-AC-306", "FR-038-AC-307")]` bindings, and its dependency order is checked by the public reader rather than by a self-assertion. The QSpec checkout is read by reference through `QUIRE_SPECIFICATION_DIR` and no vector was copied into IR. Optional full semantic review was not invoked; the changed test assertions were checked directly for the failure modes that motivated the PR.

## Recheck at ae89204a

The XP tests now carry `FR-038-AC-387..389` tags and exercise every published case through the public reader. XP-01 asserts retained typed operands and exact scale; adverse rows assert code, cause when supplied, and exact pointer including operand and member. XP-05's operand comes from QSpec #579, so the test will follow a corrected published index. The case inventory is read by reference from QSpec c6784e45; no vectors or schema were copied into this repo. QSpec conformance at ae89204a reports 162 passed/3 reaches failures; the latter are outside this harness change. No new gap was found.

## Recheck at c7acd764

The reaches fixture now derives dependency order from graph-resolved structural node IDs in both mutation paths, so the test reaches the reader rather than panicking on indexed identity syntax. QSpec #580 clarifies the legal assigned equivalence label used by the fixture. Titan's focused conformance on exact c7acd764/QSpec 2f8b9c8b reports 162 passed/3 failures, all three previously attributed IR-1249 reaches TC-048/233/453 at the same `OperatorIneligible` population locus. Both XP tests pass. No new trace or reverse-ownership gap was found in the changed lines.
