---
id: SR-20261103150645
title: "code-review and rust-review of QSL-1538 channel delivery interval (IR)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #855 at d4d4876a (base origin/main 6c556959); crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, crates/quire-contract-model/src/checked_package/v2/refinement.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for quire-contract-ir PR #855. Ticket: QSL-1538. The PR adds a module-private `ChannelDelivery` enum (`Unknown`, `AtMostOnce`, `AtLeastOnce`, `ExactlyOncePremise`, `Between { lo, hi }`) with one `#[string_edge]` `parse`, and uses it at all three sites that used to hard-code the four policy words: `local_details_fit` and `enum_value_defect` in protocol_body.rs, and `protocol_member_classes` in refinement.rs.

How it was checked. No build was run: `/` had 4.8G free, under the 15G threshold, so this is a read-only review. These checks substitute for a build:

- **Grammar.** `parse` and `canonical_natural` were replicated byte for byte in a scratch file, compiled with rustc 1.98.1 (the crate's `rust-version`) and run against boundary inputs. Accepted: `[0,0]`, `[0,18446744073709551615]`, `[18446744073709551615,18446744073709551615]`. Refused: `[0,18446744073709551616]`, `[01,2]`, `[1, 2]`, `[1,2]x`, `[ ]`, `[]`, `[,]`, `[unknown,1]`, `[1,2],[3,4]`, `[1,2]]`, `[[1,2]`, an Arabic-Indic digit, `[0x1,2]`, `[1_0,20]`, `[+1,2]`, `[1,2\n]`, `[1,2] `, `UNKNOWN` and `[2,1]`.
- **Mutation spot checks.** The same replica was run against the PR's unit-test spelling lists. Every mutant was killed: dropping `lo <= hi`, dropping the leading-zero check, dropping the digits-only check (only `+` gets past `u64::from_str`), using `lo < hi`, and refusing a lone `0`.
- **Refinement mutants.** Giving `Between` a `duplicate` or `lose` class, or admitting `[2,1]`, fails `channel_classes_follow_delivery_and_an_interval_has_none`.
- **Wire-step mutant.** Making `enum_value_defect` never fire for delivery falls through to `validate_layout`, which returns `ProtocolLayoutWireDefect::Body` with a different locus. So the integration test's `/body/arguments/4/value` pointer kills that mutant.
- **Formatting and lints.** `rustfmt --check` on all three files is clean. clippy-driver on the replica is clean except for the pedantic `must_use_candidate` lint. That lint is not enabled in this workspace and does not apply to a `pub(super)` fn. The enum's `lo`/`hi` are never read outside tests, but rustc 1.98.1 raises no `dead_code` warning for that pattern (checked on a replica).
- **String edge.** The `qsl-string-edge` scanner flags only string-literal or `&str`-const operands. `canonical_natural` uses only char and byte predicates, so it needs no marker, and `parse` carries one. The scanner is not installed locally; CI `Rust Checks` installs it and runs `make lint`.
- **Panic hygiene.** No unwrap, expect, panic or indexing in production code. `expect` appears only in tests. No `#[allow]` added.

## Examined scope

- protocol_body.rs:417-462 `ChannelDelivery`, `parse`, `canonical_natural` (examined): "An interval is canonical: no space, decimal naturals with no sign and no leading zero, at most `u64::MAX`, and `lo <= hi`. Any other text is `None`."
- protocol_body.rs:776-791 `local_details_fit` channel arm (examined): "delivery.and_then(ChannelDelivery::parse).is_some() && ..."
- protocol_body.rs:1114-1124 `enum_value_defect` channel arm (examined): returns `EnumValue { argument: 4, member: None }` for a text delivery that does not parse, matching the previous `invalid_enum_at(arguments, 4, None, ...)` behaviour for text and non-text terms.
- protocol_body.rs:969-1005 `wire_layout_defect` ordering (context_only): `enum_value_defect` runs before `validate_layout`, so a bad delivery refuses at the delivery term.
- refinement.rs:3529-3542 `protocol_member_classes` channel arm (examined): `Between { .. }` gives no class, same as `ExactlyOncePremise`.
- protocol_body.rs:5775-5829 unit test, refinement.rs:3925-3971 unit test, tests/it/checked_package_v2_protocol_body.rs:8094-8160 integration test (examined).

## Verdict

PASS. The change is small, typed and correct. A single enum now owns the delivery vocabulary, replacing three string copies. The canonical grammar is exact and the error locus is unchanged. Every proposed mutant is killed by the new tests. `ChannelDelivery` is `pub(super)`, so it adds no public API, and `#[non_exhaustive]` does not apply. QSL depends on quire-contract-model at `branch = "main"` but does not use this type. Rejected inputs are now a strict subset of what was rejected before, and nothing on QSL origin/main f385c801 depends on IR refusing an interval delivery. The QSL build and tests are therefore unaffected.

This PR does not cover one cross-repo matter: QSL origin/main `qsl-package/src/protocol_view.rs::delivery` (around line 832) trims spaces and accepts a sign, a leading zero and `lo > hi`. Its test asserts that `[ 2 , 2 ]` is admitted, which contradicts the FR-465 grammar this PR implements. That code belongs to QSL and is reported to the dispatcher. The full `make ci` gate was not run because of disk space.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
