---
id: SR-2026102938
title: "Code and Rust review of V1 package digest removal"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@5afef435521b14806f3177cc33b9ee558728fd9f; PR #708 code and tests; IR-1141"
review_set: subset
---

## Summary

Ticket: IR-1141; PR #708 at frozen head 5afef435521b14806f3177cc33b9ee558728fd9f, stacked on #707 head 31b872ea0cfe4fc632bc29465ecbec3495cb047e. Reviewed the Rust producer, strict reader, read-only views, lowering API, changed tests, and callers of the removed public API. Static review only; the shared build lock was occupied.

## Verdict

**FAIL** — The named TC-447 QSL conformance target still calls the removed `ContractPackageV1::identity()` API and cannot compile when that target is selected. The five-field V1 producer/reader change and its changed integration controls otherwise agree in this diff.

## Examined units

- `FR-035-AC-2` (`examined`): source, meaning, dependency, and version mutations refuse before backend output.
- `FR-035-AC-3` (`examined`): one disposition per mixed request, with package ceiling behavior.
- `FR-035-AC-4` (`examined`): local and dependency references and source correspondence.
- `FR-035-AC-5` (`context_only`): withdrawn V1 digest requirement; no trace tag remains in changed tests.
- `FR-035-AC-12` (`examined`): five required top-level members and removed identity member refusal.
- `FR-049-AC-6` (`examined`): contextual lowering changes complete canonical V1 value and preserves read-only query behavior.
- `TC-447` (`examined`): named QSL producer conformance target.
- Changed source and test paths (`examined`): `crates/quire-contract-model/src/checked_package/v2/contract_package.rs`, its `guard_context.rs`, `result_context.rs`, and `scalar_operands.rs` children, `lower.rs`, `lower/ceiling_tests.rs`, and the six changed `tests/it/` files.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The TC-447 QSL conformance target still calls the deleted V1 `identity()` accessor, so its named Cargo target fails to compile. | tests/conformance_qsl_ir769/main.rs:197; crates/quire-contract-model/src/checked_package/v2/contract_package.rs:671 |

## Coverage

Code review and Rust review were folded into this artifact. The changed Rust sources remove the V1 content digest from the emitted wire, reader check, and three read-only views. Changed tests assert the five required members, canonical byte/value differences, and refusal of the removed `identity` field; existing source and dependency refusal checks remain. The untouched `tests/conformance_qsl_ir769/main.rs` target is declared in `Cargo.toml` and invoked by `make conformance-qsl-ir769`; line 197 calls a method deleted by this PR. No Cargo/build/CI gate was run because the code lead reserved the shared build lock for #704.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At #708 head 94e65cd3ee297031fbf0a2e98f9228c8c4e51b35, TC-447 encodes each admitted five-field V1 wire with `quire_canonical::to_vec` and compares complete canonical values; the removed accessor has no call left in the named target. The independent `[0, 9]` and `[0, 10]` assertions remain. |
