---
id: SR-2026102754
title: "IR-1105 PR 689 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #689; AGENTS.md, CLAUDE.md, .github/workflows/ci.yml, plan/, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, tests/fixtures/ir987/, tests/it/admitted_result_context.rs, tests/it/checked_package_v2_model_fields.rs, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_model_members/reaches_population.rs"
review_set: subset
---

## Summary

Ticket: IR-1105. Reviewed PR #689's removal of historical prior-wire fixture bytes and reconstruction comparisons, an unused test import, and the scanner's frozen source selection. The live result, refusal, identity, source-map, provenance, and bounded-work assertions remain. The scanner still installs before `make lint`, which still invokes `qsl-string-edge` using the repository's rules.

## Examined scope

- FR-049-AC-5 (examined): The result identifies exact application and conversion occurrences, expression key, source package, lowering profile, and admitted sources; shared-expression items retain distinct outcomes. The edited test retains direct result and source assertions, with separate tagged tests covering shared occurrences.
- FR-038-AC-139 (examined): The model field accessor distinguishes unknown and wrong-kind nodes, ambiguity, absent fields, and intact declarations. The edited test retains valid and unknown-node assertions; other tagged tests cover remaining refusal branches.
- FR-038-AC-144 (examined): Resolution charges bounded work once at admission. The edited model test retains the one-below-limit assertion; other tagged tests cover chains, precedence, and cycle work.
- FR-038-AC-277 (examined): Repeated and cloned reaches-population queries preserve results and charge no work; packages without population retain their outcomes. The edited test retains selected result and bounded-work assertions; a separate tagged repeatability control remains.
- IR-1105 purge objective (examined): Four historical prior-wire byte fixtures and their comparisons, prose records of upstream source revisions, and the CI scanner's frozen source selector are removed. Product-defined canonical content identities remain in live assertions.

## Verdict

PASS. The deleted fixture comparisons had no contract-defined historical-byte oracle; their subjects are removed. The test import was unused. The CI scanner install, lint gate, tests, formatting, and supply-chain steps remain, while source acquisition follows the scanner repository's default branch. The CI YAML parsed, the changed plan passed Quire validation, and the diff passed whitespace validation. Cargo tests and the aggregate gate were not run during this review under the shared build-lane instruction; the code lead owns final gate evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Dispositions

No findings required a disposition. The follow-up edits to the CI install, unused import, and plan provenance section were reviewed at their final PR state and introduced no finding.
