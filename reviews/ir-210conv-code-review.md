---
id: SR-12110
title: "IR-210 conversion slice code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@a773b9051c732d683b545017e973364a8d982cea; crates/quire-contract-model/src/canonical.rs, crates/quire-contract-model/src/decimal.rs, crates/quire-contract-model/src/expression.rs, tests/it/conformance.rs, tests/it/expression.rs, tests/it/identity.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: references
---

## Summary

Ticket: IR-210 (conversion slice), PR quire-contract-ir#434, head a773b90 over origin/main fabdbb4. The diff converts 15 tests in 6 files from the fully qualified `#[ix_trace_rs::trace(...)]` to `use ix_trace_rs::trace;` plus `#[trace(...)]`, and replaces the legacy `/// Tracing:` and bare-id doc lines with one `/// Trace:` line that mirrors the attribute. The rust-review lane is folded into this file; coverage accounting is in SR-12111. Measured with quire 0.36.2 (engine 0.50.2) and quoin 0.28.3.

Every id on every converted test was read against its criterion text in `spec/` and against the test body. The emphasis was on the three tests that gain credit with this PR, because on main their attribute ids were not read by quire at all (`quire trace --file` shows FR-016-AC-6 on `canonical.rs` as `evidence_near_miss`, and nothing for FR-013-AC-5 on `decimal.rs`):

- `decimal.rs` `tests::tc_016_the_grammar_is_minimal_base_ten` (FR-013-AC-5, new credit)
- `decimal.rs` `tests::tc_016_range_is_decided_after_the_grammar` (FR-013-AC-5, new credit)
- `canonical.rs` `tests::tc_017_a_limit_and_an_encoder_fault_share_the_code_but_not_the_message` (FR-016-AC-6, new credit)

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-013-AC-5 | examined | Integer bounds and literals at `i128::MIN` and `i128::MAX` ... decode and serialize back to the same decimal strings. Each of the eight members given as a JSON number (`0`, `1`, `1.0`, `9223372036854775807` and `100000000000000000001`) or a string outside the minimal-decimal grammar (`""`, `"+1"`, `"01"`, `"-0"`, `"1.0"`, `"1e3"`, `" 1"`) refuses `invalid_wire_format`. Each of the eight given `"170141183460469231731687303715884105728"` or `"-170141183460469231731687303715884105729"` refuses `invalid_numeric_bounds`; ... |
| FR-016-AC-6 | examined | Every canonical byte sequence and digest of this crate comes from `quire-canonical`: for one fixture of each of the five kinds the bytes returned equal an expected byte string written out in the test ...; a byte limit of the exact length returns the bytes and one byte lower returns `canonicalization_resource_exhausted`; and the crate source holds no `CanonicalWriter` ... symbol ... |
| FR-016-AC-3 | context_only | A deterministic reservation-failure harness forces canonical byte allocation failure and verifies `canonicalization_resource_exhausted`, no partial public bytes, and no digest. |
| FR-015-AC-8 | examined | At an `i128::MIN` endpoint, integer `reject` negation and division by `-1` report `potentially_undefined` ...; `saturate` negation returns the declared maximum ...; the singleton product `i128::MAX/(i128::MAX - 1) * (i128::MAX - 1)/i128::MAX` is admitted as normalized `1/1` ...; `i128::MAX/1 * 2/1` reports `potentially_undefined` ... None wraps or panics. |
| FR-013-AC-6 | examined | Public `IntegerType` bounds/accessors, `RationalType` numerator bounds and maximum-denominator/accessors, and integer/rational literal payloads retain `i64::MAX + 1`, `u64::MAX`, and the permitted i128 endpoints without narrowing; ... |
| FR-014-AC-8 | examined | With matching declared types, integer literal nodes at `i64::MAX + 1`, `u64::MAX`, `i128::MIN` and `i128::MAX` and rational literal nodes ... retain those exact values in typed output; ... A literal outside its declared type reports `invalid_numeric_bounds` at that literal's span. |
| FR-019-AC-7 | examined | Public compile-time/API fixtures construct and read `IntegerType` bounds, `RationalType` numerator and denominator bounds, and integer/rational literal payloads at `i64::MAX + 1`, `u64::MAX` and the permitted i128 endpoints through signed 128-bit signatures without lossy casts; ... |
| FR-018-AC-1 | examined | Running the corpus directory yields one matching row per input with non-empty trace targets, and the union of observed tokens equals the published inventory ... |
| FR-019-AC-1 | examined | Compile-time/API fixtures plus public-source signature inspection show wire/request values are distinct from private-field validated values, ... the fixed conformance registries, three semantic-limit constants, and wire-depth constant are stable public exports, ... |
| FR-019-AC-3 | examined | `expected_inventory` equals the five prefixed registries and nothing else, strictly ascending with no duplicate. |
| FR-020-AC-1 | examined | A process test runs the published corpus twice without linking a test harness to the library and obtains byte-identical JSON Lines, non-empty observed trace ids per fixture input, empty stderr, and complete tool/schema/profile identity. |
| FR-020-AC-3 | examined | The published fixture schema accepts `"-9223372036854775808"`, `"0"` and `"9223372036854775807"` in each of the eight integer members. |
| FR-020-AC-4 | examined | The published package schema accepts a revision and a byte offset of `9007199254740992`. |
| FR-020-AC-5 | examined | The fixture input schema refuses as `invalid_corpus` values above `9007199254740992` for each of these members: ... |
| FR-020-AC-6 | examined | The fixture input schema passes the value `9007199254740992` for each of these members: ... |
| FR-020-AC-7 | examined | The runner returns `invalid_wire_format` when a decoder fixture supplies a JSON number in one of the eight integer members straight to the decoder. |
| FR-020-AC-8 | examined | The expectation of a decoder fixture that supplies a JSON number in one of the eight integer members carries the code `invalid_wire_format`. |
| FR-020-AC-9 | examined | The same number as the decoder fixture supplied to the decoder, placed in one of the eight integer members of a valid-operation input, is refused as `invalid_corpus` at the fixture schema check. |
| FR-020-AC-10 | examined | Every canonical file of the published corpus whose object holds one of the eight integer members equals the expected bytes written in the test. |
| FR-020-AC-11 | examined | Every canonical file of the published corpus whose object holds one of the eight integer members spells those members only as strings. |
| FR-020-AC-12 | examined | A published corpus run exits 0 and emits one row per fixture input, with every row reporting `match`. |
| FR-020-AC-13 | examined | The published fixture schema rejects `0`, `1.0`, `"+1"`, `"01"`, `"-0"` and `""` in each of the eight integer members. |
| FR-020-AC-14 | examined | The published package schema rejects a revision and a byte offset of `9007199254740993`. |
| FR-020-AC-15 | examined | A successful `document_json` expression document that the published schema rejects makes the runner exit 2 with the code `invalid_corpus`. |
| FR-016-AC-5 | examined | The eight integer members are JSON strings of minimal base-ten digits in every canonical object: ...; no byte sequence of the form `"<one of the eight names>":` is followed by a digit or `-` in any recorded canonical fixture; ... |
| FR-011-AC-3 | examined | ... zero source or requirement revisions and a revision above 9007199254740992 (`invalid_source_revision`, `invalid_requirement_revision`, through the constructors, the package decoder, ...) ... fail with their registered structured diagnostic codes. |
| FR-012-AC-6 | examined | Source spans reject ... a byte offset above 9007199254740992 ... using `invalid_source_span`, through the constructors, the package decoder, ...; the probe and coverage inputs of the conformance corpus refuse it `invalid_corpus` at the fixture schema (FR-020-AC-5). |
| NFR-001-AC-1 | examined | Two complete corpus runs over fixed inputs and profiles produce byte-identical results. |
| StR-003-VC-1 | examined | Malformed, ill-typed, undefined, unsupported, and orphaned inputs yield stable source-located diagnostic codes. |
| TC-015, TC-016, TC-017, TC-018 | context_only | Test-case rows in spec/core/matrix/tests.md and spec/conformance/matrix/tests.md naming the FRs each test case verifies. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `tc_017_a_limit_and_an_encoder_fault_share_the_code_but_not_the_message` newly backs FR-016-AC-6, but its body asserts none of the criterion's clauses. It hand-builds `Error::Limit` and `Error::Protocol` values and checks that `encoder_refusal` gives both `canonicalization_resource_exhausted` with different messages. No canonical bytes, no digest, no real byte-limit call, no symbol scan. FR-016-AC-6 is already backed by the two `canonical_v1_quire_canonical.rs` tests that do assert it. Drop FR-016-AC-6 from this test's attribute and Trace line and keep TC-017. | crates/quire-contract-model/src/canonical.rs:1139-1142 |
| FND-002 | low | The two `decimal.rs` tests newly back FR-013-AC-5 with partial, helper-level evidence. `tc_016_the_grammar_is_minimal_base_ten` asserts the private `in_grammar` predicate, not the `invalid_wire_format` refusal the criterion names. `tc_016_range_is_decided_after_the_grammar` asserts the JSON-number spellings fail as a serde error with no code, and checks `invalid_numeric_bounds` only for the positive one-past value. The bindings are honest partials, because each test asserts real sub-clauses. But FR-013-AC-5 goes from 6 to 8 backers without adding any observable evidence. The `wire.rs` tests already assert the member-level codes. | crates/quire-contract-model/src/decimal.rs:92-136 |

## Verdict

APPROVE WITH NON-BLOCKING FINDINGS. The conversion is mechanical and correct. FND-001 is a one-line binding correction. It is best made in this PR, because the PR itself creates the credit. Neither finding changes any coverage total or any backed/unbacked status.

Checked clean:

- **No assertion changed.** I filtered `git diff -U0 origin/main...HEAD` to drop `///` lines, `#[trace(` and `#[ix_trace_rs::trace(` openers, `use ix_trace_rs::trace;` and blank lines. Nothing was left. All 33 added and 57 deleted lines are trace metadata.
- **Attribute ids equal the Trace-line ids** on all 15 tests, in the same order. No id was dropped or added against the attribute on main.
- **Imports.** `decimal.rs` and `canonical.rs` add `use ix_trace_rs::trace;` in their `mod tests`. `expression.rs` `range_set_tests`, `tests/it/conformance.rs`, `tests/it/expression.rs` and `tests/it/identity.rs` already had it. Clippy reports no unused import.
- **Fully qualified spelling is gone.** `git grep 'ix_trace_rs::trace'` finds only `use` lines in `.rs` files. It finds no hit in `spec/`, `plan/`, the README, AGENTS.md or CLAUDE.md. The only other hits are prose in historical `reviews/` artifacts that describe the old form. Those are records, not examples.
- **Body against id for the rest of the converted tests.** Each claim was already credited on main through doc lines, and each body asserts its id:
  - **TC-018 corpus test.** The test runs the corpus twice and requires byte-identical stdout (NFR-001-AC-1 and FR-020-AC-1). It requires every row to `match` with non-empty trace ids (FR-018-AC-1 and FR-020-AC-12). It rebuilds the inventory from the five registries in strict order (FR-019-AC-3). It runs the negative corpus with structural diagnostics (StR-003-VC-1).
  - **Fixture schema test.** All 8 members are checked against the accepted and refused spellings (FR-020-AC-3 and FR-020-AC-13). This is the schema-level half of FR-013-AC-5.
  - **Package schemas test.** It checks 2^53 accepted and 2^53+1 refused (FR-020-AC-4 and FR-020-AC-14).
  - **Decoder fixture test.** It covers FR-020-AC-7, FR-020-AC-8 and FR-020-AC-9.
  - **Probe and coverage bounds test.** It covers FR-020-AC-5, FR-020-AC-6, and the fixture-schema clauses of FR-011-AC-3 and FR-012-AC-6.
  - **Recorded canonical files test.** It checks 47 written-out files (FR-020-AC-10). It also runs the no-digit-after-name scan, which serves both FR-020-AC-11 and that clause of FR-016-AC-5.
  - **Expression document test.** It covers FR-020-AC-15.
  - **`tests/it/expression.rs` wide-value tests.** They cover FR-013-AC-6, FR-014-AC-8 and FR-019-AC-7, and the public-path clauses of FR-015-AC-8.
  - **`range_set_tests`.** They assert the helper-level FR-015-AC-8 product reduction and the reject range.
  - **`identity.rs` tc_015.** It covers the constructor and decoder clauses of FR-011-AC-3 and FR-012-AC-6.
  - **FR-019-AC-1 on the corpus test.** The body asserts only the registry-export clause. That thin binding predates this PR and is recorded in SR-12111.
- **Rust lane.** The PR has no production code change, no new `unwrap` or panic surface, and no new dependency.
  - **Tests.** `cargo fmt --all --check` is clean. `cargo clippy --locked --workspace --all-targets -- -D warnings` is clean. `cargo test --test it -- conformance:: identity::tc_015 expression::wide` passed 16 of 16. The model lib tests `decimal::tests`, `canonical::tests` and the two `range_set_tests` passed 5 of 5.
  - **Line length (note).** The single `/// Trace:` line on the corpus test is 111 columns, over `max_width = 100`. rustfmt does not reflow comments here, and the file already has longer lines, so this is cosmetic and not a finding.
- **PR body.** The body is accurate on every measured claim: the 6->8 and 2->3 gains, no lost pair, 43/0, 508/565, and unmatched tags 75->62. It names quire 0.36.1, while this machine has 0.36.2. The results reproduce on 0.36.2. The body does not say whether the gained bindings were checked against the criterion text. FND-001 shows one of them does not hold.
