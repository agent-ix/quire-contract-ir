---
id: SR-611002
title: "IR-874 PR 611 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; crates/quire-contract-model/src/checked_package/v2/model_fields.rs; crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs; crates/quire-contract-model/src/checked_package/v2/model_members.rs; crates/quire-contract-model/src/checked_package/v2/refinement.rs; tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Ticket: IR-874. Planless, diff-scoped trace and reverse-gap audit of PR #611 at 76228ae301cca6da01f8c625a5b4699b96d3a7ac. Plan completion was not assessed. FR-038-AC-325 has six bound tests, but none tests the redefinition diamond with the redefining branch second.

## Examined units

- **FR-038-AC-325** (`examined`): "A record activation binder follows effective fields in source supertype order and puts a redefining field where the hidden field stood; reversing source supertype order changes binder order."
- **TC-048** (`examined`): "Record activation binders test authored inheritance, reversed supertypes, diamond deduplication and redefinition through L."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The six FR-038-AC-325 tagged tests omit reversed diamond supertypes with a redefinition on the second branch. The current redefinition loop always uses [Left, Right], leaving the wrong binder order undetected. | tests/it/checked_package_v2_refinement.rs:679 |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` at PR #611 head, 749 criteria: 629 tagged, 100 untagged, 19 method-without-symbol, 1 tagged-by-ignored-test. FR-038-AC-325 is tagged by six tests.
- Plan completion: not assessed
- Reverse gap: changed field-order and activation code is owned by FR-038-AC-325 and TC-048; no new unowned behavior or stub found in the diff.
- Optional semantic review: skipped as a separate repository-wide audit; changed intent, tests and code were inspected in SR-611001.

## Verdict

CONDITIONAL. FND-001 is a concrete missed adverse ordering case. Repository-wide uncovered criteria predate this PR and are not a claim that the changed AC is fully discharged.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The same tagged public-reader test now reverses the diamond's supertype list for both redefinition states and requires the replacement at Top/t's earlier position; the reverse-diamond case passed 1/1 at 065dc191a7dafdf265103079e2e82b5737d8e810. The model-field unit test asserts the corresponding identity order. |

## Current verdict

PASS for the changed AC-325 trace and reverse-gap claims at PR #611 head 065dc191a7dafdf265103079e2e82b5737d8e810. The repository-wide strict-coverage backlog remains separate.
