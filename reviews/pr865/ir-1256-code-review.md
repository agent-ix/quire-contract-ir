---
id: SR-20261103150654
title: "Code review of IR-1256 rational carried quantity reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/structural.rs; tests/conformance_qspec/carried_quantity.rs; FR-038-AC-450..452; FR-056-AC-7"
review_set: subset
---

## Summary

Reviewed the original PR #865 diff against QSpec FR-322-AC-382 and FR-142-AC-31, all published CQ-W and CQ-E cases, and the repository's string-edge policy. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**FAIL** — the fix for the hosted string-edge failure marks a post-intake structural validator as an intake conversion, contrary to FR-056-AC-7. The focused QSpec reader test and `make string-edge` pass, but the latter is green because of the marker.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `carried_quantity_shape` is marked as a string intake edge although it consumes typed terms and makes post-intake branch decisions | crates/quire-contract-model/src/checked_package/v2/structural.rs:903 |

## Coverage

The diff adds no public item, dependency, digest, copied artifact, compatibility path or CI workflow change. The reader admits only bounded `RationalRange` with `exact`; existing structural validation and application mode checks enforce the referenced refusals. Focused `cargo test --workspace --test conformance_qspec published_carried_quantity_vectors_read_as_wire` passed with `QUIRE_SPECIFICATION_DIR` set to the authoritative QSpec checkout. The initially attempted workspace-filter command without the feature failed to compile an unrelated `tests/it/source_obligation_quire.rs` import; the direct conformance target is the relevant check.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The revised reviewed diff removes the marker from `carried_quantity_shape` and introduces the narrow marked `CheckedRoundingMode::from_literal` converter. It validates against the authoritative rounding catalog before returning an owned typed mode; the validator branches on that enum. No serde catchall or JSON clone remains. The focused QSpec test, string-edge scan and rustfmt check pass on the revised reviewed diff. |
