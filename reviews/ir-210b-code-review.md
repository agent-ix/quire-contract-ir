---
id: SR-12065
title: "IR-210 slice B code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/identity.rs, tests/it/integration.rs, tests/it/conformance.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: references
---

## Summary

Ticket: IR-210 (slice B), PR quire-contract-ir#430, over origin/main (the merge base). One commit. The diff touches three test files only. Seven tests that were credited by name and legacy doc lines gain `#[trace(...)]`, the files gain `use ix_trace_rs::trace;`, and each `/// Tracing:` block plus its bare id lines becomes one `/// Trace:` line. No test body, assertion or function name changed. The rust-review lane is folded into this file. Each added and each dropped id was read against its criterion text in `spec/` and against the test body. Coverage accounting is in the gap-analysis artifact (SR-12066).

Every added id was already claimed on main by a doc line on the same test, so the PR adds no new claim. It only changes the binding form and drops ids. Two of the drops are wrong in the way SR-12020 FND-002 on #426 described: StR-001-VC-1 and StR-001-VC-2 are dropped as "not FR/NFR rows, over-claims", but quire mints and counts stakeholder VC rows, and the body asserts the TC-015 legs of both.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-011-AC-1 | examined | A package round trip through the non-canonical JSON value representation preserves namespace, schema version, requirement ID, requirement revision, and source-document identity/revision with structural equality. |
| FR-011-AC-2 | examined | Incrementing one requirement revision changes its clause and dependency identities without changing unrelated requirement identities. |
| FR-011-AC-3 | examined | Empty or malformed package, source-document, requirement, clause, anchor, or dependency-path-segment identities; a zero schema major; zero source or requirement revisions and a revision above 9007199254740992 ...; duplicate requirement or clause identifiers; non-increasing revision advances; and cross-package references fail with their registered structured diagnostic codes. Schema minor zero is valid. |
| FR-012-AC-1 | examined | A floating executable clause is rejected at its source span with a stable diagnostic code. |
| FR-012-AC-2 | examined | The recursive reference body returns each exposed structural dependency identity exactly once in deterministic order. |
| FR-012-AC-3 | examined | Malformed, cross-package, stale-revision, orphaned-requirement, and orphaned-clause references produce distinct STD-001 diagnostic codes and the narrowest available source span. |
| FR-012-AC-4 | examined | Every executable clause kind accepts only its declared anchor kinds using `floating_executable_clause` or `incompatible_clause_anchor`; informational clauses reject any anchor using `informational_clause_anchored`; malformed anchor identifiers use `invalid_identifier`. |
| FR-012-AC-6 | examined | Source spans reject zero line/column positions, decreasing byte offsets or positions, a byte offset above 9007199254740992 ..., and endpoints from different source-document identities/revisions using `invalid_source_span`, through the constructors, the package decoder, ... |
| NFR-002-AC-3 | examined | Public JSON field names, identity kinds, anchors, clause kinds, dependency kinds, and diagnostic codes contain no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary. |
| FR-018-AC-1 | examined | Running the corpus directory yields one matching row per input with non-empty trace targets, and the union of observed tokens equals the published inventory ...; a corpus missing a token's only fixture, a fixture with an unknown operation prefix or a missing expectation, and oversize, over-count and over-budget corpora fail before any row is written. |
| FR-018-AC-2 | examined | Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing. |
| FR-018-AC-3 | examined | Raw package probes pin exact and one-past wire depth, and quoted delimiters do not count toward depth. |
| FR-019-AC-2 | examined | The complete negative corpus executes package/expression decode, validation, canonicalization, and coverage through `catch_unwind`; exact-at-limit and one-past-limit type depth, semantic node, and semantic collection cases return the specified result with no public panic, partial result, or message parsing. |
| FR-020-AC-2 | examined | Process fixtures pin exit 1 with all six mismatch kinds in fixed order and exit 2 for each of the six closed operational codes ..., with no absolute path in the error record; stdout/stderr separation, no partial output, `--version`, unknown/repeated arguments, non-UTF-8 argument handling, and pre-decode rejection of a 60000-level referenced JSON input are exact. |
| NFR-003-AC-1 | examined | Every declared invalid-input class returns a stable diagnostic code and no public panic. |
| StR-001-VC-1 | examined | Every executable clause has a stable requirement revision, named anchor, typed expression, dependency set, and source location. (Verification: Contract-model inspection and conformance fixtures (TC-015, TC-016)) |
| StR-001-VC-2 | examined | Downstream artifacts can cite the same package, requirement, clause, and expression identities. (Verification: Interface and canonicalization review (TC-015, TC-017)) |
| StR-003-VC-1 | examined | Malformed, ill-typed, undefined, unsupported, and orphaned inputs yield stable source-located diagnostic codes. (Verification: Negative corpus and diagnostic review (TC-016, TC-018)) |
| STD-001 | context_only | Contract IR diagnostic code registry (a Standard, no criterion rows) |
| TC-015 | context_only | Test case named on TC-015 tests; kept on all three TC-015 tests |
| TC-018 | context_only | Test case named on TC-018 tests; kept on all four TC-018 tests |
| tests/it/identity.rs | examined | `use ix_trace_rs::trace;` and two `#[trace("TC-015", ...)]` attributes with `/// Trace:` lines |
| tests/it/integration.rs | examined | `use ix_trace_rs::trace;` and `#[trace("TC-015")]` |
| tests/it/conformance.rs | examined | `use ix_trace_rs::trace;` and four `#[trace("TC-018", ...)]` attributes with `/// Trace:` lines |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | StR-001-VC-2 dropped from the identity conformance test as "not an FR/NFR row", but quire counts VC rows, this was the row's only backer (StR-001 2/2 -> 1/2), and the body asserts the TC-015 leg | tests/it/identity.rs:118-133 |
| FND-002 | low | StR-001-VC-1 dropped from the same test on the same wrong rationale although the body asserts its TC-015 legs; the row stays backed elsewhere, so no credit is lost | tests/it/identity.rs:118-133 |
| FND-003 | low | FR-018-AC-2 kept on the mismatch test although the test applies five mutations to one fixture at once, so the AC's "independently ... each produces the exact mismatch result" leg is not asserted | tests/it/conformance.rs:221-292 |

## Verdict

CHANGES REQUESTED. One medium (FND-001) and two low findings. FND-001 should be fixed in this PR by putting StR-001-VC-2 back on `tc_015_identity_anchor_dependency_and_reference_contract_conforms`. FND-002 is the same one-line fix. FND-003 is a pre-existing weakness carried over from a doc line. That test is FR-018-AC-2's only backer, so dropping the tag would unback the row. The better fix is a follow-up that splits the mutations, and deferring it is reasonable.

FND-001 detail. Before this PR, `quire trace --file tests/it/identity.rs` showed StR-001-VC-2 (and VC-1) bound to the identity test through `rust-doc-comment-id`. After it, no symbol in the repository binds StR-001-VC-2: `minted_targets` flips it from backed to unbacked, and the StR-001 VC group goes from 2/2 to 1/2. SR-12020 on #426 dropped StR-001-VC-2 from the TC-017 canonicalization test, so this test was the last backer. The test does assert the TC-015 leg of the VC, the leg the VC's own verification column assigns to TC-015. It shows that the requirement and clause references a downstream artifact cites are equal across two packages that differ only in another requirement's revision (lines 186-203), that `SemanticIdentity::Package/Requirement/Clause/Dependency` serialize in the neutral vocabulary (lines 511-540), and that a round trip preserves every identity (line 139). The expression-identity leg belongs to TC-017. `strict` coverage does not catch this, because the 43 unbacked rows it counts are rows of the matrix reference tables and VC rows are not among them.

Per-binding judgement. Each binding was checked against the AC text above and the test body.

- **identity `..._contract_conforms`.** Every added id is correct.
  - FR-011-AC-1: the round trip `assert_eq!(round_trip, package)` is on line 139.
  - FR-011-AC-2: the advanced clause and dependency identities differ from the originals, and the unrelated requirement, clause and dependency identities are equal across `valid_package_at(3)` and `valid_package_at(4)`.
  - FR-011-AC-3: this is the constructor leg. It covers the empty and malformed namespace and identifiers, zero revisions, duplicate requirement and clause, and the non-increasing `advance(3)`. The decoder and 2^53 legs are complementary legs on the other two TC-015 tests.
  - FR-012-AC-1: the floating clause is rejected at `source_span`.
  - FR-012-AC-2: the dedup and the alpha/zeta order are asserted.
  - FR-012-AC-3: it covers malformed, cross-package, stale, orphaned requirement and orphaned clause references, each with its span.
  - FR-012-AC-4: the test checks the full 6x5 kind/anchor table and an invalid anchor identifier.
  - FR-012-AC-6: it covers a zero line, a reversed span, and a cross-document span.
  - NFR-002-AC-3: it checks serialized codes, kinds, anchors, identity kinds and the package JSON field names against the forbidden list.
- **identity `..._untrusted_json_...`.** The decoder legs of FR-011-AC-3, FR-012-AC-3 (malformed and cross-package, with span), FR-012-AC-4 (floating, informational, incompatible and invalid anchor name) and FR-012-AC-6 (offset order, cross-document, revision mismatch) are correct.
  - Dropping FR-011-AC-1 is right. The body serializes the valid package but never decodes a valid one, so no round trip is asserted.
- **integration `tc_015_public_api_exposes_the_v1_wire_schema`.** The TC-015-only tag is right. The body checks only `V1_1.major()` and `.minor()`, so dropping FR-011-AC-1 is right.
- **conformance mismatch.**
  - FR-018-AC-1 is correct. The negative half is asserted: the uncovered-token corpus, an unknown operation prefix, a missing expectation, and over-count, oversize and over-budget corpora. Each of these goes through `error_code`, which asserts that stdout is empty. The positive half is on the corpus test.
  - FR-020-AC-2 is correct. The fixed order of five kinds is asserted, coverage is asserted, and four of the six exit-2 codes are covered here, along with unknown and repeated arguments, a non-UTF-8 argument, and 60000 levels. The test below covers the other two codes, `--version`, and the no-absolute-path check.
  - FR-018-AC-2: see FND-003.
  - Dropping StR-003-VC-1 is right on body grounds. The test asserts runner operational codes, which NFR-003's Scope places outside the semantic, source-located diagnostics, and it asserts no source-located diagnostic. The row loses its only credit, though, which is gap-analysis SR-12066 FND-001.
- **conformance unsafe paths.**
  - FR-020-AC-2 is correct: it covers `unsafe_path` (traversal and symlink, each with no absolute root in stderr), `unsupported_profile`, `--version`, and stray, non-UTF-8 and orphan entries.
  - Dropping FR-018-AC-1 is right. Stray non-`.json` entries, non-UTF-8 names, orphan expectations and unsafe paths are FR-018 Behavior and FR-020 classification, not items on AC-1's list.
- **conformance semantic depth.**
  - FR-019-AC-2 is correct. It runs the complete corpus under `catch_unwind`, which includes the semantic-node at-limit and over-limit fixtures. It also checks type depth and semantic collection at the limit and one past it.
  - Dropping NFR-003-AC-1 is harmless. On main, the id sat on the `Tracing:` line, which quire reads only as an `evidence_near_miss` citation, so it never credited the row. The body does not assert "every declared class" in any case. That is TC-454's job, in `nfr003_invalid_input.rs`.
- **conformance wire depth.** FR-018-AC-3 is correct. 575 arrays plus the object give 576 levels and reach `document`. 576 arrays plus the object give 577 levels and reach `document.nesting`. The quoted 600-delimiter string does not count.
- **STD-001 dropped (both identity tests).** Harmless. On main it was only an `evidence_near_miss` citation. It is a Standard with no criterion rows, so it never credited anything. The in-body comment on line 402 is still a `mention` citation.

Rust lane:

- **Attribute spelling, measured.** At head, `quire trace --file` reads the seven tests as `rust-trace-attribute`: 26 (id, symbol) pairs, all `rust-trace-attribute`. The tests that still use the fully-qualified `#[ix_trace_rs::trace(...)]` show only `rust-doc-comment-id`, `rust-comment-id` and `rust-test-name-id`, never `rust-trace-attribute`. That confirms the claim that only `use ix_trace_rs::trace;` plus `#[trace]` is recognised. The fully-qualified tests are untouched. There are seven in `conformance.rs` (corpus, fixture-schema integer strings, package schemas, decoder fixture, probe/coverage bounds, recorded canonical files, expression document), not four, plus `tc_015_a_revision_or_byte_offset_above_two_to_the_53_is_refused` in `identity.rs`. Their credit still rests on legacy doc lines.
- **Doc lines agree with attributes.** On all seven tests the attribute ids equal the `/// Trace:` ids. The identity test wraps its `Trace:` line over two doc lines, but credit comes from the attribute, so this has no effect.
- **Imports.** Each new `use ix_trace_rs::trace;` is used, and clippy stays clean. `identity.rs` now mixes `#[trace]` and `#[ix_trace_rs::trace]`. That is cosmetic and follows from the scope.
- **Surface.** The diff has no panic, unsafe, integer-conversion, lock or async surface.

Gates re-run on:

- `cargo fmt --all -- --check` is clean.
- `cargo test --locked --test it -- identity:: integration:: conformance::` passes 21 of 21. That is the 16 tests in the three files plus 5 `application_node_identity::` tests that the filter also matches.

No mutants were run. No body changed, so a mutant would only re-measure tests that already exist on main. It cannot judge these tags, which were judged by reading each assertion against the AC.

## Dispositions

Round 1, (fix atop the reviewed PR). Each fix was checked against the diff `..`, the criterion text and the test body. `quire trace --id` was re-measured. `cargo test --locked --test it -- tc_015_identity_anchor tc_018_the_corpus_runs` passes 2 of 2, and `cargo fmt --check` is clean.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | StR-001-VC-2 is back on `tc_015_identity_anchor_dependency_and_reference_contract_conforms`, in both `#[trace]` and `/// Trace:`. `quire trace --id StR-001-VC-2` binds it there as `rust-trace-attribute`, and the row is backed again. |
| FND-002 | fixed | StR-001-VC-1 is restored on the same test (`rust-trace-attribute`). It is still also bound by tc_016 in tests/it/expression.rs. |
| FND-003 | deferred | IR-839 (Backlog). It asks for one mutation per fixture for each mismatch kind, traced to TC-018 and FR-018-AC-2, with a mutant that merges two kinds to show the test catches it. The claim predates this PR, and that test is the row's only backer. |
