---
id: SR-12070
title: "IR-210 slice C code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@eece4f2c9a49cfc4ea80263e93d0351d0989c6e1; crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/src/checked_package/v2/flat_wire.rs, crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/identity.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/natural.rs, crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs, crates/quire-contract-model/src/checked_package/v2/rust_spelling.rs, crates/quire-contract-model/src/checked_package/v2/structural.rs, crates/quire-contract-model/src/output_mapping.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-210 (slice C), PR quire-contract-ir#431, head eece4f2 over merge base 1bd6749. origin/main has since moved to 0747970 (slice D, #429), which touches no file in this diff. The PR changes 28 unit tests in 10 files of `crates/quire-contract-model/src`. Each test gets `use ix_trace_rs::trace;` in its module (or reuses the import already there in `v2/mod.rs` and `output_mapping.rs`), a `#[trace(...)]` attribute, and its `/// Tracing:` doc line becomes `/// Trace:`. No test body, assertion or name changed. Ten tests changed their ids: some ids were retargeted, some dropped and one added. The rust-review lane is folded into this file. Every added, kept, retargeted and dropped id was read against its criterion text in `spec/` and against the test body. Coverage accounting is in the gap-analysis artifact (SR-12071).

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-2 | examined | The reader refuses malformed, duplicate-member, unknown-member, noncanonical, ... inputs with exactly those codes, and every authored adverse structural mutation returns its declared outcome, before exposing a package; ... |
| FR-038-AC-3 | examined | Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117). |
| FR-038-AC-5 | examined | Every nominal preimage of the in-repo nominal fixture re-derives the node key it is keyed by ...; every authored invalid nominal mutation ... refuses as `invalid_semantic_graph`; ... |
| FR-038-AC-12 | examined | A `creates` or `deletes` entry's eligibility is exactly `model`/`object_type` and `model`/`process`, and this eligibility predicate is checked against every `(member, tag, form)` triple the closed node taxonomy can produce, not sampled; ... |
| FR-038-AC-22 | examined | ... a parameter with a negative, non-canonical or missing level, an empty, non-identifier or wrongly typed name, or an extra binding refuses as `invalid_semantic_graph` at the node's `body`; ... a compound unit with a zero exponent ... refuses ... |
| FR-038-AC-23 | examined | An application node whose `dependencies` omits a body reference target, lists them out of digest order, repeats one, or adds its `result_type` refuses as `invalid_semantic_graph` at `semantic_graph.nodes.dependencies`, located at that node. |
| FR-038-AC-24 | examined | Every refusal about a value carries the RFC 6901 pointer of that value, built from the reader's own position: member names escaped (`~` as `~0`, `/` as `~1`), array elements by index, ... a repeated member at that member, ... a stale mirror at the first differing value, ... |
| FR-038-AC-65 | examined | Every application operator class, operation member kind and constraint kind the operation catalog declares decodes to a member of its closed vocabulary, ... |
| FR-038-AC-79 | examined | A package document holding the integer 9007199254740993 in a node body refuses `noncanonical_wire` with no pointer, as does one holding -9007199254740993, one holding the float `2.0` in a body, and ... U+E000 before ... U+10000 ...; the same document with 9007199254740992, with `2` in place of `2.0`, and with U+10000 before U+E000 ... is not refused `noncanonical_wire`. |
| FR-038-AC-114 | examined | ... an `aggregate` inside a Group's members, ... a `binding` whose value is a `binding`, a `binding` or a Tuple inside a Tuple's members ... and a `binding` as a body root refuse `malformed_wire` at that value; ... |
| FR-038-AC-117 | examined | `CheckedPackageReadLimits` has no depth member and `CheckedPackageLimit` has no `Depth` variant ...; a package holding a chain of 100000 nodes ... on a thread whose stack is 256 KiB, is admitted ...; and an otherwise canonical document whose node body nests a term 300 levels deep, past the strict parse's recursion limit of 128 ... refuses `malformed_wire` with no pointer at the strict parse ... and is never `incomplete`, while the same document nested 20 levels deep, and one nested within the strict parse's limit but far past the grammar ..., each refuse `malformed_wire` at the first value outside the body grammar ... |
| FR-038-AC-121 | examined | Under `quire.temporal.timed/v1`, `lower` `{numerator: "2", denominator: "4"}` refuses `invalid_semantic_graph` ...; ... `lower` `{numerator: "18446744073709551617", denominator: "3"}` with `upper` `{numerator: "18446744073709551616", denominator: "3"}` refuses `invalid-value` at the body, which a float comparison ... gets wrong ...; ... read with a work limit that the GCD or cross-multiplication of its bounds takes past returns `incomplete` naming the `work` limit (FR-038-AC-3) ... |
| FR-040-AC-1 | examined | A `modifies` array holding a `relationship` entry and a `field` entry ... admits; ... a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` with no cause, located at that entry. |
| FR-040-AC-2 | examined | A relationship entry naming an object type, a field entry declared on a process, a relationship or a systems node, and a `creates` entry naming a relationship each refuse as `invalid_model_binding`/`malformed-declaration` at the entry's declaring node key; ... |
| FR-346-AC-2 | examined | IR reading: a body missing `frames` ... each refuse as `invalid_semantic_graph` with no cause, ...; a node of another `semantic_form` carrying this body refuses the same way at its `body`. ... |
| FR-346-AC-6 | examined | A `fields` name `ConfigVersion` does not expose, a `rust_field` of `"not a name"`, a `function` segment `"crate"` and a `rust_type` segment `"9lives"` each refuse as `invalid_model_binding`/`malformed-declaration` at the entry; ... |
| FR-034-AC-3 | examined | Every zero, exact, just-over, and overflowing aggregate limit is classified without partial output or a smaller successful package. |
| NFR-060 | context_only | External requirement `ix://agent-ix/quire-specification/NFR-060`, cited from the FR-034 relationships; not a target of this repository. |
| TC-043, TC-048, TC-053, TC-056, TC-225 | context_only | The test-case ids each test is named for; every attribute keeps its own test-case id. |
| 10 changed files | examined | 28 `#[trace(...)]` attributes, 8 `use ix_trace_rs::trace;` imports, 28 `/// Tracing:` to `/// Trace:` doc-line edits |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `tc_048_a_document_within_the_parse_recursion_limit_is_read` is newly bound to FR-038-AC-117, but its body asserts no clause of AC-117. It only reads a 4-level document (`{"a":[1,{"b":2}],"c":{"d":{"e":3}}}`) and checks that it round-trips. AC-117's depth-free-limits leg is not asserted, because the test never inspects `CheckedPackageReadLimits` or `CheckedPackageLimit`. Its within-limit cases are refusals at the body grammar, not reads. The "127 levels is read" control is already in `tc_048_the_parse_recursion_limit_refuses_malformed_wire_with_no_pointer`. Correct binding: TC-048 only. The AC-117 row has 14 other binders, so no row status changes. | crates/quire-contract-model/src/checked_package/common.rs:1639-1649 |
| FND-002 | low | The retarget of `tc_048_strict_shape_agrees_with_the_strict_parser` from AC-3 to AC-2 removes a wrong id (the test has nothing to do with limits) but adds a weak one. The body is differential: it asserts only that `strict_shape` and `strict_json_value` both accept or both refuse, and that their refusals are equal. It never names a refusal code, so it passes when both refuse with the same wrong code. AC-2's leg "with exactly those codes" is therefore not asserted. Correct binding: TC-048 only, unless the team counts first-pass/strict-parser agreement as an AC-2 leg. | crates/quire-contract-model/src/checked_package/common.rs:1819-1872 |

## Verdict

APPROVE WITH NITS. There are two low trace findings and no medium or high. Mergeable on this method, and either finding may be fixed in this PR or accepted. Neither changes a row's status: every affected row has other binders, and strict coverage is identical before and after (SR-12071).

**Per-binding judgement.** Each binding was checked against the AC text above and the test body.

**Retargets**

- **`strict_shape_agrees`, AC-3 to AC-2.** Dropping AC-3 is right. AC-2 is weak; see FND-002.
- **`syntax_or_duplicate_defect`, AC-3 to AC-2, plus AC-117: correct.**
  - A duplicate member inside a document 2,000,000 levels deep refuses `DuplicateMember` at `/a`, which is AC-2's duplicate-member code.
  - A deep syntax error refuses `malformed_wire` with no pointer, never `incomplete`, on a 256 KiB stack. That is AC-117's strict-parse leg, and AC-2's malformed leg.
  - Not claimed: AC-24's "a repeated member at that member", which the `/a` pointer also asserts. This is an under-claim, not a defect.
- **`decides_reduction_beyond_u128`, AC-5 to AC-121: correct.** AC-5 is about nominal preimages, which this test does not touch. The body decides coprimality beyond u128, including `0/2` and the 2^128 multiples. That is the "non-reduced" decision AC-121 refuses on, at unit level.
- **`number_tokens_decode_identically`, AC-2 kept plus AC-79 added: keep both.**
  - The body asserts `2.0` refuses `noncanonical_wire` as `refused_bytes`, with no pointer. That is AC-79's float leg, verbatim.
  - It asserts an integer past 2^53 (18446744073709551615) refuses the same way. That is the same leg as AC-79's 9007199254740993, at a different value.
  - It asserts `-7` and `1.5` are not refused, and that the `$serde_json::private::Number` spelling is refused identically.
  - AC-79 is therefore partial here. It misses the exact 2^53+1 boundary, the `2` and 9007199254740992 controls, U+E000 ordering and the precedence clause. The integration tests `tc_048_an_integer_past_2_pow_53_a_whole_float_and_a_utf8_ordered_name_are_noncanonical`, `tc_048_the_canonical_bytes_of_those_values_are_not_refused_as_noncanonical`, `tc_048_members_in_utf16_code_unit_order_pass_the_canonical_bytes_check` and `tc_048_an_integer_spelled_past_the_64_bit_range_is_noncanonical` bind it fully.
  - A real partial leg is an honest claim, matching slice D's AC-27/AC-29 practice, so it should not be dropped. AC-2's noncanonical-code leg is asserted too.
- **`tc_056_field_names_are_identifiers`, FR-040-AC-1 dropped: correct.** AC-1 says nothing about identifier names; its `field` case is "without `name`". The identifier rule for `name` is in FR-040's Behavior prose ("`name` is the field's identifier"), and no numbered FR-040 AC states it. TC-056 only is honest. The missing AC is a pre-existing spec gap, noted in SR-12071.

**Drops**

- **AC-3 from the three depth tests: correct.** `strict_shape_agrees`, `syntax_or_duplicate_defect` and `deep_syntax_error_in_linear_time` assert parse behaviour, not a byte, node, edge, occurrence, diagnostic or work limit.
- **`deep_syntax_error_in_linear_time` is TC-048 only.** No numbered criterion states the reader's linear-time first pass, and AC-117 is about stack and recursion, not time. TC-only is honest.
- **AC-2 from `v2_literal_numbers_are_integers`: correct.** The body asserts the `is_literal_value` predicate (1.5 and 2.0 are not V2 literals) and no refusal code. AC-79 and AC-93 are about canonical bytes, not the literal grammar. No numbered AC fits, so TC-only is honest.
- **AC-5 from `identity.rs` `integer_and_identifier_grammars_are_exact`: correct.** The body tests the `integer_magnitude`, `positive_integer` and `is_identifier` predicates. AC-5's legs are fixture round-trips and refusals of authored invalid mutations; the predicates feed those refusals, but the test asserts none. This is a stricter standard than `structural.rs` AC-22 below, but both are defensible: the AC-22 predicate is the exact parameter `level` grammar AC-22 names (`structural.rs:445`), while AC-5 names no grammar.
- **NFR-060 and FR-034-AC-3 from `tc_043_absolute_region_arithmetic_overflow_refuses`: correct.**
  - NFR-060 is a quire-specification requirement, not a target here. Before this PR it was an unmatched tag.
  - The body shifts an `OutputByteRegion` past `u64::MAX` and gets `ArithmeticOverflow`. That is a region endpoint (STD-003's `arithmetic_overflow` "region endpoint overflows its width"), not an aggregate limit, so FR-034-AC-3 does not fit.
  - FR-034-AC-3 keeps three integration binders.

**Kept or added, unchanged ids: correct**

- **AC-24, on `trails_materialize`.** It asserts the escaping, the indices, the empty pointer and the round-trip.
- **AC-24, on `first_difference`.** It asserts AC-24's "stale mirror at the first differing value" locator.
- **AC-117, on three tests.**
  - `term_walk`: a 200,000-deep term on 256 KiB, with no stack overflow.
  - `parse_recursion_limit`: 127 read, 128 and more refuse `malformed_wire` with no pointer, never `incomplete`.
  - `member_scan`: a 200,000-deep application found on 256 KiB.
- **AC-114, on `a_term_is_checked_against_the_place_it_stands_in`.** It covers a binding as body root, an aggregate in a Group, and a binding in a Tuple, each at that value's pointer.
- **AC-3, on the three `natural.rs` work tests.** They cover exact work admits and one-over is `incomplete` with kind `Work`, limit and consumed. The comparison test also asserts AC-121's work leg at unit level (not claimed).
- **AC-121, on `compares_rationals_exactly_beyond_2_pow_64`.** It asserts AC-121's 2^64+1/3 against 2^64/3 ordering, the case a float comparison gets wrong.
- **AC-22, on `structural_integer_grammars`.** It covers the level grammar and the nonzero exponent.
- **AC-23, on `application_join`.** It asserts the join a dependencies list is checked against.
- **AC-12, on the `frame.rs` creates/deletes test.** It runs exhaustively over `CheckedNodeKind::all()`, which is the AC's "not sampled" predicate leg.
- **FR-040-AC-2, on the modifies-entries test.** It asserts the exact relationship and field eligibility sets.
- **AC-65, newly added on `the_catalog_vocabularies_are_the_decoded_enums`.** It asserts that the production catalog's member kinds and constraint kinds equal the closed enum sets. That is AC-65's "decodes to a member of its closed vocabulary" for two of the three vocabularies. Operator class is not covered here; the other binders back it.
- **FR-346-AC-2, on `exactly_one_kind_has_an_abstraction_relation_body`.** It is the unit-level leg of "a node of another `semantic_form` carrying this body refuses".
- **FR-346-AC-6, on both `rust_spelling.rs` tests.** They assert `"not a name"`, `"crate"` and `"9lives"` are rejected by the predicates the entry check uses.

**Rust lane**

- **Attribute spelling.** Every attribute uses the imported short form `#[trace("TC-0nn", ...)]`, with the test-case id first, and sits above `#[test]`. `quire trace --file` over all 10 files reports every one of the 28 tests' ids as `rust-trace-attribute` (SR-12071). No fully qualified `#[ix_trace_rs::trace]` form is used.
- **Imports.** There are 8 new `use ix_trace_rs::trace;` imports. `v2/mod.rs:2446` and `output_mapping.rs:2751` already had the import.
- **Doc lines.** Every attribute's ids equal its `/// Trace:` doc-line ids.
- **Untagged tests.** Every `#[test]` in the changed modules is tagged except two non-`tc_` tests in `v2/mod.rs`: `dependency_selection_required_members_match_the_type` and `exactly_one_kind_has_a_frame_body`. Both are outside this slice's "name-bound" scope.
- **Surface.** The diff has no panic, unsafe, integer-conversion or lock surface.

**Gates re-run on head eece4f2**

- `cargo fmt --all -- --check` is clean.
- `cargo clippy --locked -p quire-contract-model --all-targets -- -D warnings` is clean.
- `cargo test -p quire-contract-model --lib -- checked_package:: output_mapping::tests::tc_043_absolute_region` passes 188 of 188.

No mutants were run. No body or assertion changed, so the tags were judged by reading each assertion against the AC.
