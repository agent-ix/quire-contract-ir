---
id: SR-1135
title: "gap analysis of PR 275 (IR-532/IR-476 bind 23 operation-law unit tests to FR-038 ACs)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@f257d1fb2612a59e410249f8c78897165eee2501; crates/quire-contract-model/src/checked_package/v2/operations.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md, tests/it/executable_binding.rs"
review_set: subset
---
# SR-1135: gap analysis of PR 275

## Summary

Ticket: IR-532 (and its parent IR-476). Plan completion: not assessed. This review is scoped
to the PR diff and the FR-038 acceptance criteria it binds (AC-44, AC-56, AC-81 through
AC-86, AC-88), plus the matrix rows that describe them.

Measured with `quire coverage --scope . --json` at base 74087d2 and at head f257d1f, in the
same reviewer worktree:

- **Backed targets.** 225 to 232 of 263. The newly backed targets are FR-038-AC-81, AC-82,
  AC-83, AC-84, AC-85, AC-86 and AC-88. None was lost. No FR-038 target is unbacked at
  head. AC-44 and AC-56 were already backed and gain more tests.
- **Rust binding census.** Tagged went from 397 to 416 and bound from 395 to 414 (+19).
  The other 4 of the 23 (`tc_048_application_preimage_...`, `tc_048_group_references_...`
  and the two `tc_048_validate_application_keys_...` tests) were already bound by their
  `tc_048_` name. `operations.rs` has 5 unmatched tags at both base and head (`FR-322`,
  `SHA-256` and `FR-038-AC-70` prose tokens, all from before this PR). The PR adds none.
- **`make spec` strict.** 23 unbacked rows, 0 contradicted, and the same row set as base.
  `quire validate` passes, with 1 grammar finding (FR-014, the known baseline). The
  newly backed AC targets were not among the 23 strict rows, so strict does not drop.
- **IR-476 scope**, read as data (`linear issue view IR-476`) and measured again: (1)
  untraced unit tests in `operations.rs`: 66/66 now carry `#[trace]`; (2) bare
  `/// TC-035` tags in `executable_binding.rs`: none left, and 13/13 tests carry
  `#[trace]` (#247, merged). Both items are met, so `Closes IR-476` is accurate for the
  code. The matrix finding below is the one loose end of IR-532.
- **AC clause coverage of the bound ACs.** Every clause of AC-56, AC-81, AC-82, AC-84,
  AC-85, AC-86 and AC-88 is asserted by a test bound to that AC. AC-44's leaf
  mode-type-mismatch clause is asserted by the leaf test, now bound. One clause of AC-83 is
  asserted only by a test bound to a different AC (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Both matrix rows that describe AC-81 through AC-88 still say they are untagged and planned. The FR-038 row says "AC-81 through AC-88 ... are planned ... those tests carry no `#[trace]` tag yet ... so no other row is backed until the binding PR tags them". The TC-048 row says "AC-81 through AC-88 are planned (IR-532 spec, then a binding code PR) and are not yet tagged". After this PR, coverage reports AC-81 to AC-86 and AC-88 as backed. The status prose now contradicts the code, and `--strict` cannot catch it (0 contradicted), because the row status is the whole-row 🚧. This PR is that binding PR, so it should rewrite both sentences to implemented, naming the `operations.rs` unit tests (and AC-44's added leaf test) | spec/checked_package/matrix/tests.md:15, spec/checked_package/matrix/tests.md:23 |
| FND-002 | low | AC-83's second clause says the `rounding` mismatch is refused when the domain is reached "by a `reference` ... and, separately, by a `literal` whose `type` names it". The reference path is bound (`operation_defect_refuses_mode_type_mismatch_on_operand`). The literal path is asserted only in `operation_defect_checks_literal_and_application_operands` (the "Mode pin" block), which this PR binds to AC-85 alone. The file already uses multi-AC traces (`#[trace("TC-048", "FR-038-AC-69", "FR-038-AC-98")]` at line 6834). Fix: `#[trace("TC-048", "FR-038-AC-83", "FR-038-AC-85")]` with the matching `Tracing:` line | crates/quire-contract-model/src/checked_package/v2/operations.rs:6227-6237 |

## Verdict

The bindings are sound and the coverage gain is real: 7 FR-038 AC targets backed, none lost,
strict unchanged at the 23-row baseline, and no contradicted row. FND-001 (medium) blocks
merge: the matrix still says what this PR makes false, and the fix is a two-sentence edit
in this PR. FND-002 is a small trace completion. Verdict: changes requested (FND-001).

## Dispositions

Round 1 at ec5ac123f3e93d9cf75c09eb9665e37681aff977. The delta against f257d1f is: the FR-038
and TC-048 rows of `spec/checked_package/matrix/tests.md`, the checked-package line of
`spec/tests.md`, the leaf-test doc comment, and one trace attribute. The full diff against
base still changes no non-doc, non-attribute code line.

Measured again at head: `quire validate` passes with grammar 1 (FR-014). Strict is 23
unbacked rows and 0 contradicted. Backed is 232/263. Rust tagged is 416 and bound is 414.
`operations.rs` has 5 unmatched tags, the same as base. FR-038-AC-44, AC-56 and AC-81
through AC-88 are all backed. AC-87 was already backed at base, by the operand-classification
test's `#[trace("TC-048", "FR-038-AC-87")]` (introduced by #253, afb01a2). It was not in the
newly-backed set, so the matrix's "AC-87 already tagged" claim holds.

FND-001. Both matrix rows and `spec/tests.md` now say AC-81 through AC-88 are implemented
and verified by TC-048 in the `operations.rs` unit tests, each tagged with its AC. No
"planned" or "not yet tagged" wording is left for that range.

FND-002. `operation_defect_checks_literal_and_application_operands` now carries
`#[trace("TC-048", "FR-038-AC-83", "FR-038-AC-85")]` with the matching `Tracing:` line, so
AC-83's literal-path clause is bound.

No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ec5ac123f3e93d9cf75c09eb9665e37681aff977 |
| FND-002 | fixed | ec5ac123f3e93d9cf75c09eb9665e37681aff977 |
