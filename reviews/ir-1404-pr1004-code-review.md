---
id: SR-202611081512300006
title: IR-1404 PR 1004 code review
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-contract-ir; IR-1404 PR #1004 head 8ca9b6a1; crates/quire-contract-model/src/checked_package/v2 flat_wire.rs, operations.rs, protocol_body.rs, stage_storage.rs, state.rs; deny.toml'
review_set: subset
---

## Summary

Ticket: IR-1404. Reviewed `git diff origin/main...HEAD` with the Rust lane folded in. One PR-scoped focused test was run at the head to establish an observable failure. Coder-reported fmt, clippy, cargo-deny and focused-suite results were taken as reported and not repeated.

## Verdict

CONDITIONAL. The rewrite of the protocol-body, state and flat-wire sites it touched is sound: refusal identity is preserved, each new charge precedes its allocation and equals it, and Path capacity cannot overflow. One claim is too broad: the commit and PR body say flat-wire refusals are paid before being built, but two flat-wire refusal sites are still eager, and one of them was observed failing the physical accounting at this head.

Style note, not a finding: several rewritten calls in `flat_wire.rs` (around lines 331 and 345) and one in `protocol_body.rs` (around line 3103) exceed the 100-column limit, because rustfmt leaves those expressions unformatted without reporting drift.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The commit and PR body claim flat-wire refusals name a stack position and are charged before being built, but the interval-bound refusal and the node-header refusal in `flat_wire.rs` are still built eagerly and unpaid. A non-reduced lower bound with a negative upper refuses `invalid_package` at `/interval/upper` with 574 unpaid bytes (allocated 1172562, admitted 1171988). | crates/quire-contract-model/src/checked_package/v2/flat_wire.rs:602-612 |
| FND-002 | low | The Path capacity comment attributes the 13-step maximum to a capture initializer target, but initializers walk through `Trail`. The 13-step `Path` is a binder or control-reference target, such as `argument_path(2)/members/{entry}/members/0/value/target`. | crates/quire-contract-model/src/checked_package/v2/stage_storage.rs:44-47 |
| FND-003 | low | `refuse_detailed` sets the cause twice: `.because(cause)` and then `refusal.cause = Some(cause)`. The second assignment does nothing and suggests a difference that does not exist. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:44-46 |

## Scope

- FR-038-AC-511 (examined): every request is charged before it is made and the admitted delta equals the requested bytes. Traced `Refusal::bytes` against `CheckedPackageRefusal::new` (Box plus one `loci` element), the pointer, the node clone and `document_pointer`, and each `refuse_detailed` byte sum against its `CheckedProtocolRefusalDetail` variant. All variants match, including `CausalCycle` (`Vec` of exact length).
- FR-038-AC-512 (examined): refusal identity. Compared old and new code, cause, final pointer, locus and `document_pointer` at each rewritten site, including `boolean_at_node` (old primary-path rewrite equals the new direct form), `body_defect`, the merged `Body | EnumValue` arm and the `NegativeBound` arms. Evaluation order of `meter.formatted` before the refusal is unchanged. No difference found.
- FR-038-AC-510 (examined): a denied diagnostic returns the budget stop and builds nothing; `refuse_detailed` attaches detail only to a built refusal.
- Path capacity (examined): `argument_path` is 7 steps and the deepest fixed chain is 13. Every `Path` extension in the diff is a fixed chain; variable-depth targets use `Trail`. Overflow would be an index panic, but no input can reach it.
- `RefCell` in `state.rs` placement (examined): the `borrow_mut` in `enter` is not held across a walker reservation or the cursor charge.
- `#[allow(clippy::large_enum_variant)]` on `Position` and `OperationPath` (examined): justified, since boxing would be an uncharged heap request on the path being accounted.
- `deny.toml` (examined): required. `ar_archive_writer` 0.5.3 is licensed solely `Apache-2.0 WITH LLVM-exception`.
- `canonical_term_bytes` (examined): both encoder passes charge through `WriterCharges` before writer heap; a reservation failure is returned before an encode error is interpreted.
- The diff makes no claim that AC-511, AC-513 or other modules are closed. The PR body names the remaining `owner.rs` failure.

## Checks

- `cargo test --locked --test it <one test>`: exit 101, compile error, `crate::support::corpus` needs feature `conformance-cli`. No test ran.
- `cargo test --locked --all-features --test it tc_048_a_bound_not_in_lowest_terms_is_refused_at_the_bound_before_the_temporal_step`: exit 101, one test run, failed as recorded in FND-001.
- `cargo tree` license listing: exit 0.

## Dispositions

Round 1, checked against 77eb685a.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Both flat-wire sites now pay through `flat_refusal` / `flat_refusal_at_site` before building. Refusal identity holds: the header refusal keeps its code, member pointer and absence of cause and locus, and the interval refusal keeps code, cause, bound pointer and node locus. No eager refusal remains in the production part of `flat_wire.rs`. |
| FND-002 | fixed | The comment now names a binder or control-reference target and the 13-step path. |
| FND-003 | fixed | The duplicate assignment is removed; `.because(cause)` remains. |
