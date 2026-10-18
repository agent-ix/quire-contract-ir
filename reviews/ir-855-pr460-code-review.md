---
id: SR-12550
title: "IR-855 PR 460 Code review of test-to-criterion alignment"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md; spec/model/matrix/tests.md; spec/conformance/matrix/tests.md; tests/it/conformance.rs; Cargo.toml; crates/quire-contract-model/src/conformance.rs"
review_set: subset
---

## Summary

Reviewed PR 460 at b9ee487399b6e177f1604ae2972ee1f7938c1267. The old AC-1 clauses map to narrowed AC-1 and new AC-8 through AC-13: registries, wire/validated boundary, unknown-member refusal, fallible signatures, explicit profile, four public limits, and forbidden signature vocabulary. The six new criteria are planned, with no test tags in this branch. The finding(s) below identify evidence or boundary wording that still needs correction.

## Examined units

- **FR-019-AC-1** (`examined`): "A default-feature external API test imports `PUBLIC_CONSTRUCT_TAGS`, `CONFORMANCE_BOUNDARIES`, `DiagnosticCode::ALL`, `ConformanceOperation::ALL`, and `expected_inventory` from `quire_contract_model` and uses them to construct the published inventory."
- **FR-019-AC-8** (`examined`): "Compile-time fixtures and public-source inspection show package and expression wire/request values are distinct from validated semantic values, validated fields are private, and no unchecked constructor or `From` conversion crosses that boundary."
- **FR-019-AC-9** (`examined`): "An otherwise valid package or expression request with one unknown object member is refused as `invalid_wire_format`, consistently with its published schema, before it yields a validated value."
- **FR-019-AC-10** (`examined`): "Public package and expression decoding, semantic validation, canonicalization, and coverage classification signatures return typed fallible results; none silently converts an invalid request into a validated value."
- **FR-019-AC-11** (`examined`): "Each public canonicalization entrypoint requires an explicit `CanonicalProfile`, and the closed profile exposes only `quire.contract.canonical-json/v1`; omitting the profile does not compile."
- **FR-019-AC-12** (`examined`): "A default-feature external API test imports `MAX_SEMANTIC_NODES: u32 = 25000`, `MAX_SEMANTIC_DEPTH: u32 = 256`, `MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10000`, and `MAX_WIRE_JSON_DEPTH: u32 = 576` from `quire_contract_model` and checks their exact values."
- **FR-019-AC-13** (`examined`): "Public-source signature inspection finds no host handle, filesystem path, mutable cache, downstream engine type, or schema-library type in the default-feature semantic API; the check runs on stable Rust without nightly rustdoc JSON."
- **TC-018 model and conformance matrix rows** (`examined`): AC-8 through AC-13 are expressly planned; existing corpus and interface checks are identified as implemented.
- **Root Cargo feature graph and public conformance signature** (`context_only`): root dev-dependency enables `fault-injection`; `run_corpus` takes two `&Path` arguments.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-1 claims a default-feature external API test, but its sole tagged TC-018 test links the model with the root dev-dependency fault-injection feature. | spec/model/functional/FR-019-rust-library-interface.md:210 |

## Verdict

FAIL pending the stated correction(s). The original AC-1 obligations remain represented and the six new rows are honestly planned; this verdict concerns the current wording and evidence attribution.
