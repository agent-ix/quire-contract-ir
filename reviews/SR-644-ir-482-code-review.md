---
id: SR-644
title: "code review (with rust-review) of PR 237 (IR-482 ordered_enum operand family)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@9f8699e5736918f3dccf639148297e4d835d2750; crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_enum_order.rs, tests/it/main.rs"
review_set: base
---
# SR-644: code review (with rust-review) of PR 237

## Summary

Ticket: IR-482. Diff is `origin/main...HEAD` over IR main b995245, one commit 9f8699e. The fix adds an enum branch to `resolve_family`: a `scalar_type`/`enum` node whose `nominal_identity_preimage` is an `EnumDeclaration` with `ordered == true` resolves to `ordered_enum`, otherwise `enum`. This matches QSpec FR-322 "Operation families" (an enum is `ordered_enum` when its nominal preimage is ordered and `enum` otherwise) and the catalog at quire-verification-contracts ead78f3 (`enum.lt/le/gt/ge` operands `ordered_enum` + `same_type`; `enum.eq/ne` operands `enum_kind` + `same_type`).

Every family decision goes through `resolve_family`. `operand_family` is called only from `resolve_family` and its own table test. The callers are `argument_family` (reference, literal and application terms, through `operand_type_node`), `check_mode_type` and `check_inner_result`. That covers member literals, parameters, expression and field-projection results (they use `result_type` or the target's `semantic_type`), and dispatch results. The branch runs at every level of the `bounded_domain` recursion, so an ordered enum under a `bounded_domain` still resolves to `ordered_enum`. Option and sequence wrappers keep their own family, which is correct.

Over-admission: `NominalKind::required_by` makes an `EnumDeclaration` preimage mandatory on every enum `scalar_type` node. `ordered` is part of the re-derived node key, so it cannot be flipped without changing the key. A missing or other preimage falls back to `enum`, the conservative choice. A dependency or foreign type id is not in the local index and resolves to `None`, the same as on main.

Probes and mutations, all run at head with `CARGO_TARGET_DIR` set to the worktree:
- Real QSL output (a486e555) read through the head reader in a scratch harness: `ordered_enum_lt_literals.json` and `ordered_enum_lt_params_located.json` now both read Admitted; on main efa5632 both were IllTyped/OperatorIneligible at arguments/0. `int_record_eq.json` is still refused at `/operation/leaves`; that is IR-483.
- Always `ordered_enum`: `tc_048_ordering_over_an_unordered_enum_refuses` fails. Never `ordered_enum`: the admit test and the cross-enum test fail. Inverted flag: three tests fail. `None`: the unordered test fails.
- Inferring "ordered" from member order (members not sorted): every test passes (FND-001).

Rust-review lane: the `matches!` with a guard and the early return are idiomatic. No `unwrap`, panic, `unsafe`, integer conversion or new allocation. The import is added to the existing `super::` use list. The `operand_family` doc note is accurate. Tests carry `#[trace("TC-048", "FR-038-AC-42")]` with matching `Tracing:` doc lines, assert the full refusal (code, pointer, cause, locus) by equality, and build packages in the QSL emitter's shape: `reference` arguments to `enum_value` and `parameter` nodes, the same shape as the real emission.

Gates at head: `make fmt-check lint test corpus` exit 0 and `make deny` exit 0 (advisories, bans, licenses, sources ok).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The test fixtures tie `ordered` to member order: the ordered enum is always `[READY, DONE]` (not sorted) and the unordered one `[DONE, READY]` (sorted). A mutant that decides ordering from "members are not sorted" instead of the `ordered` flag passes every test. An ordered enum whose declared order happens to be sorted is never exercised | tests/it/checked_package_v2_enum_order.rs:44-61 |

## Finding Detail

- FND-001: the production code reads the flag correctly. Only the tests are too weak. Fix: add one ordered declaration whose members are sorted (for example `ordered: true, members: [DONE, READY]`) to `tc_048_ordering_over_an_ordered_enum_admits`. Then the member-order mutant fails.

## Verdict

Correct, minimal, and matches FR-322 and the catalog exactly. One low test-strength finding. Mergeable once FND-001 is fixed or dispositioned.
