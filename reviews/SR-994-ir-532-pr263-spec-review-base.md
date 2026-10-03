---
id: SR-994
title: "spec review of PR 263 (FR-038 operation-law ACs AC-81 to AC-88)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@376269e0d67ee1451ee9629dbe897c06a8a730de; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (sections Application node keys, Operation identity, laws, mode, member and operands; FR-038-AC-81 to AC-88), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md (FR-038 and TC-048 rows); checked against crates/quire-contract-model/src/checked_package/v2/operations.rs at the same sha, QSpec FR-322 and the checked-package v2 schema (quire-specification origin/main), and QSL qsl-semantics node_key tests and FR-092 (quire-spec-language origin/main)"
review_set: base
---
# SR-994: spec review of PR 263

## Summary

Ticket: IR-532. Base and correctness review of a spec-only PR. It adds two FR-038
prose sections, eight planned ACs (AC-81 to AC-88), a TC-048 procedure section and
the matching tests.md rows. The aim is to give owning ACs to the untraced
operation-law unit tests in `operations.rs`.

Enumeration, measured at origin/main (9631d7e; main has since moved to 7d7716d,
a reviews-only commit, and the PR merges clean onto it):

- `operations.rs` has 61 `#[test]`. 37 carry `#[trace]` (AC-43, AC-44, AC-70,
  AC-71, AC-72). 24 carry none. Of those 24, 4 have `tc_048_` names: lines 3005,
  3050, 5872 and 5900. The other 20 are untagged. The PR's table covers all 24
  with the right line and assertion: 21 map to AC-81 to AC-88, 2 (3363, 3398)
  to AC-56 and 1 (3637) to AC-44. Each test was read and each code and pointer
  checked against the code.
- The ticket's "about 28" is 24 in this file. Two more untagged operation-adjacent
  tests sit next door: `tc_048_the_catalog_vocabularies_are_the_decoded_enums`
  (operation_catalog.rs:196, belongs with AC-65) and
  `tc_048_application_join_collects_targets_and_member_declarations_only`
  (structural.rs:650, the application dependency join). Both are outside
  operation-law scope.

Code facts behind the findings: `validate_operations` and
`validate_application_keys` visit only nodes whose body root is an application
(operations.rs:133-137, 352-360). `OperationWire.mode` and `.member` are `Option`
fields with no `#[serde(default)]` override, so an omitted member decodes as
`None`, and `member_or_operation` then points at `operation` itself
(operations.rs:262-268, 512-518). `check_mode_type` checks every fixed operand,
not only the first (1644-1664). The catalog's `type_pinned_modes` is
`rounding: [decimal, float32, float64, quantity]` and `text_profile: [text]`.

Correct as stated (no finding): AC-81, AC-82 and AC-87 codes and pointers. AC-84's
`operation.member.name` and `operation.member.declaration` pointers. AC-85's arity,
family, clause and `same_type` refusal pointers. The catalog facts the ACs quote
(integer.div law role, decimal.add rounding mode, quantity.convert `type_argument`,
record.project `field`, structural.eq `same_type` over [0, 1], state.clause result
`clause`) all match the locked quire-verification-contracts catalog.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-86 and the "Operand families by form" bullet say `temporal`/`formula` has no operand family and is not type-shaped. Merged FR-038 prose ("Catalog words", 817-821) and FR-038-AC-68 require a `reference` to a `temporal`/`formula` node to resolve to family `temporal`, which needs the node to be type-shaped. Open PR #253 implements exactly that (`operand_family` and `is_type_shaped` gain `Temporal(Formula)`). Merged first, AC-86 contradicts AC-68 and #253's code | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:961-974, 1408 |
| FND-002 | high | AC-88 says the preimage "serializes to the one canonical text QSL's `application_key` pins for it". That is false. QSL's `preimage_bytes_are_pinned` (qsl-semantics/src/check/node_key/tests.rs:230) pins a node outside any group (`recursion: null`, both arguments plain references). The IR test's expected text is its own literal (operations.rs:2999-3044). QSL does pin in-group application vectors in FR-092 (lines 788 and 796), but for other nodes | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1410 |
| FND-003 | medium | The new "Operation identity, laws, mode, member and operands" section speaks of every `application` term. The reader checks operations only for a body-root application (merged prose 772-773; operations.rs:352-360). QSpec FR-322 also checks "each argument application under the same order". "Application node keys" likewise re-keys only body-root applications, while QSpec FR-322 keys every node whose body contains one. Neither section names the narrower scope or the deviation | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:886-910 |
| FND-004 | medium | The section says it does not fix the order between its checks. QSpec FR-322 does fix it: identity, class, law-missing, law-mismatch, law-unselected, mode-mismatch, member-mismatch, then argument applications, then `operation-mode-type-mismatch`, then `operator-ineligible`. The reader follows the first seven and reverses the last pair. Calling the order unspecified hides a known conformance deviation | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:910-915 |
| FND-005 | medium | The prose says `operation` is read "as its own closed shape" and that an application that "carries no mode" or "no member" refuses at `operation.mode` or `operation.member`. The QSpec v2 schema requires both members. The reader admits an `operation` that omits them, and for an omitted one it points at `operation`, not `operation.mode`. The tests and AC-83/AC-84 only hold for `mode: null` and `member: null` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:906-908, 931-939, 1405-1406 |
| FND-006 | medium | The Mode bullet limits the type pin to "the first operand's type", "a `bounded_domain` over a `decimal` that binds `rounding`". The reader checks every fixed operand whose family fits. It pins `rounding` for decimal, float32, float64 and quantity, and `text_profile` for text, per the catalog's `type_pinned_modes`. The bullet reads as the whole rule, so it understates what refuses | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:931-937 |
| FND-007 | medium | "An operand that resolves to no type leaves the constraint undecided and is admitted" (prose and AC-85's last clause) turns a silent skip into a normative admission. The AC's example, a `literal` with no `type`, is a shape the body grammar refuses (common.rs:693-699 requires `term`, `type`, `value_kind` and `value`), so the clause pins behaviour no package can reach | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:959-960, 1407 |
| FND-008 | low | The Arity bullet says only "more arguments than its entry has fixed operands". The reader refuses any count other than the fixed operands, or fewer than them when a rest operand is admitted (operations.rs:1038-1044). The merged temporal.clause prose already says "any other argument count" | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:945-947 |

## Verdict

Changes needed. The enumeration is complete and correct, and most codes and
pointers match the reader. Two ACs are wrong as written: AC-86 contradicts merged
AC-68, and AC-88 claims a QSL vector that does not exist. Suggested fixes:

- FND-001: add `temporal`/`formula` → `temporal` and its type-shaped flag to AC-86
  and the bullet, marked as arriving with IR-503 (#253). Or keep AC-86's table and
  land it after #253.
- FND-002: drop the QSL provenance clause and put the expected text in TC-048. A
  better option is to make the test reproduce QSL FR-092's pinned in-group
  application vector (FR-092 line 788 or 796), which is a real cross-implementation
  check.
- FND-003 to FND-007: scope the sections to body-root applications and name the
  deviations from QSpec. Pin QSpec's order and record the one reversed pair as a
  deviation with a ticket. Say `mode`/`member` `null`, and ticket the omission
  leniency as a pre-existing reader defect. Generalise the mode-pin rule. Drop the
  untyped-literal admission.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@9d4379bea459d7a30bc277ecbdaa72bae08d9bac.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | low | The omission leniency is called tracked as an existing defect, but no IR ticket exists for it (Linear search, 2026-10-03), and the other three stated deviations (reversed mode-type/operator-ineligible pair, body-root-only operation check and keying, binding-only mode pin) name no ticket either. File the tickets and cite them, or drop the word tracked. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:937-941 |
| FND-010 | low | The Mode bullet attributes the unbound exact rounding pin to QSpec FR-322. FR-322 (c76c6ae, lines 160-162) says only that the value equals the single value pinned by the operand and result types; the exact default for an omitted rounding spelling is QSpec AD-005 line 65 and FR-140 line 40. Cite those for the exact part. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:963-966 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9d4379b |
| FND-002 | fixed | 9d4379b |
| FND-003 | fixed | 9d4379b |
| FND-004 | fixed | 9d4379b |
| FND-005 | fixed | 9d4379b |
| FND-006 | fixed | 9d4379b |
| FND-007 | fixed | 9d4379b |
| FND-008 | fixed | 9d4379b |
| FND-009 | fixed | cfb815d |
| FND-010 | rejected | Not a defect: QSpec FR-322 at c76c6ae line 684 states 'a type without a `rounding` binding pins `exact`' (and line 682 that the result type pins too), so the original attribution to FR-322 was right; the AD-005/FR-140 citation the author added for the strict default is accurate and harmless. |
