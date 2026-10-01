---
id: SR-633
title: "rust-review design audit of quire-contract-ir and quire-contract-model"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@8371caa; src/**, crates/quire-contract-model/src/**, tests/it/**, Cargo.toml, Cargo.lock, deny.toml, Makefile, .github/workflows/ci.yml, spec/assurance/AD-001, spec/interface/FR-019, spec/interface/FR-039, spec/contract/FR-028, spec/contract/FR-030, spec/contract/STD-001"
review_set: base
---
# SR-633: rust-review design audit of quire-contract-ir and quire-contract-model

## Summary

Ticket: IR-317. This is a whole-repo design audit using the rust-review method, with the rust-review lane folded into code-review. It is not a PR diff. Reviewed at origin/main 8371caa, after #233. It re-measures every item of the prior audit comment on IR-317, which was made at 69cd1cb, and adds new findings.

Gates the reviewer ran at 8371caa, with CARGO_TARGET_DIR set to a directory inside the audit worktree:

- `make lint`: exit 0. Both lanes ran: the workspace lane, and the model-only lane with the feature off.
- `make test`: exit 0. 170, 58 and 2 tests passed, plus the model doctests.
- `make deny`: exit 0. cargo-deny reported advisories, bans, licenses and sources ok, and the one-copy awk check passed.

The reviewer also ran two reproductions in a scratch crate outside the repo:

- A balanced multiplication tree of a `-1..1` integer guarded by `x != 0`. At 16 leaves it takes 3.7 ms. At 32 leaves (63 nodes) it aborts the process with `memory allocation of 4294967296 bytes failed`.
- serde_json 1.0.151's error for a closed enum given the string `"unknown field"`. It reads ``unknown variant `unknown field`, expected `alpha` ``.

Re-measurement of the prior audit (69cd1cb):

- H1 (two model copies): fixed by #225. Cargo.lock has one `quire-contract-model`, with no git source. `check_one_copy.awk` exits 0. The codegen lock also has one copy, at 54f9a48. What remains is CI (FND-009).
- H2 (bridge codec): fixed by #210. `src/bridge` was deleted.
- M1: the QSL runtime import is fixed by #225 and #233 (no QSL dependency, `replay.rs` deleted). The lowerings in the root crate are still live (FND-004).
- M2: still live (FND-005, FND-012).
- M3: the `common.rs` part is still live (FND-006). The predicate part is fixed by #210.
- M4: the `v2/mod.rs` `expect`s are gone. The `expression.rs` `unreachable!()` calls are still live (FND-011, tracked as IR-360).
- M5: the predicate and temporal parts are fixed by #210. The only hard-coded digest left is in `#[cfg(test)]` (operations.rs:1922).
- M6: still live (FND-002, FND-003).
- M7: still live (FND-015).
- L1: still live. expression.rs is 3886 lines, v2/mod.rs 2149 and operations.rs 3103 (it grew).
- L2: still live (FND-014).
- L3: still live for `forbid(unsafe_code)`, the `//!` headers, the double depth pre-scan and the 128 cap (IR-452). The bare `as u32` casts in wire.rs are fixed: none remain.
- L4: wrong as a finding. Every `allow(dead_code)` carries a reason.
- L5: still live, and there are now three tag forms (FND-016). The kani_replay tests were deleted (#233).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Integer interval sets grow without bound. A `NonZero` fact splits a range in two, and `integer_ranges` takes the full cross product with no merging or cap. So a 63-node expression `x != 0 && (x*x*...*x) > 0` over `x: -1..1` builds 2^32 intervals and aborts the process. That is not a catchable panic, and it is far inside MAX_EXPRESSION_NODES and depth. Reachable from untrusted expression and package input | crates/quire-contract-model/src/expression.rs:2464-2502, crates/quire-contract-model/src/expression.rs:3216-3236 |
| FND-002 | medium | The root crate glob-re-exports the whole model (`pub use quire_contract_model::*`), which AD-001, FR-028 and FR-039 forbid. Codegen imports model types through `quire_contract_ir` (harness.rs:5, kani_witness_join.rs:159). So every model item has two paths, and an IR pin and a model pin at different revs give two non-identical types | src/lib.rs:12 |
| FND-003 | medium | The model crate root has 14 glob re-exports (7 in lib.rs, 4 in checked_package, 3 in v2), against FR-019-AC-5, and TC-058 has no test. Today all 169 glob-reachable top-level items appear in the FR-019 table, by a reviewer script. But a new `pub` item joins the stable surface silently, and nothing fails | crates/quire-contract-model/src/lib.rs:31-47, crates/quire-contract-model/src/checked_package/mod.rs:18-21, crates/quire-contract-model/src/checked_package/v2/mod.rs:22-24 |
| FND-004 | medium | The family lowerings (arithmetic, collections, objects) and `KaniProviderRecord`/`KaniProviderResult` are still exported from the root crate. FR-039 lists them under "Items codegen owns" and "Items QSL owns", and AD-001 places the lowerings in codegen. Codegen still consumes them (bounded_kani_corpus.rs) | src/kani/mod.rs:19-23 |
| FND-005 | medium | `KaniOutcome` has public fields and a derived `Deserialize`, and its codes are free strings. The code emits 21 distinct `kani_*` codes, and STD-001 registers none of them. STD-001 registers `kani_outcome_invalid`, which nothing emits; the non-success constructor emits an unregistered `kani_outcome_kind_invalid` as a Refused outcome instead. FR-039 lists a `KaniOutcomeError` that does not exist. A consumer matching the registered code never matches, and `KaniOutcome{kind: Proved, ..}` or an `Unavailable` with any cause is constructible | src/kani/outcome.rs:92-102, src/kani/outcome.rs:161-176 |
| FND-006 | medium | `decode_closed` picks UnknownMember versus MalformedWire by substring-matching the serde error message. A closed-vocabulary string value equal to `unknown field` yields ``unknown variant `unknown field` `` (measured), so a wrongly typed value refuses as `unknown_member` | crates/quire-contract-model/src/checked_package/common.rs:326 |
| FND-007 | medium | The strict model-document reader parses with serde_json's default 128 nesting cap, and maps every parse failure, including a duplicate member, to ByteDigestMismatch. A depth-129 document whose digest matches is refused as a stale digest | crates/quire-contract-model/src/checked_package/v2/model_members.rs:885, crates/quire-contract-model/src/checked_package/common.rs:911-916 |
| FND-008 | medium | `KaniProfile`, `ProfileSelection`, `FiniteInput` and `CapabilityEntry` derive `Deserialize` with public fields and no `deny_unknown_fields`. Serde or a struct literal bypasses `KaniProfile::new`, so a profile with family `other` or duplicate constructs is built, and `classify` then uses the first duplicate. This breaks FR-039's "every conversion from untrusted input is fallible" | src/kani/profile.rs:23-50, src/kani/profile.rs:71-95, src/kani/abi.rs:54-71 |
| FND-009 | medium | CI is narrower than the make gates. ci.yml runs only workspace clippy, so the model is never linted with `fault-injection` off (the `cfg(not(feature))` code at output_mapping.rs:1191). It runs `cargo deny check`, not `make deny`, so the one-copy awk gate never runs in CI. With `multiple-versions = allow` in deny.toml, nothing in CI catches a second model copy (still live from SR-623 FND-001) | .github/workflows/ci.yml:26, .github/workflows/ci.yml:54, Makefile:47-52 |
| FND-010 | low | `validate_operations` walks each body with `at.clone().key(..).index(..)` at every level, which is quadratic in depth (IR-454). It is now bounded by MAXIMUM_DEPTH 16384, and the clone recursion is fixed (on_stack_for in `Clone`). Not re-timed | crates/quire-contract-model/src/checked_package/v2/dependency_references.rs:213, crates/quire-contract-model/src/checked_package/shared.rs:43 |
| FND-011 | low | Four `unreachable!()` calls rely on the unstated invariant "Integer type implies an Integer range". The reviewer traced the producers and found none that breaks it, but the type does not encode it | crates/quire-contract-model/src/expression.rs:2385, crates/quire-contract-model/src/expression.rs:2388, crates/quire-contract-model/src/expression.rs:2521, crates/quire-contract-model/src/expression.rs:2541 |
| FND-012 | low | The kani lowerings repeat a 40-line dispatch preamble three times. `_input` is unused in two of them. `CheckedArithmeticRequest.source_id` is `&'static str`, so a runtime id must be leaked. `classify` puts the construct in the context slot | src/kani/arithmetic.rs:14, src/kani/arithmetic.rs:44-84, src/kani/collections.rs:38-79, src/kani/objects.rs:41-81, src/kani/profile.rs:118 |
| FND-013 | low | The model crate has no `#![forbid(unsafe_code)]`, which the root crate has. 7 modules lack a `//!` header, and there is no `missing_docs` lint: public fns in expression.rs have no `///` | crates/quire-contract-model/src/lib.rs:1, crates/quire-contract-model/src/expression.rs:20 |
| FND-014 | low | `Diagnostic`, `RunnerError` and `CheckedPackageRefusal` do not implement `std::error::Error`. Only `MappingRequestError` does, and the root crate uses thiserror. Callers cannot `?` model errors into a boxed error | crates/quire-contract-model/src/identity.rs:239, crates/quire-contract-model/src/output_mapping.rs:261 |
| FND-015 | low | The conformance runner lives in the model crate: fs::read_dir and fs::read, a worker thread, and `include_str!` of `../../../schemas` from outside the crate. AD-001 says the runner stays in the root package. FR-019 says no filesystem path appears in the API, yet lists `run_corpus(&Path, &Path)`. `validate_schema` also recompiles three schemas on every call | crates/quire-contract-model/src/conformance.rs:498, crates/quire-contract-model/src/conformance.rs:669, crates/quire-contract-model/src/conformance.rs:1036, crates/quire-contract-model/src/binding.rs:23, crates/quire-contract-model/src/binding.rs:399-415 |
| FND-016 | low | Tests use three trace-tag forms: `/// Tracing:`, bare `/// TC-035` and `#[trace]`. 40 of 228 `#[test]` fns carry none of them, and 27 are not named `tc_`. 24 of those are in operations.rs and 13 are the bare form in executable_binding.rs. kani_shared.rs:177 says no AC covers the vacuous proof, but FR-030-AC-4 now does | crates/quire-contract-model/src/checked_package/v2/operations.rs:1719, tests/it/executable_binding.rs:151, tests/it/kani_shared.rs:177 |
| FND-017 | low | Helpers are duplicated: rational gcd normalisation appears three times, and there are two JSON depth scanners with different semantics (lexical u32 versus the strict grammar) | crates/quire-contract-model/src/expression.rs:1011, crates/quire-contract-model/src/expression.rs:1698, crates/quire-contract-model/src/expression.rs:3605, crates/quire-contract-model/src/limits.rs:10, crates/quire-contract-model/src/checked_package/common.rs:992 |
| FND-018 | low | `MappingCandidate::new` takes 11 positional arguments and `admit` takes 8, both under `allow(too_many_arguments)`. Same-typed `Vec` axes (`conditions`, `causes`, `local_regions`) can be swapped at a call site | crates/quire-contract-model/src/output_mapping.rs:1348, crates/quire-contract-model/src/output_mapping.rs:2101 |
| FND-019 | low | AD-001 and FR-039 cite "Linear IR-358" as tracking the KaniOutcome-to-TerminalValue map in codegen. IR-358 is actually the two-model-copies ticket | spec/assurance/AD-001-contract-ir-architecture.md:99, spec/assurance/AD-001-contract-ir-architecture.md:171, spec/interface/FR-039-root-crate-public-interface.md:74 |

## Verdict

The design is sound in the places the prior audit flagged as wrong results. The bridge codec and the QSL coupling are gone, the dependency graph has one model copy, and the checked-package reader now runs on a stack sized to the admitted depth and drops values iteratively.

One new high defect needs a fix: FND-001, an unbounded interval blow-up that aborts the process on a small untrusted expression.

The remaining medium findings are the layout gaps the specs already mark as planned:

- the root and model glob re-exports;
- lowerings and provider types still in the root crate;
- the `KaniOutcome` invariants;
- serde bypassing the profile constructor.

Two real defects sit alongside them: message-derived refusal codes, and the 128 cap. CI is also narrower than the make gates.

Areas checked and found clean:

- Integer casts at the wire boundary (all widening or bounded).
- No `unsafe` anywhere.
- Every `allow` carries a reason.
- Output mapping and coverage are bounded by MAX_SEMANTIC_COLLECTION_ITEMS and the work budgets.
- The checked-package `Clone` and `Drop` are stack-safe.

Not verified: IR-454 timings at the 16384 ceiling. That every integer-typed `Checked` producer sets a range (FND-011 was traced by reading, not by test). Whether the FND-001 abort is also reachable through a checked-package/v2 lowering path; it was reproduced through `DeclarationEnvironment::check_expression`. Whether `quire coverage` accepts the bare `/// TC-NNN` form; `make spec` was not run.
