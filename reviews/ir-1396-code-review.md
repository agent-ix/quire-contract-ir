---
id: SR-20261107060894601
title: IR-1396 PR 946 carried_member code review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 946 head 186b6d2b; Cargo.lock, crates/quire-contract-model/src/checked_package/v2/{vocabulary.rs,operations.rs,operation_catalog.rs}"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1396. Code review with the rust-review lane folded in, over `git diff origin/main...HEAD` at 186b6d2b. The PR adds `OperationConstraintKind::CarriedMember`, a `check_operands` arm that refuses `ill_typed`/`operator-ineligible` unless the `type_argument` member is a `scalar_type`/`carried_quantity` node, and moves the QVC lock entry 5070338 -> 5769cb83. The lock move is one line; QVC 5070338..5769cb8 is one commit (QSL-1784) touching only the catalog JSON and its own test. The meaning matches QSpec FR-322 L3220 and L3278-3315. Focused tests pass (5/5). Mutating the arm to always admit fails the refusal test for each of `unit`, `bounded_quantity` and `integer` run singly, so the oracle is real.

## Verdict

FAIL: ticket AC2 is not met on the path that caused the defect, and the new refusal pre-empts the owner's earlier mode-type-mismatch refusal.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Ticket AC2 is not met. The defect IR-1396 names is the panic at `operation_catalog()` (operation_catalog.rs:278), and that panic is still there. The new test `tc_048_an_unknown_constraint_kind_on_the_carried_sum_is_a_typed_read_error` only proves again what FR-038-AC-65 already tests: `read_catalog` over supplied bytes returns `CatalogReadError`. That is a catalog read error, not a reader refusal. QVC is a `branch=main` git dependency, so how it fails: QVC main adds the next constraint, operator or member word, a consumer (CG) runs `cargo update -p quire-verification-contracts`, and the first V2 reader call panics at operation_catalog.rs:278. That is the same IR-1391 failure. Fix: make the catalog-dependent read return a typed refusal or error (for example cache a `Result` in the `OnceLock`), test it through injected bytes, and update FR-038-AC-58. Otherwise the lead re-scopes AC2 explicitly. | crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs:271-281 |
| FND-002 | medium | Wrong refusal order. `CarriedMember` is decided in `check_operands` (called at operations.rs:1313), before `check_mode_type` (operations.rs:1424). QSpec FR-322 L3300-3311 refuses a `rounding` mode the binder bound or `result_type` does not pin with `invalid_package`/`operation-mode-type-mismatch` first. It decides `carried_member` only "with the pinned mode agreeing". Probe: a `sum.carried_quantity` application with `type_argument` and `result_type` set to a `unit` node and mode `nearest-even` refuses `ill_typed`/`operator-ineligible` at /semantic_graph/nodes/0. With the arm disabled, the same application refuses `invalid_package`/`operation-mode-type-mismatch`. Fix: decide `carried_member` after `check_mode_type`, and add that ordering case. | crates/quire-contract-model/src/checked_package/v2/operations.rs:2309-2326 |
| FND-003 | low | The refusal test covers `unit`, `bounded_quantity` and `integer`. It does not cover the `compound_unit` type argument that QSpec FR-322 L3309 and FR-322-AC-391 name, or an application with no `type_argument` member (`member_declaration` returns None). | crates/quire-contract-model/src/checked_package/v2/operations.rs:6595 |

## Coverage

- Cargo.lock QVC move, examined: one line; upstream range 5070338..5769cb8 is QSL-1784 only (catalog JSON plus QVC test).
- vocabulary.rs `CarriedMember => "carried_member"`, examined: closed vocabulary, wire round trip tested.
- operations.rs CarriedMember arm, examined: matches the FR-322 L3220 meaning; mutation (always admit) kills the refusal test for each form.
- operation_catalog.rs new tests, examined: the production catalog reads; the unknown-word test duplicates the FR-038-AC-65 test.
- operation_catalog.rs `operation_catalog()` panic, examined (unchanged, the AC2 path).
- Rust lane: no unsafe, no new unwrap or expect in production code, exhaustive match kept (no wildcard), focused tests only; no Kani or aggregate gate run, as the brief says.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | The fix for FND-001 turns a reader build defect into a false refusal of the producer's package. `resolve_catalog` maps any `CatalogReadError` to `EMPTY_CATALOG` and discards the error. A valid package from QSL is then refused `invalid_package`/`unknown-operation` at its first application's `/body/operation/identity`. The cause names the producer's package as defective, and the unreadable word and path (for example `operations[150].constraints[1].kind` `carried_member`) are not reported anywhere. How it fails: QVC main adds a new word and CG runs `cargo update`. Every genuine package is now refused as invalid with an unknown operation, CG records a refusal verdict against packages that are correct, and nothing points at the lock skew. The old panic at least named the word. This also contradicts FR-038 L964-966 and FR-038-AC-58's own first sentence, which say a catalog read failure is "never a package refusal". The correct outcome is a third read result beside `Refused` and `Incomplete`, for example a `ValidationFailure`/public read-error variant `CatalogUnavailable` that carries the `CatalogReadError` path and message. It is returned before any node is checked, from every entry point that reads through `operation_catalog()`. That is an IR-local reader error, not a package refusal, so it needs no QSpec code or cause. Only mapping it onto a refusal code would need QSpec. | crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs:271-297 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | f2f0ea96 removes the panic: `OnceLock<Result<OperationCatalog, CatalogReadError>>`, and `operation_catalog()` no longer panics; tested through real `read_catalog` and `resolve_catalog`. The replacement outcome is wrong and is tracked as FND-004. |
| FND-002 | fixed | `check_carried_member` now runs after `check_mode_type` (operations.rs:1424-1429). The new test `carried_member_is_checked_only_once_the_pinned_mode_agrees` asserts operation-mode-type-mismatch at /body/operation/mode/value, the same input that refused operator-ineligible in my round-0 probe. |
| FND-003 | fixed | `compound_unit` added. The dropped cases cannot reach `carried_member` on this entry: an `integer` type_argument refuses at /body/result_type in the `member_family` check of `check_type_argument_member_result` (operations.rs:1335, round-0 mutation run), and a missing member refuses there too (`member_declaration` returns None, then refuse). Both run before `check_mode_type` and `check_carried_member`. |
