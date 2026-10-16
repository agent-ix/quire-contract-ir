---
id: SR-11760
title: "Code and Rust review — IR-795 four-owner source-inventory port"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs (PR #413 at 77284f7)"
review_set: subset
---

## Summary

Ticket: IR-795 (IR-759 Slice B). Reviewed PR #413 at 77284f7 with `git diff origin/main...HEAD`, the code-review method with its rust-review lane in this file. The port adds `SourceInventoryOwner::Quire`, `SourceInventoryScope::QuireSelectedAuthoredCriteria`, `SourceAuthoredCriterionId`, `SourceCriterionSourceBinding`, and a Quire envelope position plus a binding on `SourceObligationInput`. It replaces `Criterion(ClauseRef)` with `Criterion(SourceAuthoredCriterionId)` and moves that variant's owner from Qsl to Quire.

Checks run on the worktree with `CARGO_TARGET_DIR=<worktree>/target`, `-j2`, and the `/tmp/luna-ir-heavy.lock` flock. `cargo test --test it source_obligation` passed 7/7. `cargo test --test it public_interface` passed 4/4, so the FR-019 inventory matches the compiled surface. The model doctests for `source_obligation` passed 6/6. `cargo clippy -p quire-contract-model -p quire-contract-ir --tests -D warnings` is clean, and so is `rustfmt --check` on both changed files.

Mutation probes ran in a separate detached scratch worktree, never on the PR branch. Seven were killed: M1 criterion owner changed to Qsl, M2a Qsl scope accepted for the Quire owner, M2b Quire scope mapped to Qsl, M2c owner/scope check dropped, M3 binding dropped by `SourceObligationInput::new`, M4 row-owner rejection dropped, and M5 Quire envelope dropped. One survived: M6 added `impl From<ClauseRef> for SourceAuthoredCriterionId`, and the `compile_fail` doctests stayed green (FND-001).

## Examined scope

- FR-052-AC-14 (examined): A default-feature neutral consumer distinguishes a typed Quire criterion identity containing obligation declaration, scoped document and authored criterion ID from `ClauseRef` and QSL `RequirementRecord`. Two neutral fixture rows with distinct authored criterion identities, one without a correspondence assertion, remain distinct Quire-owned rows before any generation or result. A formal clause, QSL requirement record or assertion-only key cannot substitute for either fixture criterion. This neutral fixture does not claim Quire owner-issued completeness.
- FR-052-AC-15 (examined): An explicit source-author binding connects the selected Quire corpus/obligation scope to the selected IR `SourceIdentity`. A missing, foreign or conflicting binding refuses a complete census with distinct typed causes; a matching label or revision alone is not treated as proof of equal source bytes across Quire, IR V2 and QSL. Foreign and dangling AC-to-occurrence/function assertions retain their separate typed dispositions.
- FR-052-AC-12 (examined): A synthetic `SourceInventoryEnumerator` producing zero or multiple `Ok(Some(row))` events yields a neutral `Complete` outcome only after its `Ok(None)` terminal event. Replacing that terminal event with `Err(typed_cause)` yields `Incomplete(typed_cause)` and retains exactly the preceding rows; the typed cause is equal to the emitted error.
- FR-052-AC-13 (examined): For a synthetic enumerator, the neutral outcome keeps its retained rows and count private. Mismatched observed count, owner/scope pair, row owner, row source, or stop-cause owner returns the corresponding typed structural error and no outcome.
- FR-052 statement, owner/scope pairing (spec/model/functional/FR-052-source-obligation-correspondence.md:89-99, 128-140, 275-279) (examined): `SourceInventoryOwner` shall distinguish `Quire`, `IrV2`, `Qsl`, and `QuireCodeRs`; `SourceInventoryScope` shall distinguish `QuireSelectedAuthoredCriteria` ...; each owner shall supply exactly its corresponding scope; a `SourceInventoryIdentity::Criterion` row shall have owner `Quire`.
- FR-052-AC-16 (context_only): IR-812 / PLAT-1165 Quire producer; out of scope. No Quire producer or PLAT-1165 claim was added.

## Verdict

CHANGES REQUESTED (medium). There is no critical or high defect.

- Exhaustive matches: `SourceInventoryScope::owner` (source_obligation.rs:44-51), `SourceInventoryIdentity::owner` (:385-394), `SourceInventoryIncompleteReason::owner` (:112-124) and the test `TestEnumerator::new` scope match have no wildcard arm.
- `ClauseRef`: it is gone from `source_obligation` except inside the negative doctest. It has no dual variant and no `From`/`Into` conversion, and the authored id is only built from three strings.
- Owner/scope enforcement: the `from_enumerator` owner/scope, row-owner, row-source and stop-owner checks cover the Quire owner. `quire_outcome_rejects_other_owner_scope_and_rows` tests them, and the mutations confirm it.
- Binding: `SourceCriterionSourceBinding` is retained in the input. Its census refusal is Slice C, as FR-052:262-273 specifies at census level.
- Module doc and comments say four-owner throughout. No census, builder or IR V2 adapter leaked in.

The weak point is the AC-14 "cannot substitute" oracle (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-14 negative doctests are `compile_fail` with no error code and probe only implicit coercion (`let _: SourceAuthoredCriterionId = clause;`). Adding `impl From<ClauseRef> for SourceAuthoredCriterionId` leaves them green (mutation M6 survived), and an unrelated import or syntax error would also pass them. Pin `compile_fail,E0308` and add `SourceAuthoredCriterionId::from(clause)` / `.into()` probes pinned to E0277, for ClauseRef and SourceRequirementOccurrenceId. | crates/quire-contract-model/src/source_obligation.rs:310-324 |
| FND-002 | low | `SourceObligationInput::new` now takes four adjacent positional `Option<SourceInventoryEnvelope>` arguments (quire, ir_v2, qsl, quire_code_rs), and nothing checks the envelope owner against its position. A swapped Quire/IrV2 envelope compiles silently. Consider asserting `envelope.owner()` per position, or using a per-owner typed slot. | crates/quire-contract-model/src/source_obligation.rs:866-884 |
