---
id: SR-12555
title: IR-855 PR 462 code and Rust review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/lib.rs; crates/quire-contract-model/src/wire.rs;
  spec/conformance/matrix/tests.md; spec/model/matrix/tests.md; tests/it/conformance.rs;
  tests/it/fr019_api.rs; tests/it/main.rs
review_set: subset
---

## Summary

Ticket IR-855. Non-author diff-scoped code and Rust review of PR #462 at 40a7fb6c95465af84dc5c8b162ac87e8ae19e2fa. The only production change adds deny_unknown_fields to WireExpressionKind; FR-019 line 93 and AC-9 require it. Focused tests pass (six new API tests, decoder unit test, conformance test). Removing the attribute and raising MAX_SEMANTIC_NODES each fails its targeted test. No CI workflow diff, vendoring, unsafe, new panic path, or downstream direct expression feeder found in QSL, CG, or QSpec source scans.

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
| FND-001 | medium | The AC-8 inspection rejects public fields and From impls, but does not inspect public inherent constructors. A new public unchecked constructor inside identity.rs or expression.rs would leave this tagged test green while allowing validated values to bypass checks. Add an explicit constructor inventory or a mutant that introduces a public unchecked constructor. | tests/it/fr019_api.rs:107-132 |
| FND-002 | medium | The fallible-signature list omits BoundPackage::from_json_bytes (binding.rs:210), a public semantic decode entrypoint. The tagged AC-10 test never examines that signature. Include it in the inspected boundary list and a targeted mutation. | tests/it/fr019_api.rs:137-153 |
| FND-003 | medium | The forbidden-type visitor examines function signatures only. FR-019 Behavior includes semantic input and result types, so a public result field of PathBuf, Mutex, or another forbidden type would evade the tagged inspection. Traverse exposed fields of the reachable public semantic types as well as signatures. | tests/it/fr019_api.rs:283-327 |

## Coverage

Merge base 14f875f: 527/602 backed; head: 533/602 backed. Both report 40 strict unbacked and zero contradicted. The normalized matrix has 568 criterion rows on both sides; only FR-019-AC-8 through AC-13 acquire new binders/status. Plan completion: not assessed. Optional repository-wide semantic review: not run.

## Verdict

CONDITIONAL. The decoder behavior and mutants are sound, but the three medium inspection gaps leave AC-8, AC-10 and AC-13 with incomplete tagged evidence.
