---
id: SR-2687
title: "IR-680 gap-analysis of the retained expected node key"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #318; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-183, FR-038-AC-184, FR-038-AC-185), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md, crates/quire-contract-model/src/checked_package/{common.rs, shared.rs, v2/derived_keys.rs, v2/operations.rs}, tests/it/checked_package_v2_structural_keys.rs, tests/it/checked_package_v2_model_members.rs, tests/it/support/checked_package.rs; ticket IR-680"
review_set: subset
---

## Summary

Ticket: IR-680, CODE stage, PR #318. Planless gap analysis. Plan completion: not assessed. The
reviewed commit identity is recorded only in the Linear marker. Model `claude-opus-5-5`; run
`324c4e6a-ee78-4f27-9020-194cd2cd917e`. The computed matrix is from
`quire coverage --scope . --strict` on the PR head (quire 0.36.1, engine 0.50.1): 25 unbacked
rows and 0 contradicted statuses, the same as the author's baseline. FR-038-AC-183, AC-184 and
AC-185 each have tagged tests.

## Verdict

**No blocking gap.** Each criterion has a tagged test and code that owns it:

- FR-038-AC-183: tagged by `tc_048_derived_expected_keys_rekey_unreferenced_nodes_in_both_branches`
  (the closed-shape `integer_range` `base.zero` and the `UngroupedPreimage` `value/parameter`
  node, each rekeyed and admitted in full, plus a second bound that changes the digest),
  `tc_048_nonderivable_stale_keys_retain_absence` and
  `tc_048_each_self_typed_shape_needs_its_id_and_type_changed_together`, which covers all eight
  self-typed forms. Owner: `validate_derived_keys`.
- FR-038-AC-184: tagged by `tc_048_application_expected_key_survives_reader_and_dispatch`
  (an unreferenced application with full admission after rekeying, a second preimage change,
  and the dispatch forwarder) and by the unit test
  `tc_048_validate_application_keys_refuses_a_stale_node_key`. Owner:
  `validate_application_keys`, which reuses its single `computed` digest.
- FR-038-AC-185: covered by the `None` cases above, the model-declaration case
  `tc_048_selected_model_declaration_shape_has_no_derived_expected_key`, the migrated
  `ExpectedRefusal` literals (canonical encoding and pre-key-stage refusals compare `Absent`),
  the `Eq` and clone assertions, and the private field with its `pub(crate)` constructors.

No production code was added without an owning requirement. No stub, tautology or
reader-versus-reader oracle stands in for an assertion. The positive controls are full-package
admissions. The domain-assertion weakness is recorded once, in SR-2686 FND-001, and is not
repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-183/184/185 rows, the TC-048 procedure and the TC-226 section still read "PLANNED / UNRUN until IR-680 code lands", although this PR implements and tags them; after merge the spec's status prose contradicts the code | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3591-3593; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1206; spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md:97 |

### FND-001 detail

The earlier IR-627 code change (#308) also left TC-226 status prose unflipped, so this follows a
repository precedent rather than breaking a rule. Strict coverage does not count the prose
status, so no gate catches it. A reader of FR-038 after merge would conclude IR-680 has not
landed. Flip the three rows and the two procedure headers in this PR or in a tracked status
follow-up.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The status fix for FND-001 committed an unresolved rebase conflict marker, `>>>>>>> <commit id> (IR-680: close review findings on typed key evidence)`, at the end of the FR-038 row of the checked-package matrix. This adds a commit identity to the public repository against its CLAUDE.md/AGENTS.md rule, and adds a stray cell after the row's closing pipe. `quire validate` and strict coverage both pass, so no gate catches it | spec/checked_package/matrix/tests.md:16 |

## Dispositions

Disposition pass 1. Strict coverage on the head shows 325/360 rows backed against main's 322/360, the +3 being FR-038-AC-183/184/185. There are 31 unbacked rows on both, and 0 contradicted. Each changed spec document validates on its own: FR-038, TC-048, TC-226 and tests.md. IR-651's FR-038-AC-177 through AC-182 rows are unchanged.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit "IR-680: close review findings on typed key evidence" (short id in the Linear marker). The FR-038-AC-183/184/185 rows now read "Implemented and verified by TC-048 [and TC-226] (IR-680 code)". The TC-048 procedure and both TC-226 passages name the tests that actually run, and the tests.md FR-038 row marks AC-183 through AC-185 implemented. All of this matches the tagged tests. The fix introduced FND-002. |
