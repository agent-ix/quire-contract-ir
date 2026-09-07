---
id: SR-030
title: "Conformance result binding preimplementation review"
type: SpecReview
analysis: gap-analysis
scope: "issue #44 suite registry and native producer trace targets"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-005
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
---

# SR-030: Conformance result binding preimplementation review

## Summary

The native corpus can own explicit verification-target metadata without owning the obligation graph.
The currently pinned Quoin adapter cannot yet preserve that metadata into evidence entries.

## Verdict

**PASS for the IR-owned producer change.** Use the existing TC-015 through TC-018 identities and
Quire's derived criterion relationships. Do not create a second obligation graph or a local adapter.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-301 | high | A suite declaration alone cannot bind: Quoin 0.23.1's `contract-conformance` adapter emits entries without `traceIds`. Preserve explicit ids in native rows and file the adapter gap upstream. | issue #44, PLAN-005 | correct-requirement-no-evidence |
| FND-302 | high | A fixture with an empty, duplicate, reordered, or undeclared trace set could silently stop contributing to the suite. Make `trace_ids` required, bounded by schema, and checked for non-empty sorted uniqueness. | FR-018-AC-1, TC-018 | missing-requirement |
| FND-303 | medium | Repeating FR acceptance criteria in a corpus mapping would create a second static obligation graph. Emit TC ids; Quire remains the owner of Test Case-to-criterion relations. | PGM-01-R09, issue #44 | wrong-requirement |
| FND-304 | medium | Adding trace metadata must not alter semantic inputs, expectations, canonical bytes, or coverage observations. Regenerate and compare the complete corpus. | FR-018-AC-3, TC-018 | implementation-bug-despite-evidence |
