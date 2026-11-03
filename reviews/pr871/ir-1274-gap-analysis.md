---
id: SR-20261103150658
title: "Gap analysis of IR-1274 protocol location evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-448; FR-038-AC-449; crates/quire-contract-model/src/checked_package/v2/protocol_locations.rs; tests/conformance_qspec/protocol_locations.rs"
review_set: subset
---

## Summary

Compared the two acceptance criteria, changed source, trace tags and published QSpec PL-R3 rows. Both criteria have trace tags, but two published leaf forms are collapsed by the integration fixture.

## Verdict

**CONDITIONAL** — the trace binding exists while the by-reference evidence for two required graph forms is hollow.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The full-reader trace to AC-448/449 exercises the literal leaf for PL-R3-leaf-local and PL-R3-leaf-reference, leaving their published schema-valid parameter and reference cases unverified. | tests/conformance_qspec/protocol_locations.rs:83 |

## Coverage

Examined FR-038-AC-448 and FR-038-AC-449. Trace tags are present on the changed unit and integration tests. The code path performs an application-kind and term check before occurrence, source-map and location checks. PL-01..03 and PL-R1/2 remain covered by the existing by-reference loop. The QSpec PL-R3 vectors are read from the owning checkout, with no local copy. Plan completion: not assessed. Optional semantic review was not separately requested; this scoped review compared the changed code, tests and requirement text.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The corrected fixture maps each QSpec leaf row to an admitted node with the same node tag and semantic form, so the full-reader trace now exercises distinct literal, parameter and expression/reference nodes. Its refusal assertions compare each published code, cause and pointer; the application test uses the same graph context. |
