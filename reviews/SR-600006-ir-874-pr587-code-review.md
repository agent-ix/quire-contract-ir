---
id: SR-600006
title: "IR-874 PR 587 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #587; crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Reviewed the frozen binder and slot-scope diff against QSpec FR-464/466 and IR FR-038-AC-291/292. Hosted Rust Checks fail in the changed Rust source, and the terminal scope admits a forbidden read under `each` activation.

## Verdict

**FAIL** — the PR does not compile at its frozen head and one binder scope rule admits an invalid package.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Hosted `make lint` fails: `known` needs an explicit map value type (E0282), and the C0 index comparison triggers denied Clippy `int_plus_one`; no tests ran. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3399 |
| FND-002 | high | Terminal scope always includes `over`; under `each`, a terminal condition that reads `over` is admitted although the stage permits no binder. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:4013 |

## Scope

- **FR-038-AC-291 (examined)**: Binder levels start at the introducing construct's entry count; protocol `over` is level 0, an `each` trigger follows, captures advance in their authored order, and a control binder is at the size of its readable input set. The reader admits reads within the FR-464/466 stage tables and refuses a wrong level, duplicate readable name, forward or foreign binder, or read after a branch, choice, repeat or await path where the binder is unavailable with the specified code, cause, stage and owner. In a binders-only slot, a read of `over` or model state refuses `ill_typed`/`operator-ineligible` before binder-scope checks.
- **FR-038-AC-292 (context only)**: At a `choice` or `repeat` decision the reader admits only QSpec FR-466 visible facts owned by the deciding role and listed individually in `visible`; an unavailable binder refuses `missing_declaration`/`missing-name` first. The decision-fact slice remains Planned.
- **QSpec FR-464-AC-7 (examined)**: Binder levels run from the introducing construct's entry count in layout order; a wrong level and two readable binders of one name refuse.
- **QSpec FR-464-AC-8 (examined)**: Expressions read only stage-readable binders; binders-only slots refuse `over` or model-state operations before scope; terminal under `each` has no capture and no binder.
- **QSpec FR-464-AC-12 (examined)**: In each clause, cycle and progress precede slots; slot order is clause, declarations, run, finish, terminal, compensations.
- **QSpec FR-466-AC-10 (examined)**: Control In/Out flow through sequence, choice, repeat, parallel and await governs reads and finish.
- **QSpec FR-466-AC-12 (examined)**: Compensation registration, trigger, activation, retry and recovery stages have distinct readable sets.

## Coverage

The changed files add no CI or gate edits, unsafe code, wire decoder, or dependency. `git diff --check` passed. Hosted Rust Checks at the exact frozen head failed `make lint` before tests, with E0282 at line 3399 and denied `clippy::int_plus_one` at line 3402; Supply-chain, CLA, guard and language-boundary checks passed. No local Cargo or Make job was started under the shared heavy-build lock. The nine new public-reader tests exercise some AC-291 paths but do not establish whole AC-291 or AC-292, both still Planned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, `known` is explicitly `BTreeMap<usize, Vec<ProtocolBinderOrigin>>`, and the index comparison is `index > usize::from(activation.is_some())`; both cited source defects are removed. Exact-head hosted Rust Checks are pending. |
| FND-002 | fixed | In the reviewed PR, the terminal readable set starts empty and inserts `over` and captures only for `origin`; the new public-reader test admits `origin` over and requires `missing_declaration`/`missing-name`, stage `terminal`, for the same read under `each`. Exact-head hosted execution is pending. |

## Disposition verdict

**PASS for recorded code findings ** — the changed source and tests address both findings. Hosted Rust Checks are pending, so this is not final gate evidence. FR-038-AC-291/292 remain Planned.

## Verification after hosted regression fix

Hosted Rust Checks compiled but failed 13 tests, including previously admitted protocol fixtures, the new terminal test, and two work-boundary tests., production code is unchanged from the prior reviewed state. The fix adjusts fixture binder levels to their new readable entry counts, keeps the valid related-end admission assertion, and changes only the added peer-end case to the required `missing_declaration`/`missing-name` scope refusal. Static review found no new code finding. The exact-head hosted Rust Checks are pending.

## Verification after second hosted regression fix

Hosted Rust Checks ran 915 tests and failed three: a duplicate parameter node ID in the sibling-branch compensation fixture, a receive binder at sequence rather than parallel-branch entry level, and the Tree work-boundary pointer., production code remains unchanged. `clause_parameter` reuses an existing node with the same structural key, the independent parallel receive binder is relevelled from 2 to 1, and the Tree fixture restores its dependency charge at 489/490 while retaining total M=549. The related-record test again admits two distinct canonically ordered records by introducing the peer binder in an earlier event and the send binder at level 2. Static review found no new code finding. Exact-head hosted Rust Checks remain pending.

## Verification after measured Tree boundary

The diagnostic hosted run passed 914 tests and failed only TC-226's Tree pointer assertion. Its printed charge sequence measured Tree dependency at 536/537 and final body target at 543/544., the diagnostic print loop is removed, parameter-key reuse is in `relevel_parameter` where it belongs, and TC-226's authored constants match those measured boundaries: 453 pre-scope charges plus 47 new scope charges and 37 Tree charges give dependency 536/537; seven remaining charges give admitted M=544. No production source or gate changed. Static review found no new code finding; exact-head hosted Rust Checks remain pending.

## Final exact-head verification

At authored code, hosted Rust Checks, Supply-chain Checks, guard, CLA and language-boundary checks passed. The original FND-001 and FND-002 remain fixed, and the hosted suite now passes the previously failing protocol and Tree tests. **Final disposition: PASS** for this PR diff. The full FR-038-AC-291/292 criteria remain Planned beyond this binder slice.
