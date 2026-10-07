---
id: SR-2961
title: "Gap analysis of the IR-663 private intake retention code"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir branch code/ir663-private-intake-retention, head commit 'Charge missing-origin group candidates with an independent visit oracle'; FR-038-AC-174, FR-038-AC-175, FR-038-AC-186 through FR-038-AC-196; TC-048 private intake refusal-origin checks"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-663. A planless gap analysis over the computed Test Matrix, the tagged
tests, and the production code in the diff. `quire matrix --format tsv`
(quire 0.36.1, engine 0.50.1) produces 347 criterion rows at both main and
head, with the same identities. Exactly ten rows change status:
FR-038-AC-174, AC-175 and AC-186 through AC-192, plus AC-196, go from
`untagged` to `tagged`. Every binder is in
`model_members/tests/intake_retention.rs`. No other row or binder changes.
Totals move from 301 to 311 tagged and from 40 to 30 untagged. The counts
for method-without-symbol (4) and tagged-by-ignored-test (2) are unchanged.
AC-193 and AC-194 stay `method-without-symbol` (Inspection). AC-195 stays
`untagged`; it is a compiler lane, with no test symbol possible. `quire
coverage --strict` exits 1 at both main and head on pre-existing rows. The
unbacked count falls from 36 to 26, with 0 contradicted statuses, so strict
coverage is no worse.

Code with no owning requirement: none. Every production hunk implements the
FR-038 retention contract or is its type plumbing (`SelectionFailure`
replaces `ModelFailure` in `read_semantic_ir`). Stubs and tautologies: none
found. The binders assert typed values, not `is_err`.

## AC to test map (examined)

- AC-174: tagged by the source-values test and the nested-context test.
  Exact identity is checked, including the refused `bad-id` spelling;
  Null and number identities give None.
- AC-175: tagged by the source-values test and the pre-declaration test.
  The pre-declaration test covers missing bytes, digest mismatch and wrong
  identity, with the original member and None for both fields.
- AC-186: partially backed. See FND-001 in SR-2960: 2 of the 4
  end-coordinate presence combinations are asserted.
- AC-187: backed. Exact version text and ordered, repeated inputs are
  checked.
- AC-188: owner substitution is backed. Stale-context leaks are unbacked
  (SR-2960 FND-001).
- AC-189: backed for Null, empty both-branch, unknown member and empty
  inputs. Salvage cases are weak (SR-2960 FND-004).
- AC-190: backed for equal, distinct-value, distinct-presence, missing and
  malformed origins, in both permutations, for conflicting-binding and
  bad-kind groups. Also backed for a bad common object-id with equal and
  missing origins, and for the candidate visit charge.
- AC-191: backed. Meaning precedes Reference within a node. The exact-work
  refusal equals the unlimited refusal. InexactNumber precedes the
  declaration checks under both digests.
- AC-192: the group visit charge and the exact limit payload are backed.
  Located-retention charges are unbacked (SR-2960 FND-003).
- AC-193: Inspection holds. The arm projects code, path and cause only, and
  `CheckedPackageRefusal` is unchanged.
- AC-194: Inspection holds. Typed `PartialEq` decides group retention, the
  values are owned through `admit_selection` after `drop_value`, and the
  mapping arm drops them explicitly.
- AC-195: compiler lane. Not run by this reviewer.
- AC-196: backed. Under one owner and across two owners, the later
  relationship's origin is retained; when that relationship has no origin,
  the result is None with malformed-declaration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Tests are tagged but have not run at the frozen head. The only binder for AC-190 (the group test), and binders shared by AC-189, AC-191 and AC-192 (the group test, and the metadata and pre-declaration tests), changed in the last two commits. The last commit also changed production code: the group visit charge. The author records that the final-head charge, its oracle and two assertions are unrun, and that the full `make ci` has not run. A `tagged` status here proves only that the binding exists. No executed evidence supports these rows at this head. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1216; crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs:193-302,344-396,399-469 |
| FND-002 | low | TC-048 step 1 asks for a declaration reference failure with a valid generated origin. Generated retention is exercised only through a malformed-identity refusal. Step 2's spelling variation (identity, path, version) is exercised only for version and input order. The criteria remain covered, but the procedure text is not followed literally. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1263-1284; crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs:89-104 |
| FND-003 | low | A defensive arm still drops AC-196 metadata. The cross-owner `relationships.insert(..).is_some()` arm returns a bare `ModelRefusal` through `From<ModelRefusal>`, which retains None. Today it is unreachable: the known/local duplicate defect fires first, with the later relationship as context. The invariant is neither stated nor tested, so a future reordering would silently break AC-196. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:2030-2037,2480-2489 |

## Coverage

- Criteria in scope: 13. Tagged: 10. Method-without-symbol (Inspection): 2.
  Untagged by design (compiler lane): 1.
- Matrix delta from main to head: 0 identities added or removed, 10
  untagged-to-tagged changes, no other status or binder change.
- Strict coverage: exit 1 at both main and head (pre-existing). Unbacked
  rows fall from 36 to 26, with 0 contradicted.
- Semantic review: done for the in-scope criteria, by reading the code and
  tests (see SR-2960).
- Plan completion: not assessed

## Verdict

Not merge-ready on evidence. The matrix improves and nothing regresses, but
the tagged rows for AC-189 to AC-192 rest on tests that have not run at
this head. The partial clause coverage recorded in SR-2960 also applies to
AC-186, AC-188 and AC-192.
