---
id: SR-625
title: "gap analysis of PR 229 (reaches_field reference_edge constraint)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@65d29c5a3bf803c340987fb21eaadb6bd173013b; spec/contract/FR-040-admit-frame-entries-and-state-clauses.md, spec/contract/FR-038-consume-checked-package-v2.md, tests/it/checked_package_v2_model_members.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---
# SR-625: gap analysis of PR 229

## Summary

Ticket: IR-370. Plan completion: not assessed.

This analysis traces the new `reaches_field` behaviour through three layers:
the requirement in this repo, the upstream QSpec acceptance criterion
(FR-322-AC-38, which TC-281 RE-01..RE-15 make concrete), and the tagged
tests.

The code decides every RE vector the way TC-281 records it. The one
exception is direction, which SR-624 FND-001 covers. The tests exercise
RE-01..RE-11 and RE-13. RE-12, RE-14 and RE-15 are not exercised.

`quire coverage --strict` reports the same 22 unbacked rows on this head as
on `origin/main` (3e7935f). The PR changes nothing in that set.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new behaviour has no acceptance criterion. FR-040's statement was rewritten to admit `reaches_field`, but no FR-040 or FR-038 AC names `reaches_field` or `reference_edge`. The two new tests are traced to FR-038-AC-29, which covers `dispatch_call`, inherited fields and reference-eq conformance, not reaches. The coverage gate therefore counts the behaviour as backed through an unrelated AC. | tests/it/checked_package_v2_model_members.rs:515-516 |
| FND-002 | medium | Several refusals that FR-322-AC-38 lists are untested here: an ambiguous field name (`ambiguous_declaration`/`ambiguous-name`, RE-14), a declaring node under an unselected version (`missing_declaration`/`missing-selection`, RE-12), and an operand of family `object` (RE-15). The two-level subtype target (`SubSub`) is also untested. None of these cases goes red if the step-2 or step-3 refusal pass-through in `check_reference_edge` is broken. | tests/it/checked_package_v2_model_members.rs:541-568 |

## Finding Detail

- FND-001: Add a FR-040 AC, for example FR-040-AC-13, that restates the
  FR-322-AC-38 admissions and refusals this reader decides, including the
  supertype-target refusal from SR-624 FND-001. Retag the two
  `tc_048_reaches_field_*` tests to it. The TC-048/TC-056 matrix row may need
  the new AC id as well. FR-040's ACs are traced to TC-056, so choose the TC
  row that matches.
- FND-002: Extend `edge_document` so that `Both` has two supertypes that each
  expose `code`, and add an unselected-declaration case and an object-family
  operand case, following RE-12, RE-14 and RE-15.

## Verdict

Not mergeable as is. FND-001 and FND-002 are both small test and spec
additions to make in this PR.
