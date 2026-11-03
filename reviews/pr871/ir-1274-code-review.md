---
id: SR-20261103150657
title: "Code review of IR-1274 protocol location reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_locations.rs; tests/conformance_qspec/protocol_locations.rs; FR-038-AC-448; FR-038-AC-449"
review_set: subset
---

## Summary

Reviewed PR 871 against FR-038-AC-448 and FR-038-AC-449 and current QSpec PL-R3 vectors. The reader guard has the specified invalid-package, invalid-value, application-pointer outcome; the conformance fixture erases two published leaf forms.

## Verdict

**CONDITIONAL** — the full-reader PL-R3 parameter and reference cases are not independently exercised.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | PL-R3 parameter and reference rows are rebound to a literal node, so the traced full-reader test cannot detect form-specific regressions or establish the published by-reference graph cases required by AC-448 and AC-449. | tests/conformance_qspec/protocol_locations.rs:83 |

## Coverage

Examined FR-038-AC-448: complete IR packages built by reference from QSpec PL-R3 must refuse each literal, Local parameter, and expression/reference leaf row at the exact application locus, including the copied enclosing location; every schema-valid ValueNode form must refuse.

Examined FR-038-AC-449: the admitted application in each PL-R3 graph must remain admissible and TC-048 must read QSpec PL-R3 and PL-01..03 by reference.

The private `validate_rows` unit loop supplies a synthetic Reference term for all kinds; it is not a full-reader replacement for distinct schema-valid leaf cases. Existing PL-01..03 and PL-R1/2 test paths are unchanged. No CI workflow, package dependency, hash, pin, copied artifact, or compatibility change appears in the PR diff. No applicable AssuranceProfile was found under `spec/`. Focused checks were not rerun; coder's reported results were not used as evidence for this verdict.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | After the fix, `published_leaf_fixture` selects the admitted node by the published node tag and semantic form. PL-R3-leaf-local now selects value/parameter, PL-R3-leaf-reference selects expression/reference, and PL-R3-leaf-literal selects value/literal. The same full-reader loop checks each vector's code, cause and primary pointer. The application admission test uses this same graph context. |
