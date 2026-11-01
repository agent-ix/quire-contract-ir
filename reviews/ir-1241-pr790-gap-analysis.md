---
id: SR-202610310149
title: "Gap analysis of IR-1241 QSpec conformance harness PR 790"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 790 frozen head 3d798253; tests/conformance_qspec/{extension,hybrid,main,model_tuple_union,node_index,protocol_vectors,reaches_population,selection_vectors,timed_model}.rs; IR-1241"
review_set: subset
---

## Summary

Ticket: IR-1241. Scoped the computed Test Matrix, changed tests, and the QSpec vector read path to this harness repair. No production source or requirement changed. Existing trace tags bind the changed test functions to their criteria, and the fixed helper paths exercise reader admission or exact refusal rather than asserting only their own constructed values. No applicable AssuranceProfile was found under `spec/`.

## Verdict

PASS for the scoped PR diff. The current QSpec run still has six attributed failures: XP rows await IR-1247, three reaches positives await IR-1249, and CL-57 awaits the separately merged IR-1234 reader on a newer base. This is not a whole-repository coverage claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format tsv` completed on the frozen PR tree (894 records including header); FR-035-AC-9 is tagged by two changed `selection_vectors.rs` tests and a separate integration test. The changed test names and trace attributes for the QSpec reader controls were inspected; no test was added without a requirement binding. The QSpec checkout is read by reference through `QUIRE_SPECIFICATION_DIR` and no vector was copied into IR. Optional full semantic review was not invoked; the changed test assertions were checked directly for the failure modes that motivated the PR.
