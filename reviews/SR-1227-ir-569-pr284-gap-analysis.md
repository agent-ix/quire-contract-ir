---
id: SR-1227
title: "gap analysis of PR 284 (IR-569 registered revision and span codes through the expression decoder)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@8afbebdb0b7f6e3ad8aedc0715808e6e737feec5; spec/core/functional/FR-011-package-identity.md, spec/core/functional/FR-012-anchors-clauses-dependencies.md, spec/core/functional/STD-001-diagnostic-registry.md, spec/model/functional/FR-013-type-system.md, spec/model/functional/FR-023-executable-projection-binding.md, spec/core/matrix/tests.md, spec/tests.md, crates/quire-contract-model/src/wire.rs, crates/quire-contract-model/src/identity.rs, crates/quire-contract-model/src/binding.rs, crates/quire-contract-model/src/conformance.rs, schemas/contract-executable-projection-v1.schema.json, schemas/contract-conformance-fixture-v1.schema.json, tests/it/executable_binding.rs"
review_set: subset
---
# SR-1227: gap analysis of PR 284

## Summary

Ticket: IR-569. Plan completion: not assessed. This review checks the merged spec on main
543cd8a8b6a5fe2a1974617869824a75f6c38629 against the code, tests and matrix rows at
8afbebdb0b7f6e3ad8aedc0715808e6e737feec5.

Units examined:

- FR-011-AC-3. It says that zero source or requirement revisions "and a revision above
  9007199254740992 (`invalid_source_revision`, `invalid_requirement_revision`) ... fail
  with their registered structured diagnostic codes". The clause names no operation or
  decoder.
- FR-012-AC-6. It says "Source spans reject zero line/column positions, decreasing byte
  offsets or positions, a byte offset above 9007199254740992 ..., and endpoints from
  different source-document identities/revisions using `invalid_source_span`."
- FR-013-AC-5 (must not regress).
- FR-023 Behavior ("Limit or schema failure returns structured diagnostics") and
  FR-023-AC-5 ("normative projection schema positive and negative controls agree with the
  decoder").
- STD-001 rows `invalid_requirement_revision`, `invalid_source_revision` and
  `invalid_source_span`, with their required locations.
- The TC-015 rows: the FR-011 and FR-012 rows of `spec/core/matrix/tests.md`, and the Core
  row of `spec/tests.md`.

Test and AC bindings, each read:

| Test | AC | Result |
| --- | --- | --- |
| `wire::tests::tc_015_the_expression_operation_refuses_..._registered_code` | FR-011-AC-3, FR-012-AC-6 | correct. Admits exact 2^53. Refuses 2^53+1 and zero for the owner, 2^53+1 for a span offset, a span source revision and a value declaration's span, with the exact code and path and one diagnostic |
| `wire::tests::tc_015_a_binding_expression_refuses_..._registered_code` | FR-011-AC-3, FR-012-AC-6 | correct. Exercises `check_expression_input` with a binding, and the code comes before the owner comparison |
| `executable_binding::tc_015_a_binding_expression_admits_..._exactly_two_to_the_53` | FR-011-AC-3, FR-012-AC-6 | correct for its admission half (the 2^53 boundary). Its refusal half pins `invalid_wire_format` at `projection`, the opposite of the AC's code (FND-001) |
| `wire::tests::tc_016_*` (three) | FR-013-AC-5 | correct and unchanged. Passes at this head |

Behaviour measured by this review with a throwaway probe test, run in its own worktree and
never committed. Each input is a whole projection through `BoundPackage::from_json_bytes`:

- Binding-expression owner revision 0: `invalid_requirement_revision` at
  `reference.revision`.
- Binding-expression span source revision 0: `invalid_source_revision` at
  `source.revision`.
- Binding-expression span line 0: `invalid_source_span` at `source_span`.
  The fixture schema's `revision` and `line` have minimum 0, so zero values reach the
  decoder.
- `bindings[0].clause.requirement.revision` 0 or 2^53+1: `invalid_wire_format` at
  `projection`.
- Unknown member in the binding-expression owner: `invalid_wire_format` at `projection`,
  because the schema closes the object.
- A binding-expression byte offset of 2^53+1: `invalid_wire_format` at `projection`
  (the PR's own test).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-011 and FR-012 matrix rows flip to ✅, but the over-2^53 clause of FR-011-AC-3 and FR-012-AC-6 does not hold on the public projection decoder. `BoundPackage::from_json_bytes` checks the projection schema before decoding. That schema embeds the fixture schema's `revision`/`byteOffset`, which cap at 2^53, so a projection carrying 2^53+1 refuses `invalid_wire_format` at `projection` and never `invalid_requirement_revision`, `invalid_source_revision` or `invalid_source_span`. The AC text has no decoder qualifier. The row states the deviation in prose but still marks it ✅. The PR also tags a test asserting `invalid_wire_format` to FR-011-AC-3 and FR-012-AC-6. This is a spec conflict between FR-011-AC-3/FR-012-AC-6 and FR-023's schema-first stage, so an owner decides. Until then, keep both rows 🚧, naming the projection decoder's over-2^53 case with a linked ticket for that decision, and keep the Core index row's partial note. Alternatively, record a spec change (via /spec-review) that scopes the clause or sets schema-failure precedence in FR-023 or STD-001, and only then mark ✅ | spec/core/matrix/tests.md:20-21, tests/it/executable_binding.rs:138-164, crates/quire-contract-model/src/binding.rs:424-445 |
| FND-002 | medium | Serde-derived revision fields remain outside the expression decoder, and the FR-011 row's ✅ claim covers them. The row says the registered codes hold "through ... the executable-projection binding path". But `Binding.clause: ClauseRef` (binding.rs:110) still deserializes `RequirementRevision` through serde. A zero clause revision in `bindings[].clause` (which the fixture schema admits, minimum 0) refuses `invalid_wire_format` at `projection`, measured by this review's probe. The same serde-derived `ClauseRef`/`RequirementRef`/`SourceSpan` fields remain in the package-probe `clause_resolutions` and the coverage operation's `WireArtifactTrace.source`/`target`/`target_span` and `WireTraceDepth.digest_span` (conformance.rs:1906, 2000-2012), so zero revisions and bad spans there are `invalid_wire_format`, not FR-011-AC-3/FR-012-AC-6's codes. IR-569's own scope is the expression decoder, so these need not be fixed here. Either narrow the row so it claims only the expression input, and file a ticket for the binding clause, package-probe and coverage inputs, or route those fields through the same wire types in this PR | crates/quire-contract-model/src/binding.rs:110, crates/quire-contract-model/src/conformance.rs:1906, crates/quire-contract-model/src/conformance.rs:2000-2012, spec/core/matrix/tests.md:20 |
| FND-003 | low | This finding predates the PR and is not introduced by it. STD-001's registry rows give the conditions `invalid_requirement_revision` "Zero or non-increasing requirement revision" and `invalid_source_revision` "Zero source-document revision". Neither names a revision above 2^53, which FR-011-AC-3 assigns to those codes since IR-274. Under STD-001's guidance, a new failure class needs a registry row. Ticket the registry wording; it is not for this PR | spec/core/functional/STD-001-diagnostic-registry.md:36-37 |

## Verdict

The code meets IR-569's stated scope. Through the expression operation and
`check_expression_input` with a binding, every owner and span member reports the
registered code and path for zero and over-2^53 values. Inside a projection, zero values
also reach those codes. FR-013-AC-5 does not regress. All gates pass, `quire validate`
exits 0, strict coverage is 23 rows (main's baseline), and grammar has 1 finding (FR-014).

The matrix status is what is wrong. The ✅ on FR-011 and FR-012 claims more than the code
does:

- The over-2^53 clause is false on the public projection decoder, because the schema check
  runs first (FND-001).
- Serde-derived revision fields remain on the projection's binding clause and in the
  package-probe and coverage inputs (FND-002).

Both are row-text fixes plus tickets. Neither needs a code change.

Verdict: changes requested (FND-001, FND-002). Mergeable after the rows are corrected (or
the spec decision is recorded) and the follow-up tickets exist.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The FR-012 row's "Unmet (IR-574)" note names only the whole-projection path. The coverage operation's `WireArtifactTrace.source`/`target_span` and `WireTraceDepth.digest_span` are still serde `SourceSpan`s, so a zero line or column, reversed or cross-source endpoints, or an offset above 2^53 there give `invalid_wire_format`, not FR-012-AC-6's `invalid_source_span`. The FR-011 row and the PR body name the coverage traces, but only for revisions. IR-574's item (2) also frames those sites as "a bad revision". Add the coverage spans to the FR-012 row's unmet note (or to IR-574's text) so the residue is stated where FR-012 is tracked | spec/core/matrix/tests.md:21, crates/quire-contract-model/src/conformance.rs:2000-2012 |

## Dispositions

Round 1 at 3c0d4f2460dbda2cb87e34fc8e768fda23ef29e4, the fix commit on top of
8afbebdb0b7f6e3ad8aedc0715808e6e737feec5. The delta touches tests and matrix rows only. I
checked every finding against the code and rows myself. I did not rely on the coder's
account, because the coder worked without reading this file.

FND-001.

- The FR-011 and FR-012 rows are back to 🚧. Each says what is implemented (the
  constructors, the package decoder, the expression operation and the binding expression
  decoder) and what is unmet: the whole-projection path, which the schema refuses first.
  IR-574 is cited.
- The `spec/tests.md` Core row states the same split, so the two files are consistent.
- IR-574 exists, is in Backlog, and records the spec decision needed on FR-023's
  schema-first stage versus the ACs.
- `tc_015_a_binding_expression_admits_...` is retagged `#[trace("TC-015")]` only. Its doc
  comment says it pins today's schema-first behaviour and is not evidence for FR-011-AC-3
  or FR-012-AC-6. That is right: its refusal half asserts the non-registered code.
- Strict coverage is unchanged at 23 rows. The ACs stay backed by the `wire::tests`
  TC-015 tests.

FND-002 (fixed by narrowing the claim, which is what the finding asked). The FR-011 row narrows its claim to the binding expression decoder. Its unmet
note names `Binding.clause`, the package probe's `clause_resolutions` and the coverage
artifact traces, citing IR-574 item (2). These are not fixed in this PR; the row names them
and the ticket tracks them. FR-012's span side of the coverage traces is not named; see
FND-004.

FND-003. The STD-001 registry wording is deferred to IR-574 item (3). The FR-011 row says
so.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 3c0d4f2460dbda2cb87e34fc8e768fda23ef29e4 |
| FND-002 | fixed | 3c0d4f2460dbda2cb87e34fc8e768fda23ef29e4 (row narrowed to the expression operation and binding expression decoder, remaining sites named unmet and ticketed as IR-574 item (2)) |
| FND-003 | deferred | IR-574 item (3) tracks the STD-001 registry wording; it predates this PR and the FR-011 row cites it |

Round 2 at 092b2087e513f570d89cc999163eb711258732db. The delta from 3c0d4f2 changes two
lines of wording and nothing else: the FR-012 row of `spec/core/matrix/tests.md` and the
Core row of `spec/tests.md`.

FND-004.

- The FR-012 "Unmet (IR-574)" note now names the coverage operation's artifact-trace spans:
  the `WireArtifactTrace` source and target spans and the `WireTraceDepth` digest span. The
  note says they are still decoded by serde and give `invalid_wire_format` rather than
  `invalid_source_span`.
- The Core index row says the same.
- I checked the wording against `conformance.rs` at this head. `WireArtifactTrace.source`
  and `target_span`, and `WireTraceDepth::Deep.digest_span`, are serde `SourceSpan` fields
  (lines 2000, 2002 and 2012). A decode failure of `CoverageInput` returns
  `invalid_wire_actual("coverage")` (line 2039-2041). The wording is true.
- `make spec`: `quire validate` passes. It has 1 grammar finding (FR-014) and strict
  coverage of 23 unbacked rows and 0 contradicted, the same as main. Strict coverage exits
  non-zero on that baseline, as it does on main.
- Main has moved to ffb86d9fec6a61c64bc0eaa84ba0386575776d38 (#283). GitHub reports
  `MERGEABLE`/`BEHIND`, and `git merge-tree` against it is clean.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | 092b2087e513f570d89cc999163eb711258732db |
