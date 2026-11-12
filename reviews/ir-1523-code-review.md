---
id: SR-20261112044523
title: "code-review and rust-review of IR-1523 public CatalogReadError::new"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #1017 at 09210a7c (base origin/main a156efda); crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for quire-contract-ir PR #1017. Ticket: IR-1523. The PR adds a documented public constructor `CatalogReadError::new(path: impl Into<String>, message: impl Into<String>) -> Self` to the existing public error type, next to the existing `path()` and `message()` accessors, plus one unit test of the constructor, the accessors and `Display`.

How it was checked:

- **Diff scope.** `git diff origin/main...HEAD` touches one file, +16 lines: the constructor and one test. No reader code changed, so the read path's behaviour is unchanged.
- **Public reachability.** `CatalogReadError` is re-exported from `checked_package::v2` (mod.rs:69) and from the `quire_contract_model` crate root (lib.rs:151). A `pub fn` on it is therefore reachable from a downstream crate.
- **Downstream fit.** quire-contract-codegen `src/oracle/claim.rs:154` declares `KaniReaderCatalogUnavailable(Box<quire_contract_model::CatalogReadError>)`, and quire-driver `src/refusal.rs` (head of PR #147) matches that variant. With this constructor, a driver test can build `OracleGenerationError::KaniReaderCatalogUnavailable(Box::new(CatalogReadError::new(..)))`, which meets the ticket's acceptance.
- **Focused build.** `cargo test -p quire-contract-model --lib catalog_read_error` passed (1 test). `cargo clippy -p quire-contract-model --all-targets -- -D warnings` was clean. Both ran through locked-build.sh.
- **Formatting.** `cargo fmt --all -- --check` reports one drift, in `monitor_plan/layout_context.rs:590`. That file is not in this PR's diff and is identical on origin/main, so the drift is on main and not from this PR.
- **Rust idioms.** The `impl Into<String>` arguments match the ticket's signature and the crate's other error constructors. The constructor is infallible and has no invariant to protect: the fields carry only diagnostic text, and the type is an error report rather than a validated value, so FR-019-AC-8's "no unchecked constructor" rule for validated values does not apply. The constructor has rustdoc. No unwrap, panic or `#[allow]` was added. `#[must_use]` is not used on the crate's other constructors, and the pedantic lint is not enabled.

## Examined scope

- operation_catalog.rs:277-284 `CatalogReadError::new` (examined): "/// A catalog read error for the member at `path`, failing with `message`. pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self"
- operation_catalog.rs:302-308 `catalog_read_error_new_carries_path_and_message` (examined): asserts `path()`, `message()` and `to_string() == "at `operations[3].kind`: unknown kind"`.
- operation_catalog.rs:259-275 struct, Display and Error impls (context_only).
- lib.rs:151 and checked_package/v2/mod.rs:69 re-exports (context_only).

## Verdict

PASS. The change is minimal and correct, it is the public surface the ticket asks for, and it does not change reading. The new test pins the constructor and the `Display` format. It carries no `#[trace]` tag, which is acceptable: no AC names this constructor. FR-019's public-items row for `checked_package` lists the type, not its methods.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
