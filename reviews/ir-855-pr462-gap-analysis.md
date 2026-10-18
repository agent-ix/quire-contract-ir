---
id: SR-12556
title: IR-855 PR 462 gap analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/lib.rs; crates/quire-contract-model/src/wire.rs;
  spec/conformance/matrix/tests.md; spec/model/matrix/tests.md; tests/it/conformance.rs;
  tests/it/fr019_api.rs; tests/it/main.rs
review_set: subset
---

## Summary

Ticket IR-855. Diff-scoped mechanical requirement-to-matrix-to-test and reverse-gap analysis at 40a7fb6c95465af84dc5c8b162ac87e8ae19e2fa. AC-8 through AC-13 have new trace binders; the field guard is owned by AC-9. Test-intent omissions are recorded in SR-12555.

## Examined units

- **FR-019-AC-8** (`examined`): "Compile-time fixtures and public-source inspection show package and expression wire/request values are distinct from validated semantic values, validated fields are private, and no unchecked constructor or `From` conversion crosses that boundary."
- **FR-019-AC-9** (`examined`): "An otherwise valid package or expression request with one unknown object member is refused as `invalid_wire_format`, consistently with its published schema, before it yields a validated value."
- **FR-019-AC-10** (`examined`): "Public package and expression decoding, semantic validation, canonicalization, and coverage classification signatures return typed fallible results; none silently converts an invalid request into a validated value."
- **FR-019-AC-11** (`examined`): "Each public canonicalization entrypoint requires an explicit `CanonicalProfile`, and the closed profile exposes only `quire.contract.canonical-json/v1`; omitting the profile does not compile."
- **FR-019-AC-12** (`examined`): "A default-feature external API test imports `MAX_SEMANTIC_NODES: u32 = 25000`, `MAX_SEMANTIC_DEPTH: u32 = 256`, `MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10000`, and `MAX_WIRE_JSON_DEPTH: u32 = 576` from `quire_contract_model` and checks their exact values."
- **FR-019-AC-13** (`examined`): "Public-source signature inspection finds no host handle, filesystem path, mutable cache, downstream engine type, or schema-library type in the default-feature semantic API subset defined in Behavior; it excludes the conformance runner and fixture APIs and runs on stable Rust without nightly rustdoc JSON."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Merge base 14f875f: 527/602 backed; head: 533/602 backed. Both report 40 strict unbacked and zero contradicted. The normalized matrix has 568 criterion rows on both sides; only FR-019-AC-8 through AC-13 acquire new binders/status. Plan completion: not assessed. Optional repository-wide semantic review: not run.

## Verdict

PASS for the mechanical trace and reverse-gap delta. No new unbacked criterion, missing owner, stub, or unmatched changed-code trace was found. SR-12555 records the partial inspection evidence.
