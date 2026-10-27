---
id: SR-610050
title: "Gap analysis of IR-1060 PR 608"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/timed_model.rs; tests/conformance_qspec/{dimension_equivalence,extension,model_root,timed_model}.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Reviewed the changed requirement-to-test-to-code chain in PR #608 at 073d4a549eda432ce2979d2c6356f2e3e9bac590. Ticket: IR-1060. The new tick admission test does not yet cover its writer path, although the dense fixture round-trip does.

## Verdict

**CONDITIONAL** — add an exact tick package write/read/read assertion on the original PR, using the public typed encoder and preserving the typed operation-anchor body and package identity.

## Coverage

Plan completion: not assessed. The computed Quoin matrix was read for the repository; this artifact scopes conclusions to changed FR-038-AC-318/322/323 and FR-065-AC-1/3/4/5 paths, not an assertion that the whole repository has no gaps. The semantic review was not separately opted into. Examined the production `typed_members` call path, the published healthy tick fixture, malformed timed vectors including TM-23, root MR-00/01/02/03/04/06/09, and existing dense write/read/read test. FR-065-AC-2 source correspondence remains assigned to IR-1043 and is outside this PR's new claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The healthy tick test calls the strict reader once and inspects its anchor, but never sends that admitted tick package through `encode_checked_package_v2` and reads it again. The only complete writer round-trip in this file uses the dense fixture, so the newly admitted tick body could be lost or changed by the public writer without this slice failing. | tests/conformance_qspec/timed_model.rs:19 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 7a24be8d, the healthy tick test uses the public typed encoder, compares the emitted JSON with the authoritative fixture, reads it back, compares graph, package id and operation-anchor term, then encodes again and compares bytes. |
