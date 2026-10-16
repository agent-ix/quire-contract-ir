---
id: SR-11790
title: "IR-574 Slice 3 probe and coverage corpus bounds tests code review (with Rust review)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #414 diff origin/main...6741c65: tests/it/conformance.rs, spec/conformance/matrix/tests.md, spec/core/matrix/tests.md"
review_set: subset
---

# IR-574 Slice 3 probe and coverage corpus bounds tests code review (with Rust review)

## Summary

Ticket: IR-574. PR: quire-contract-ir#414 at 6741c658514c97043e489c5b74775a4ba04b886a. The PR changes three files, adding 138 lines and removing 7. It contains tests and matrix rows only. This is one pass of code review with the rust-review lane folded in, scoped to the PR diff.

The PR adds `tc_018_probe_and_coverage_bounds_fail_at_the_fixture_schema`, traced to TC-015, TC-018, FR-011-AC-3, FR-012-AC-6 and FR-020-AC-3. For each of six members, the test writes 2^53+1 and then `u64::MAX` into a scratch copy of the corpus, runs the runner, and asserts all of the following:

- exit code 2;
- empty stdout;
- `code == invalid_corpus`;
- `path == fixtures.<id>.input`;
- `detail == "instance does not match schema"`.

The six members are:

- the probe `clause_resolutions/0/requirement/revision` (package-orphan-clause);
- coverage `source.start.source.revision`;
- coverage `source.start.byte_offset`;
- coverage `target_span.start.source.revision`;
- coverage `target_span.end.byte_offset`;
- coverage `target.revision` (the last five all in coverage-deep).

For each case it also decodes the changed member directly with `serde_json::from_value` into `ClauseRef`, `RequirementRef` or `SourceSpan` and asserts that the error starts with the member's registered code. It then writes exactly 2^53 and asserts that the schema stage is passed. For `target_span.end.byte_offset`, the run exits 0 and the fixture row is `match`. For the other five members, the run fails later, at the corpus-wide `fixtures.covers` inventory check.

Focused gates were run in a throwaway detached worktree at the PR head, with `CARGO_TARGET_DIR` set to that worktree's `target/`, under `flock /tmp/luna-ir-heavy.lock` and with `-j2`:

- `cargo test --test it tc_018_probe_and_coverage_bounds`: 1 passed.
- `cargo fmt --check`: clean.
- `cargo clippy --locked --test it -- -D warnings`: clean.
- `quire validate` on both matrix files: rc 0.
- `quoin matrix --json` (quoin 0.28.3) binds the test to FR-011-AC-3, FR-012-AC-6 and FR-020-AC-3 with static status `tagged`.

Mutation probes, run in the throwaway worktree only and never on the PR branch:

| Mutation | Result |
| --- | --- |
| M1: fixture schema `revision` maximum raised to u64::MAX | killed: probe row 2^53+1 reaches `fixtures.covers` |
| M2: fixture schema `byteOffset` maximum raised to u64::MAX | killed: `source.start.byte_offset` row |
| M3: `revision` maximum removed | killed |
| M4: `revision` maximum lowered to 2^53-1 | killed by the exact-2^53 control (`fixtures.package-orphan-clause.input`) |
| M5: `byteOffset` maximum lowered to 2^53-1 | killed by the control |
| M6: `requirementRef.revision` retyped to `u64` | killed |
| M7: `sourceLocation.byte_offset` retyped to `u64` | killed |
| M8: `artifactTrace.target` alone loosened to u64 revision (per-member) | killed by the `target.revision` row |
| M9: test table with the `target.revision` row removed, against M8 | survived. This shows that the row has independent kill power. |
| M10: test table with the `target.revision` row removed, against M6 | killed by the probe row (shared definition) |
| M11: wrong expected code (`invalid_source_span` changed to `invalid_source_revision`) | killed |
| M12: wrong expected code on the `target.revision` row | killed |

## Verdict

**PASS, mergeable.** The only finding is FND-001, which is low and non-blocking.

- **All six members and both over-bound values.** Each of the six members named by the brief is driven at 2^53+1 and at `u64::MAX`.
- **Code, path and schema stage.** Each case asserts the code, the path and the schema-stage detail string. A later-stage failure has a different path (`fixtures.covers`), so it cannot satisfy the assertion. M1, M2 and M3 show this.
- **The exact-2^53 control is not vacuous.** Lowering either bound to 2^53-1 turns it red (M4, M5). Deleting the bound turns the over-bound rows red (M3), so the test does not pass when the schema bound is removed.
- **Per-member rows carry weight.** A schema change confined to one member's context is caught by that member's row (M8 vs M9).
- **Direct decode uses only the public API.** It uses re-exported `ClauseRef`, `RequirementRef` and `SourceSpan` with serde, and the wrong-code mutants are killed (M11, M12).
- **No skippable assertions.** The one `if` on the pointer asserts in both arms, and every decode is `unwrap_err`.
- **Isolation and determinism.** The scratch directory is unique per pid and label (`probe-coverage-bounds`), and no other test uses that label. It is cleaned up by `Drop`, and the original input is restored after each member.
- **Matrix flips are truthful.** FR-020 stays ✅ and gains a sentence that describes exactly what the test asserts. The FR-012 row adds TC-018 and stays 🚧. No other row changes.
- **Rust surface.** The change is test-only: no unsafe and no production change. The `unwrap`s are on fixture pointers and runner output, and fmt and clippy are clean.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | For five of the six members, the exact-2^53 control asserts the corpus-wide `fixtures.covers` / "observed coverage union differs from inventory" error. That error is incidental: the edited fixture now produces a different diagnostic than the inventory records. The control proves that the schema stage was passed, and lowering the bound kills it (M4, M5). It does not show whether the typed decoder accepts 2^53, though: with the schema bound raised (M1), an over-bound value gives the same `fixtures.covers` signature. The control is also coupled to an unrelated inventory check, so a reorder of the corpus-level checks or a change to the inventory set would break it with no change to the bound. A tighter control would assert that the run gets past `fixtures.<id>.input` in a way that does not depend on the inventory, for example by updating the matching expectation in the scratch copy so the run exits 0. | tests/it/conformance.rs:1014-1045 |

## Examined scope

```yaml
scope:
- id: FR-020-AC-3
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: "the package schema accepts a revision and a byte offset of `9007199254740992` and rejects `9007199254740993`; a decoder fixture that supplies a JSON number in one of the eight members straight to the decoder returns `invalid_wire_format` ... while the same number in a valid-operation input is `invalid_corpus` at the schema check"
- id: FR-012-AC-6
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: "Source spans reject ... a byte offset above 9007199254740992 ... using `invalid_source_span` ... and the probe and coverage inputs of the conformance corpus refuse it `invalid_corpus` at the fixture schema (FR-020-AC-3)."
- id: FR-011-AC-3
  path: spec/core/functional/FR-011-package-identity.md
  role: examined
  excerpt: "zero source or requirement revisions and a revision above 9007199254740992 (`invalid_source_revision`, `invalid_requirement_revision`, through the constructors, the package decoder, the expression operation decoder and the binding expression decoder; a whole executable projection is FR-023-AC-6's)"
- id: FR-023-AC-6
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: context_only
  excerpt: "A projection whose ... revision ... or expression byte offset is 9007199254740993 (and, for each, `u64::MAX`) is refused `invalid_wire_format` at path `projection` ..."
- id: tc_018_probe_and_coverage_bounds_fail_at_the_fixture_schema
  path: tests/it/conformance.rs
  role: examined
  excerpt: "for value in [BOUND + 1, u64::MAX] { ... assert_eq!(error[\"code\"], \"invalid_corpus\"); assert_eq!(error[\"path\"], format!(\"fixtures.{fixture}.input\")); assert_eq!(error[\"detail\"], \"instance does not match schema\"); ... decoded.to_string().starts_with(&format!(\"{code} at \")) }"
- id: spec/conformance/matrix/tests.md#FR-020
  path: spec/conformance/matrix/tests.md
  role: examined
  excerpt: "`tc_018_probe_and_coverage_bounds_fail_at_the_fixture_schema` verifies each probe clause-reference revision and coverage trace revision/offset member at 2^53+1 and `u64::MAX` fails at its fixture input with `invalid_corpus`, while 2^53 passes that schema."
- id: spec/core/matrix/tests.md#FR-012
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "TC-018 verifies that over-bound coverage trace offsets instead meet their fixture schema before typed decoding and return `invalid_corpus` at `fixtures.coverage-deep.input`; the exact bound passes that schema."
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 46edc40. At exactly 2^53, every member is now decoded directly with the public API (`ClauseRef`, `RequirementRef` or `SourceSpan`), with the other endpoint kept consistent, and the test asserts `decoded.is_ok()`. That shows the typed decoder accepts the bound. The runner control still asserts `fixtures.covers`, which proves only that the schema stage was passed. It no longer depends on the inventory message text, so that coupling is accepted as the stage check. Re-measured in a throwaway worktree at 46edc40. Raising the fixture-schema `revision` maximum is killed. Making the model revision bound exclusive (`>=`) is killed directly by the new control at the first member. With the control removed, that mutant is caught only incidentally, when the unrelated `package-revision-maximum` fixture fails the run. With the control removed and no mutant, the test stays green, so the control is the direct kill. |
