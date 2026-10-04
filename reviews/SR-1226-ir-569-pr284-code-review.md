---
id: SR-1226
title: "code review of PR 284 (IR-569 registered revision and span codes through the expression decoder)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@8afbebdb0b7f6e3ad8aedc0715808e6e737feec5; crates/quire-contract-model/src/identity.rs, crates/quire-contract-model/src/wire.rs, tests/it/executable_binding.rs, spec/core/matrix/tests.md, spec/tests.md"
review_set: subset
---
# SR-1226: code review of PR 284

## Summary

Ticket: IR-569. This is a code review with the rust-review lane folded in. It covers
`git diff origin/main...HEAD` at 8afbebdb0b7f6e3ad8aedc0715808e6e737feec5, base main
543cd8a8b6a5fe2a1974617869824a75f6c38629 (the merge base; one commit on the branch).

What was checked, and what holds:

- **One mapping, not two.** `WireSourceSpan` and `WireRequirementRef` (the package
  decoder's wire types) become `pub(crate)` with their `validate()`, and the expression
  decoder uses them. No second mapping table, no new path strings: the codes and paths come
  from the constructors (`RequirementRef::parse` gives `reference.revision`;
  `SourceRevision::new` gives `source.revision`; `SourceLocation::new` and
  `SourceSpan::new` give `source_span`). These are the same relative dotted paths the
  package decoder already reports, and the corpus fixtures
  `package-invalid-*-revision*` and `package-invalid-span*` pin them for the package
  operation. They are not JSON pointers in either operation.
- **Every span field.** Every span or owner field in `wire.rs` is now a wire type:
  `ExpressionInput.owner`, enum and record declaration `source`, `WireEnumVariant.source`,
  `WireRecordField.source`, `WireValueDeclaration.source`, `WireFunctionParameter.source`,
  `WireFunctionDeclaration.source`, quantifier `local_source` and `WireExpression.source`.
  A grep of `wire.rs` finds no remaining serde-derived `SourceSpan`, `RequirementRef` or
  revision field, and every one is `.validate()`d in its builder.
- **Order.** The owner is validated after `preflight` and before the binding's owner and
  anchor comparison, so a bad owner revision reports its own code, not
  `malformed_reference`. An expression's own span is validated after its children. That
  fits STD-001's precedence (identity grammar before span structure).
- **FR-013-AC-5 unaffected.** The eight integer members are untouched. The three
  `tc_016_*` FR-013-AC-5 tests pass unchanged.
- **Behaviour changes, judged against the spec.** A zero line or column, reversed endpoints
  or cross-source endpoints in an expression span now report `invalid_source_span` (was
  `invalid_wire_format` at `expression`). That is what FR-012-AC-6 lists word for word.
  Unknown members in `owner` or a span are now refused. The published fixture schema
  already closes `requirementRef`, `sourceIdentity`, `sourceLocation` and `sourceSpan` with
  `additionalProperties: false`, so this only brings the decoder into line with the schema.
  One change the PR body does not list: a malformed owner `package` or `requirement` now
  reports `invalid_package_namespace` or `invalid_identifier` at `reference.package` or
  `reference.requirement` (was `invalid_wire_format`). That is also what FR-011-AC-3 asks.
- **Producers.** Nothing that produces these inputs breaks. QSL (origin/main 3f69fc62)
  and CG (origin/main 1c817d8) reach IR expression inputs only through
  `BoundPackage::from_json_bytes`. That decoder checks the projection schema first, and the
  schema already refuses extra members in owner and span objects. QSL also serialises IR's
  own typed `SourceSpan`, which has no extra members. The conformance runner checks corpus
  inputs against the same closed schema. `make corpus`: 107/107 fixtures match.
- **Rust idioms.** No new `unwrap`, panic, `unsafe`, allocation or integer cast. `line` and
  `column` stay `u32` and `byte_offset` stays `u64`, so the type still decides a width
  overflow (`invalid_wire_format`). The `one(...)` and `?` usage follows the file's existing
  pattern. The doc comments on the newly `pub(crate)` items state why they exist.
- **Tests and oracles.** The two `wire::tests::tc_015_*` tests assert the exact code and
  path, and exactly one diagnostic. Reverting the owner or `WireExpression.source` mapping
  would turn those into `invalid_wire_format` at `expression` and fail them. The
  check-before-comparison ordering on the binding path is checked: an owner of revision
  2^53+1 against a bound owner of revision 1 must give `invalid_requirement_revision`, not
  `malformed_reference`. Exact 2^53 is admitted on both ends. FND-001 and FND-002 cover
  what the tests do not reach.
- **Gates, run by this review in its own worktree and target dir.** `make fmt-check`,
  `make lint` (both lanes), `make test` (333 + 140 + 7 tests plus doctests, all pass),
  `make corpus` (107 match) and `make deny` (advisories, bans, licenses, sources ok, plus
  the one-copy gate) all pass. `quire validate` exits 0. Grammar findings: 1 (the existing
  FR-014 `ac:vague-response`). `quire coverage --strict`: 23 unbacked rows and 0
  contradicted, the same as main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The TC-015 expression-decoder tests reach only the owner, the root expression's `source` and one value declaration's `source`. A type declaration's `source`, an enum variant, a record field, a function, a function parameter and a quantifier's `local_source` are never driven above 2^53. Reverting any one of those fields to a serde `SourceSpan` would pass every test. The test's comment says "Every span member of the request goes through the same mapping". Drive one offset above 2^53 through each span-bearing member in a loop of pointers, as the TC-016 `members()` table already does | crates/quire-contract-model/src/wire.rs:1022-1030 |
| FND-002 | low | None of the behaviour changes this PR makes is tested through the expression operation. These are: a zero line or column, reversed endpoints and cross-source endpoints now `invalid_source_span`; a zero source revision in a span now `invalid_source_revision`; unknown members in `owner` and a span now refused; and the unlisted change that a malformed owner package or requirement id now reports `invalid_package_namespace`/`invalid_identifier` at `reference.*`. Each is a new observable code. Add one assertion per change, so a later edit cannot silently move them back to `invalid_wire_format`. Also list the owner-identifier change in the PR body | crates/quire-contract-model/src/wire.rs:974-1035 |

## Verdict

The code change is correct and minimal. Each span and owner field now builds through the
package decoder's own wire types and constructors, so the registered codes and paths
match the package decoder exactly. The behaviour changes it brings are the ones FR-011-AC-3,
FR-012-AC-6 and the closed published schema already require, and no producer is affected.
All gates pass.

The two findings are low test-coverage gaps, and neither blocks merge alone. The matrix
status questions are in the gap analysis (SR-1227).

Verdict (code-review lane): approve with low findings. Merge depends on SR-1227.

## Dispositions

Round 1 at 3c0d4f2460dbda2cb87e34fc8e768fda23ef29e4, the fix commit on top of
8afbebdb0b7f6e3ad8aedc0715808e6e737feec5. I reviewed only the delta (`git diff
8afbebd..3c0d4f2`). It touches tests and matrix rows only, with no production-code change:
`crates/quire-contract-model/src/wire.rs` (tests module), `tests/it/executable_binding.rs`,
`spec/core/matrix/tests.md` and `spec/tests.md`.

FND-001. `tc_015_every_span_member_of_an_expression_request_maps_the_offset_bound` walks
every `{start,end}` object in the corpus `expression-quantifier.json` input. It asserts
that eight sites are present: `/types/0/source` (enum), `/types/0/variants/0/source`,
`/types/1/source` (record), `/types/1/fields/0/source`, `/values/0/source`,
`/functions/0/source`, `/functions/0/parameters/0/source` and `/expression/source`. It
also asserts that a `local_source` is present. Each span found is then driven to
2^53 (admitted) and to 2^53+1, which must give `invalid_source_span` at `source_span`.

I ran two mutation probes in my throwaway worktree and reverted both:

- `local_source` reverted to a serde `SourceSpan` fails the test at
  `/expression/local_source/end/byte_offset`.
- `WireEnumVariant.source` reverted to a serde `SourceSpan` also fails it.

FND-002. `tc_015_the_expression_operation_maps_span_structure_and_owner_identity_codes`
asserts each change through `execute_expression`, with the exact code and path and one
diagnostic:

- zero line, zero column and reversed byte offsets: `invalid_source_span`;
- cross-source endpoints: `invalid_source_span`;
- zero span source revision: `invalid_source_revision` at `source.revision`;
- empty owner package: `invalid_package_namespace` at `reference.package`;
- empty owner requirement: `invalid_identifier` at `reference.requirement`;
- an extra member in `/owner`, `/expression/source` and `/expression/source/start`:
  `invalid_wire_format` at `expression`.

The last uses the expression operation's own decode-failure path (`wire_type_error("expression")`,
the same one the TC-016 tests assert). The package decoder reports its unknown member at
`document` for its own operation (corpus `package-unknown-field`). Each operation names its
own document root, so the two are consistent. The PR body now lists the owner-identifier
change.

Gates at 3c0d4f2, run by me:

- `make fmt-check`, `make lint`, `make test`, `make corpus` and `make deny` all pass.
  `make test` ran 333 + 142 + 7 tests plus doctests; `make corpus` matched 107/107.
- `quire validate` exits 0, with 1 grammar finding (FR-014).
- Strict coverage is 23 unbacked rows and 0 contradicted.

No new findings in this lane.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 3c0d4f2460dbda2cb87e34fc8e768fda23ef29e4 |
| FND-002 | fixed | 3c0d4f2460dbda2cb87e34fc8e768fda23ef29e4 |
