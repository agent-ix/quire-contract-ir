---
id: SR-1210
title: "PR #282 FR-034-AC-7 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@75f880ea12a818f63282971e557e580759da8711; FR-034-AC-7, TC-043; crates/quire-contract-model/src/output_mapping.rs (admission and unit tests), tests/it/output_mapping.rs, spec/output_mapping/matrix/tests.md, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-043
    type: reviews
---
# SR-1210: PR #282 FR-034-AC-7 gap analysis

## Summary

Ticket: IR-567. PR agent-ix/quire-contract-ir#282. Plan completion: not
assessed.

The PR flips FR-034-AC-7 to implemented on the strength of existing tests. All
six tests carry `#[trace(..., "FR-034-AC-7")]`. They were run at the reviewed
sha (`cargo test --workspace tc_043`) and pass: 20 integration tests and 8 unit
tests.

| Test | File |
| --- | --- |
| `tc_043_request_identity_is_the_hand_written_text_under_its_ceiling` | `output_mapping.rs` unit |
| `tc_043_request_limits_at_two_to_the_64_enter_the_material_as_distinct_decimal_strings` | `output_mapping.rs` unit |
| `tc_043_record_identity_is_the_hand_written_text_under_its_ceiling` | `output_mapping.rs` unit |
| `tc_043_package_identity_is_the_hand_written_text_under_its_ceiling` | `output_mapping.rs` unit |
| `tc_043_record_and_package_identities_are_the_digests_of_hand_written_text` | `tests/it` |
| `tc_043_request_material_is_metered_and_admits_a_two_to_the_64_limit` | `tests/it` |

Clause by clause:

- **Record and package identities are SHA-256 of written-out text.** Met.
  - The integration test compares `record_id()` and `package_id()` with
    `sha2::Sha256::digest` of format strings written in the test.
  - The unit tests compare the step output byte for byte with written-out text.
  - The oracle's digest is `sha2` called directly, not the code under test.
  - The interpolated digests (clause declaration and expression, package
    digest, and the record id once it is verified) are inputs.
- **Request material equals written-out text.** Met at the request identity
  step: the unit test's `request_text` equals `request_identity_bytes` output
  byte for byte. Not met through admission; see FND-001.
- **`"18446744073709551615"` (request `maximum_emitted_bytes`).** Met in unit
  `request_text`. The value is written in quotes. As a JSON number past 2^53 the
  encoder would refuse it, so a number spelling fails the test.
- **`"9007199254740993"` (package `maximum_emitted_bytes`).** Met in unit
  `package_text`.
- **Region `"0"` and `"1"`.** Met in the integration `record_text`:
  `"output_regions":[{"end":"1","start":"0"}]`. A number spelling changes the
  bytes, so the digest comparison fails.
- **Region `"9007199254740993"` and `"18446744073709551615"`.** Met in unit
  `record_text`.
- **u64::MAX and u64::MAX-1 both admit.** Met through the public `admit` in the
  integration test.
- **u64::MAX and u64::MAX-1 have distinct material.** Met at the step only: the
  unit test compares each material with its own written-out text.

## Verdict

**APPROVE WITH NITS** (one medium and one low finding). Every spelling clause
has an independent, hand-written oracle that would fail if a bound were spelled
as a number. The flip is defensible for the record, package and spelling
clauses.

The request-material clause is proven only at the step seam. The
admission-to-material link that the AC's own wording ("whose length admission
keeps as `request_bytes`") asserts is untested. The matrix rows this PR edits
are left 🚧 with no open item named.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The request clause of FR-034-AC-7 is checked only on `RequestIdentityMaterial` and `RequestResourceShape` values the unit test builds itself. Admission builds its own (output_mapping.rs:2251-2267, wiring each `MappingLimits` member into `resource_shape`). No test compares `admit(...).request_bytes()` with the length of a written-out request text: the integration tests only self-measure `request_bytes` and check equal length, one over and one under. A miswired `resource_shape` member in admission would still pass every AC-7 test. The unit `shape()` also gives `maximum_obligations` and `maximum_records` the same value, 32, so a swap of those two members is invisible even at the step. Add an integration assertion that `request_bytes()` equals the length of a written-out request text for the fixture, with distinct values per limit, or reword AC-7 to say the request text is checked at the request identity step. | crates/quire-contract-model/src/output_mapping.rs:2251-2267,2828-2837,2919-2946; tests/it/output_mapping.rs:2033-2068 |
| FND-002 | low | Mixed-row status is inconsistent. The PR marks FR-034 ✅ but keeps TC-043 and the `spec/tests.md` Output mapping row 🚧, and its rewritten notes name no open clause. The reason given, that FR-032's row is 🚧 for other reasons, does not hold: the FR-032 and FR-033 rows were ✅ until #264 added AC-6 as planned, #277 implemented AC-6 without flipping them back, and their notes now record every AC as implemented. The 🚧 on FR-032, FR-033, TC-043 and the index row is stale. Flip all four to ✅ in this PR, or name what remains. | spec/output_mapping/matrix/tests.md:12-14,26; spec/tests.md:16 |

## New findings (disposition pass 1)

Round 1 was reviewed at 111d164c2ee950ceaf44a6732a3da64363f2e5df.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The new test `tc_043_admitted_request_bytes_are_the_length_of_the_hand_written_request_text` checks only the length of the admitted material, so it cannot detect two members swapped in admission's `resource_shape`: a swap keeps the total length. It also misses a wrong-member copy between values of the same digit count: 40/36 (`maximum_obligations`/`maximum_records`) or 1100/4200 (`maximum_expression_nodes`/`maximum_mapping_work`). It does catch a quoting change or a copy that changes the digit count. Because the check is length-only, FR-034-AC-8's request material with `maximum_emitted_bytes` 77777 carrying `"77777"` and the five others "respectively" has no test that checks names: the unit step test checks names only with `maximum_emitted_bytes` = u64::MAX. To check what admission writes, admission and a unit test could share one `MappingLimits`-to-`RequestResourceShape` builder, with the test comparing bytes to written-out text. Otherwise, state the 77777 case as a length claim. | tests/it/output_mapping.rs:2070-2129; crates/quire-contract-model/src/output_mapping.rs:2259-2266; spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md:107 |
| FND-004 | low | `tc_043_request_material_is_metered_and_admits_a_two_to_the_64_limit` is still tagged FR-034-AC-7. The u64::MAX and u64::MAX-1 clause it backed has moved to AC-9, and it has no written-out text, so it backs none of the new AC-7. Its tags should be FR-032-AC-6, FR-034-AC-6 and FR-034-AC-9. | tests/it/output_mapping.rs:2030-2033 |

## New findings (disposition pass 2)

Round 2 was reviewed at 0623b29960ab91eb422199da24e301732a613bad.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | FR-034-AC-8's request-material sentence (`maximum_obligations` 40, `maximum_expression_nodes` 1100, `maximum_nesting_depth` 130, `maximum_mapping_work` 4200, `maximum_records` 36 and `maximum_emitted_bytes` 77777, carrying `"40"` ... `"77777"` respectively) is backed by no test. The value 77777 appears in no test: in 0623b29 the integration test switched to 55/7777/4/88888/666/999999 and dropped its AC-8 tag. The unit tests write out 40/1100/130/4200/36 only with `maximum_emitted_bytes` 18446744073709551615 or 18446744073709551614. The FR-034 matrix note still lists `77777` among AC-8's spellings. Restate the sentence as the unit fixture: the five values with `maximum_emitted_bytes` 18446744073709551615. Drop `77777` from AC-8 and the matrix note. | spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md:107; spec/output_mapping/matrix/tests.md:14 |

## New findings (disposition pass 3)

Round 3 was reviewed at 1ef7c1f9d65b49f1d2d26b9df24bc5e6a623796a.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The new comment on `tc_043_admitted_request_bytes_are_the_length_of_the_hand_written_request_text` says "A swap keeps the length; the unit tests catch it by comparing bytes." Here "it" is a swap in the request material that admission builds. The unit tests never run admission: they build `RequestResourceShape` themselves (`shape()`, output_mapping.rs:2828-2837). So a swap in admission's wiring (output_mapping.rs:2259-2266) is caught by no test. AC-7 and the matrix note scope the catch to "the encoding step" and are defensible; the comment overstates it. Suggested comment: "A swap in admission keeps the length and is not detected here; the unit tests byte-check member names only for a shape they build." | tests/it/output_mapping.rs:2074-2077 |

## Dispositions

Round 1 was reviewed at 111d164c2ee950ceaf44a6732a3da64363f2e5df.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 111d164. New integration test `tc_043_admitted_request_bytes_are_the_length_of_the_hand_written_request_text` admits with limits 40/1100/130/4200/36/77777 and asserts `request_bytes()` equals the length of written-out request text. The unit fixtures now use distinct values (40, 1100, 130, 4200, 36), so a swap of obligations and records at the step fails byte equality. AC-7 now claims only the length through admission. The swap blind spot left in a length-only check is FND-003. |
| FND-002 | fixed | 111d164. FR-032, FR-033, TC-043 and the Output mapping index row are now ✅. Every FR-032-AC-1..6 and FR-033-AC-1..6 has at least two tagged tests, and all of them pass (`cargo test --workspace tc_04`: 213 integration + 97 unit, `tc_051`: 1). |
| FND-003 | still-open | Round 2, reviewed at 0623b29. Copies are now caught. The values 55, 7777, 4, 88888, 666 and 999999 have digit counts 2, 4, 1, 5, 3 and 6, all different, and `maximum_request_bytes` (1048576) has 7. A member copied from any other member, or from the request byte limit, therefore changes the length. Swaps are still not caught. Swapping two members' values moves the same strings between member names, so the total length and `request_bytes` stay the same whatever the digit counts. Yet AC-7 now says "(so a member swapped with or copied from another changes the length)", and the matrix note says the test "detects a member swapped or copied through admission". Both claims are false. Remove "swapped with or" and "swapped or" so the text claims copy detection only, or test the content as FND-003 suggested. |
| FND-004 | fixed | 0623b29. `tc_043_request_material_is_metered_and_admits_a_two_to_the_64_limit` is now tagged TC-043, FR-032-AC-6, FR-034-AC-6, FR-034-AC-9. |
| FND-003 | fixed | Round 3, 1ef7c1f. AC-7 now claims copy detection only: "a member copied from another member or from the request byte limit changes the length; a swap of two members' values keeps the length and is caught only at the encoding step, by the byte-equality unit tests whose values 40, 1100, 130, 4200 and 36 are pairwise distinct". The matrix note says the same. No swap claim remains. A residual overstatement in the test comment is FND-006. |
| FND-005 | fixed | Round 3, 1ef7c1f. AC-8 now reads "Request material with `maximum_obligations` 40, `maximum_expression_nodes` 1100, `maximum_nesting_depth` 130, `maximum_mapping_work` 4200, `maximum_records` 36 and `maximum_emitted_bytes` 18446744073709551615 carries `"40"`, `"1100"`, `"130"`, `"4200"`, `"36"` and `"18446744073709551615"` respectively." That is the unit `request_text("18446744073709551615")` fixture, checked byte for byte by `tc_043_request_identity_is_the_hand_written_text_under_its_ceiling`. 77777 is gone from `spec/`, `plan/`, `crates/`, `tests/` and `src/`. |
| FND-006 | fixed | Round 4, 50f4b3c. The comment now reads "A swap in admission keeps the length and is not detected here; the unit tests byte-check member names only for a shape they build." This is true: `shape()` is test-local, and admission's wiring at output_mapping.rs:2259-2266 is not exercised by a byte check. |
