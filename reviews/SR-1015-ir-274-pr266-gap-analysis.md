---
id: SR-1015
title: "gap analysis of PR 266 (IR-274 part A): FR-038-AC-89..95 against tests and code, with mutation probes"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@476e9b418e69aaaf7e796e3923e3d8321a812d50; FR-038-AC-89, AC-90, AC-91, AC-92, AC-93, AC-94, AC-95 against tests/it/checked_package_v2_identity_digests.rs, tests/it/checked_package_v2_model_members.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, v2/model_members/tests.rs, v2/operations.rs tests, and spec/checked_package/matrix/{tests.md,TC-048-checked-package-v2-strict-reader.md}"
review_set: subset
---
# SR-1015: gap analysis of PR 266

## Summary

Ticket: IR-274 (part A). Plan completion: not assessed.

**Method.** I ran seven mutation probes in a detached throwaway worktree
outside the repository (`/home/peter/dev/worktrees/ir266-review`). Each one
ran the full `cargo test --workspace --all-targets`, and the source was
restored afterwards (`git status` clean).

| Mutant | Result |
| --- | --- |
| M1: drop `records = failed_records(..)` in `lower_on_stack` | survived |
| M2: lower under `u64::MAX` in place of the retained `self.bytes` | survived |
| M3: pass `u64::MAX` in place of `limits.bytes` to `validate_nominal_nodes` | survived (equivalent, see below) |
| M4: read the model document under `u64::MAX` | survived (equivalent: the encode under `budget.bytes` returns the same `incomplete`) |
| M5: application keys under `u64::MAX` | survived (equivalent, see below) |
| M6: disable the 2^53 number check | killed, by 2 AC-93 tests |
| M7: `identify_node` ignores its ceiling | killed, by the AC-95 seam test and the AC-90 source scan |

M3 and M5 are equivalent at read time. A nominal or application preimage is
always shorter than the package that embeds it, so `limits.bytes` can never
refuse it once the package itself fit. The source scan is the only possible
oracle for "the reader passes `limits.bytes`", so that is not a finding.

**Per AC:**

- **AC-89.** The ir_id, package_id and length constants equal what I measured
  on origin/main, and the nominal ×4, application (operations.rs) and
  structural (model_members/tests.rs) byte strings are hand-written. This AC
  is partly backed: see FND-001.
- **AC-90.** Backed. The four versions are checked at the limit and one byte
  under it, with `Error::Limit`, `CanonicalBytes` and the bound asserted. The
  no-`u64::MAX` scan works (M7 killed it).
- **AC-91 / AC-92.** Backed over `checked_package/` only, and correctly left
  🚧. The remainder belongs to code changes B and C. The AC-91 scan omits
  `serde_json::to_value`, which the AC names; see FND-004.
- **AC-93.** Backed for the numeric spellings, both digests, document order
  and the package-stream case. The integer-bound clause is not backed: see
  FND-002.
- **AC-94.** Backed. The document is exactly `limits.bytes` long, one byte
  under returns `incomplete`/`bytes` with `consumed` equal to the length and
  no path, and the work limit is located at the row.
- **AC-95.** The seams and the source scan are backed. The outcome of a call
  over the package ceiling and the retained-limit wiring are not: see FND-003.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-89 asks for an expected byte string written out in the test for "a lowered node and a lowered package preimage" too. Neither exists: those two are checked only through recorded digests and length. The matrix row nevertheless lists AC-89 as verified, and the `#[trace]` tag counts it backed under `--strict`. "Every positive fixture" is also only three fixtures. I measured two more as unchanged (model-members `package_over` and complete-v1 `mixed_fixture`), but no test records them. Either add the two byte strings (for example over the ceiling test's empty `ContractPackagePreimage` and its `LoweredNodePreimage`) or keep AC-89 🚧 | tests/it/checked_package_v2_identity_digests.rs:90 |
| FND-002 | medium | AC-93's clause "an integer value type whose upper bound is `9007199254740993` in a document that is otherwise admitted refuses the same way, where today it is admitted (the reader change)" has no test. Nothing in `tests/` or `crates/` puts that bound in a value type's `max` constraint. `semantic_ir_value_type` also accepts a bound written as a decimal string (`Value::String(text) => text.parse()`), which the number scan never sees, so this is the clause where the spelling matters. Add a test for it (number form refused with `document_pointer` at that bound), and state the string-form outcome | tests/it/checked_package_v2_model_members.rs:449 |
| FND-003 | medium | AC-95: "a lowered package whose bytes exceed the ceiling makes every requested record `failed` with no package bytes or id". The tests check `encode_package` and `failed_records` separately, never the `lower_on_stack` wiring. Deleting the record replacement (M1) and lowering under `u64::MAX` instead of the retained limit (M2) both pass the whole suite. The "its sibling records are unchanged" clause of the node case is not asserted either. Add a test that lowers through `lower_on_stack` (the unit seam takes the ceiling) with a ceiling between the largest node preimage and the package length | crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:60 |
| FND-004 | low | AC-91's scan list omits `serde_json::to_value`, which the AC names explicitly ("no `serde_json::to_vec` or `serde_json::to_value` call whose result reaches a digest ..."). The scan also drops everything after the first `\n#[cfg(test)]` in a file, so production code below a test module is unscanned. The structural byte test `tc_048_structural_node_keys_hash_the_expected_canonical_bytes` carries "Tracing: TC-048, FR-038-AC-89" in its doc comment but no `#[trace]` attribute | tests/it/checked_package_v2_identity_digests.rs:262 |

## Verdict

Changes requested. FND-001 to FND-003 are coverage gaps on ACs the matrix
marks verified. Mutations M1 and M2 survive the whole suite. AC-90, AC-93
(numeric spellings) and AC-94 are backed non-vacuously. AC-91 and AC-92 are
honestly scoped to `checked_package/` and correctly left 🚧.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | `tc_048_a_declaration_key_over_the_byte_limit_refuses_at_its_selection_row` is traced to FR-038-AC-89, but it checks the encode-refusal location rule ("`invalid_semantic_graph`" for a refused key). FR-038 states that rule in prose, and it is not AC-89's byte identity. No AC names it, so the tag inflates AC-89's backing with an unrelated test | crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:67 |
| FND-006 | low | The package-over-ceiling test asserts only `consumed > limit`. A mutant that records an invented `consumed = ceiling + 1` instead of the encoder's `required` (M9) passes the whole suite. That satisfies the merged AC-95, but #267's AC-95 requires `consumed` equal to "the `required` of encoding the package under that limit" | crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:297 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | The whole-call node test (`tc_048_a_node_over_the_ceiling_fails_alone_and_leaves_its_siblings_unchanged`) still asserts only `consumed > limit` for the failed node. #267's AC-95 says that record is "`failed` as above", meaning `consumed` equals the `required` of encoding the same preimage under that limit. A mutant that records `ceiling + 1` in `lower_one`'s byte failure (M9b) passes the whole suite. The seam tests check the equality on `identify_node`, but not on the record `lower` returns. Use `assert_eq!(consumed, required_of(&preimage_of_x, ceiling))` | crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:431 |

## Dispositions

Round 1, reviewed at 249acbcd028536586d0e8162b1a15ddfc59e75a9. Mutants were rerun in a detached worktree outside the
repository. M1 (drop the all-records replacement), M2 (lower under `u64::MAX`),
M8 (node byte failure recorded as `work`), M10 (skip a refused owner key) and
M11 (every document number as a double) are killed. M3, M4 and M5 still
survive and are equivalent, as in the review pass. M9 survives (new FND-006).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |
| FND-002 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |
| FND-003 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |
| FND-004 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |

### Round 2

Round 2, reviewed at 31ba6f5f7443e3cb01258f55bb66bd84669b2290 (one commit on ebea678). These are measured in a
detached worktree outside the repository, since removed. All gates exit 0
(`make spec` 2 at the baseline: 1 grammar finding, 23 unbacked, 0
contradicted). The 5-fixture digest dump and all five lowered `.bytes` files
are identical to ebea678.
The same mutant on the package path (M9) survives, but it is equivalent under
the AC-95 oracle. At one byte under the package length, `required` equals the
full length, which is also `limit + 1`, as #267 itself states. The matrix
row's AC-95 sentence is true at this head: equality at one byte under, at one
byte, inside a string value and for the package; `limit + 1` for the in-memory
integer past 2^53, saturating at `u64::MAX`; the object-buffer refusal
untested.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | 31ba6f5f7443e3cb01258f55bb66bd84669b2290 |
| FND-006 | fixed | 31ba6f5f7443e3cb01258f55bb66bd84669b2290 |

### Round 3

Round 3, reviewed at 9698c7f8cd5bfd925c1da117631a29582303d367 (one commit on
ebea678). These are measured in a detached worktree outside the repository,
since removed. All gates exit 0 (`make spec` 2 at the baseline: 1 grammar
finding, 23 unbacked, 0 contradicted). The `reviews/` copies are
byte-identical to the round-2 SR files.

The new assertion builds the oracle preimage from admitted node `x`
(`nodes[1]`, self-typed) with an empty closure, through the same
`From<&CheckedSemanticNodeV2>` conversion `lower_one` uses. It compares the
record with an independent `quire_canonical::to_vec` refusal under the same
ceiling, so it is not tautological.

It still cannot tell a correct `required` from an invented `limit + 1` at this
ceiling. I instrumented the test: ceiling = 2495, required = 2496, full length
= 2563. The refused write is a single byte, so `required` equals `limit + 1`,
and M9b (`lower_one` records `ceiling + 1`) still passes the whole suite.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-007 | still-open | The equality assertion was added, but at the chosen whole-call ceiling (2495) the encoder's `required` is 2496, which is `limit + 1`, so it cannot detect an invented count and M9b survives. Choose the whole-call ceiling where the refused write is two or more bytes. For example, search for the smallest ceiling at or above the siblings-only package length with `required_of(&x_preimage, c) >= c + 2`, as the mid-string seam test does, and add `assert_ne!(*consumed, ceiling + 1)` |

### Round 4

Round 4, reviewed at 2425551e243fe56b92a036d1c40666ecfa7b5ffb (one commit on
ebea678). These are measured in a detached worktree outside the repository,
since removed. All gates exit 0 (`make spec` 2 at the baseline: 1 grammar
finding, 23 unbacked, 0 contradicted). The `reviews/` copies are
byte-identical to the round-3 SR files.

The whole-call node test now picks the smallest ceiling at or above the
siblings-only package length at which `x`'s refused write is two or more
bytes, so `required` is not `limit + 1`. It asserts `consumed == required_of`
and `consumed != ceiling + 1`. The sibling equalities still hold at that
ceiling: records with the request removed, `lowered` ids `[y, z]`, and the
package equal to the siblings-only package. M9b is now killed by this test.
Nothing else changed in the delta.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-007 | fixed | 2425551e243fe56b92a036d1c40666ecfa7b5ffb |
