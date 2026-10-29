---
id: SR-11400
title: "IR-800 PR 405 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; Cargo.lock, crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/identity.rs, crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, crates/quire-contract-model/src/checked_package/v2/structural.rs, crates/quire-contract-model/src/checked_package/v2/vocabulary.rs, tests/it/checked_package_v2_model_fields.rs, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_operand_families.rs"
review_set: subset
---

## Summary

Ticket IR-800, PR quire-contract-ir#405. Code review with the rust-review lane folded in, scoped to `git diff origin/main...`. Examined the native String, UUID and Timestamp forms across the V2 vocabularies, structural/derived-key/identity/lowering tables, model member mapping, the literal payload pass and the text-profile String guard in `operations.rs`, the three test files, and the single `quire-verification-contracts` Cargo.lock entry ( to, whose only catalog change adds the three families to `families`, `any_value` and `any_term` and the six `eq`/`ne` entries). Criteria examined: FR-038-AC-242, FR-038-AC-243, FR-038-AC-244, FR-038-AC-245.

## Verdict

**PASS** with three low findings. String stays a distinct `scalar_type`/`string` form and `CheckedMemberType::String`; it is not `text`, needs no bound or profile (`requires_bound` false), and a String operand in a text-profile operation is refused. Every refusal code, cause, pointer and locus named in AC-243..245 matches the implementation, and the tests assert all four. UUID canonical form is exactly 36 bytes, lowercase hex with hyphens at 8/13/18/23, so nil and max admit. Timestamp is parsed as `i128` and must re-render byte-identically, which refuses `-0`, `+1`, leading zeros, whitespace and out-of-range values. String keeps the exact JSON string with no normalization. The new literal pass adds no new allocation that outgrows the already-walked term stack, adds no integer cast, no `unwrap`, and no panic on untrusted input; the one `unreachable!` can only be hit by an internal invariant break. Focused gates run in a throwaway worktree: `cargo test --test it` over the operand_families, model_fields and model_members modules (73 passed), `cargo test -p quire-contract-model --lib` (212 passed) and `cargo clippy -p quire-contract-model --all-targets` (clean).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The text-profile String guard runs before the arity and per-argument family checks, so it overrides earlier operand-step refusals. This precedence is not specified and not tested | crates/quire-contract-model/src/checked_package/v2/operations.rs:712 |
| FND-002 | low | The String guard resolves the family of every wire argument before arity is checked. This adds an unmetered walk of the bounded-domain chain for each argument | crates/quire-contract-model/src/checked_package/v2/operations.rs:714 |
| FND-003 | low | The literal payload pass uses `unreachable!` for an invariant that construction keeps but the type does not enforce | crates/quire-contract-model/src/checked_package/v2/operations.rs:408 |

## Checks

- FND-001: without the guard, `quire.op.text.eq(integer, string)` would fail the operand step with `ill_typed`/`operator-ineligible` at `/arguments/0`. It now refuses `invalid_package`/`operation-mode-type-mismatch` at `/operation/mode/value`, because the guard sees the String argument at position 1 first. A three-argument `text.eq` that includes a String refuses the same way, not with the arity refusal at `/arguments`. AC-244 names only the single-defect outcome. Either specify that the guard takes precedence over the operand step, or move the guard after the arity check and run it over `arguments[..entry.operands.len]`.
- FND-002: before this change, a fixed-arity text-profile entry resolved at most two argument families, and only after arity passed. The guard now runs `operand_type_node` and `resolve_family` over all N wire arguments, without charging the meter. `resolve_family` follows bounded-domain chains with no length limit, so the unmetered work is N times L, bounded only by the byte and node ceilings. The variadic `rest` loop in `check_operands` already does the same, which is why this is low. Fix: run the guard after arity, as in FND-001, or charge the meter for the walk.
- FND-003: `literal_defect` pushes only String, UUID and Timestamp kinds into `native`, then matches on `LiteralKind` again with an `unreachable!` arm. If the push filter and the match ever drift apart, the reader panics instead of refusing. Record a closed `NativeLiteral { String, Uuid, Timestamp }` value, or the validator itself, when pushing, so the second match is exhaustive and cannot panic. `lossless.rs` already uses `unreachable!` in a similar way, so this is a style point, not a reachable defect.
- Cargo.lock: the only change is the `quire-verification-contracts` source line.
- Wire-boundary integers: the Timestamp payload stays a string; it is checked with `parse::<i128>` and never cast. No `as` conversions were added.
- Refusal precedence for literals: a literal's type is checked before its payload (test `native_literal_payloads_are_canonical_after_type_validation`). Work-limit precedence (Incomplete before the payload defect) is retained (test `native_literal_work_limit_precedes_later_payload_defect`).

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed |: the String mode refusal moved into check_operands, after the arity check and the per-position family loop; a different-family operand or an arity defect now wins (test text_comparison_checks_arity_and_other_families_before_string_mode) |
| FND-002 | fixed |: the separate pre-arity walk is gone; the String flag reuses the argument_family lookups that check_operands already makes, after arity |
| FND-003 | fixed |: validity is computed with an exhaustive match while walking the terms (non-native kinds give None), and the deferred loop only reads (bool, path); no unreachable! arm remains |
