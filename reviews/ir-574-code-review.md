---
id: SR-11750
title: "IR-574 Slice 1 FR-023-AC-6 projection boundary tests code review (with Rust review)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #412 diff origin/main...HEAD: tests/it/executable_binding.rs, spec/model/matrix/tests.md, spec/core/matrix/tests.md"
review_set: subset
---

# IR-574 Slice 1 FR-023-AC-6 projection boundary tests code review (with Rust review)

## Summary

Ticket: IR-574. PR: quire-contract-ir#412. The PR has three files, with 67 lines added and 25 removed. This is one pass of code review with the rust-review lane folded in, scoped to the PR diff.

The PR replaces `tc_015_a_binding_expression_admits_a_revision_and_offset_of_exactly_two_to_the_53` with `tc_035_projection_revisions_and_offsets_obey_the_schema_boundary`, traced to TC-015, TC-035 and FR-023-AC-6. The new test builds one minimal projection with every revision and byte-offset member at 2^53 and asserts that it binds. It then sets each of the five AC-6 members in turn to 2^53+1 and to `u64::MAX`, and for each case asserts:

- exactly one diagnostic;
- code `InvalidWireFormat`;
- path `projection`;
- none of the three registered codes.

It also flips the FR-023 matrix rows.

Focused gates, run with `CARGO_TARGET_DIR` set to the worktree's `target/` under `flock /tmp/luna-ir-heavy.lock` and `-j2`:

- `cargo test --test it tc_035_`: 14 passed.
- `cargo fmt --check`: clean.
- `cargo clippy --test it -- -D warnings`: clean.

Mutation probes were run in a throwaway detached worktree at the same head, against the new test only:

| Mutation | Result |
| --- | --- |
| D1: `validate_schema` moved after full typed decoding | killed (`InvalidSourceRevision`) |
| D2: schema stage removed | killed |
| C1: schema refusal code changed to `InvalidSourceRevision` | killed |
| C2: schema refusal path changed to `projection.package` | killed |
| S1: package `sourceIdentity.revision` bound dropped | killed (`InvalidSourceRevision`) |
| S2: package requirement `revision` bound dropped | killed (`InvalidRequirementRevision`) |
| S3: conformance `requirementRef.revision` bound dropped (binding clause reference) | **survived** |
| S4: conformance `sourceIdentity.revision` bound dropped (expression source identity) | killed (`InvalidSourceRevision`) |
| S5: conformance `byte_offset` bound dropped (expression byte offset) | killed (`InvalidSourceSpan`) |
| S6 (control): package `byte_offset` bound dropped, not one of the five members | survived, as expected |

## Verdict

**FAIL until FND-001 is fixed.** The fix is a one-line assertion. FND-002 is low.

- **All five members and all three values.** Package source revision, package requirement revision, binding clause reference revision, expression source-identity revision and expression end byte offset are each tested at 2^53+1 and at `u64::MAX`. The 2^53 baseline is the same projection that each refusal case mutates, so "the same projection with that member at 9007199254740992 binds" holds literally. The baseline also sets the expression owner revision and the requirement and clause span revisions to 2^53, which keeps the identities consistent.
- **No skippable assertions.** No `if let` guards an assertion. Every refusal goes through `expect_err`, and the baseline goes through `expect`.
- **Registered codes absent.** All three registered codes are asserted absent. Given `len() == 1` and `code == InvalidWireFormat`, that check is redundant, but it does no harm.
- **Reaches the intended path.** Moving decode before the schema, removing the schema stage, using the wrong code or the wrong path each turns the test red. Dropping one member's schema bound turns it red for four of the five members (see FND-001).
- **Matrix flips.** The FR-023 row in spec/model/matrix/tests.md and the FR-023-AC-6 clauses of the FR-011 and FR-012 rows in spec/core/matrix/tests.md changed from planned to implemented. Nothing else changed. No corpus file, probe `clause_resolutions` input or coverage trace input is touched or pinned. The computed matrix (`quoin matrix --json`, quoin 0.28.3) binds the new test to FR-023-AC-6 only, with static status `tagged`.
- **Rust surface.** This is test-only code: no unsafe, and the `unwrap`s are on fixture pointers. fmt and clippy are clean.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The over-bound oracle asserts only code `InvalidWireFormat` and path `projection`. The typed serde envelope stage produces the same pair: `Binding.clause: ClauseRef` fails to deserialize and `decode_bounded` returns `invalid("invalid projection envelope")`. Dropping the schema bound on the binding clause reference revision (conformance schema `requirementRef.revision`) leaves the test green. For that member, the test cannot show that the schema stage decides the value before any typed decoder runs, which FR-023 requires. Asserting `diagnostics[0].message == "projection does not satisfy its normative schema"` kills the mutation and passes on the PR head (verified). Adding the over-bound documents to the independent schema negative controls would also work. | tests/it/executable_binding.rs:189-205 |
| FND-002 | low | The test keeps the `TC-015` trace tag. TC-015 is the test case behind FR-011-AC-3 and FR-012-AC-6, and this test asserts that their codes do not apply to the projection. The PR deleted the doc comment that said the test is not evidence for those criteria. The computed matrix does not bind the test to those ACs, but anyone reading by TC id will count it under TC-015. | tests/it/executable_binding.rs:138-139 |

## Examined scope

```yaml
scope:
- id: FR-023-AC-6
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: "A projection whose package source revision, package requirement revision, binding clause reference revision, expression source-identity revision or expression byte offset is 9007199254740993 (and, for each, `u64::MAX`) is refused `invalid_wire_format` at path `projection`, with no partial BoundPackage, and never `invalid_requirement_revision`, `invalid_source_revision` or `invalid_source_span`; the same projection with that member at 9007199254740992 binds."
- id: tc_035_projection_revisions_and_offsets_obey_the_schema_boundary
  path: tests/it/executable_binding.rs
  role: examined
  excerpt: "for value in [BOUND + 1, u64::MAX] { ... assert_eq!(diagnostics.len(), 1 ...); assert_eq!(diagnostics[0].code, DiagnosticCode::InvalidWireFormat ...); assert_eq!(diagnostics[0].path, \"projection\" ...); assert!(diagnostics.iter().all(|diagnostic| !matches!(diagnostic.code, InvalidSourceRevision | InvalidRequirementRevision | InvalidSourceSpan)) ...) }"
- id: spec/model/matrix/tests.md#FR-023
  path: spec/model/matrix/tests.md
  role: examined
  excerpt: "| FR-023 | FR-023-AC-1 through FR-023-AC-6 | TC-035 | ✅ implemented: AC-1 through AC-5 and AC-6, which checks all five revision and byte-offset members at 2^53, 2^53 + 1, and `u64::MAX` through the public projection binder (TC-035). |"
- id: spec/core/matrix/tests.md#FR-011-FR-012
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "a whole executable projection first meets its published schema and refuses an over-2^53 revision as `invalid_wire_format` at `projection` (FR-023-AC-6, implemented in TC-035)."
- id: FR-011-AC-3
  path: spec/core/functional/FR-011-package-identity.md
  role: context_only
  excerpt: "zero source or requirement revisions and a revision above 9007199254740992 (`invalid_source_revision`, `invalid_requirement_revision`, through the constructors, the package decoder, the expression operation decoder and the binding expression decoder; a whole executable projection is FR-023-AC-6's)"
- id: FR-012-AC-6
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: context_only
  excerpt: "a whole executable projection refuses such an offset `invalid_wire_format` at its schema stage (FR-023-AC-6), and the probe and coverage inputs of the conformance corpus refuse it `invalid_corpus` at the fixture schema (FR-020-AC-3)."
- id: crates/quire-contract-model/src/binding.rs
  path: crates/quire-contract-model/src/binding.rs
  role: context_only
  excerpt: "validate_schema(&value)?; let projection: Projection = serde_json::from_value(value).map_err(|_| invalid(\"invalid projection envelope\"))?;"
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | asserts `diagnostics[0].message == "projection does not satisfy its normative schema"` for every member and value. Re-run in a throwaway worktree: the mutant that removes the schema limit on the binding clause reference revision (conformance `requirementRef.revision`) now fails ("binding clause reference revision at 9007199254740993", left "invalid projection envelope"). `tc_035_` passes 14 of 14, and fmt and clippy are clean. |
| FND-002 | fixed | removes `TC-015` from the test's trace; it is now `#[trace("TC-035", "FR-023-AC-6")]`. TC-015 keeps its own binder in tests/it/identity.rs. |
