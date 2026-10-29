---
id: SR-600005
title: "IR-874 PR 586 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #586; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, crates/quire-contract-model/src/checked_package/v2/refinement.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---

## Summary

Ticket: IR-874. Compared the frozen PR #586 early presence check with FR-038-AC-325, the computed matrix, and its direct test. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — the new unit test bypasses public checked-package reading and has no criterion trace tag, so the intended refusal ordering and public path have no slice-specific test evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The only new test calls `validate_catalog` directly and lacks a trace tag; add a public-reader case that proves a concrete protocol with declaration-backed rows and no abstract `protocol` field refuses at `/body/class_catalog`, while a model-only extra catalog refuses there and the positive case proceeds. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:2201 |

## Scope

- **FR-038-AC-325 (examined)**: The protocol step compares every present refinement `class_catalog` with complete reconstructed classes and preserves the specified row, protocol and catalog-join refusal order. The complete graph join and memory semantics remain Planned.
- **FR-038-AC-9 (context_only)**: The exact measured work boundary is part of the public checked-package reader contract.
- **QSpec FR-375 checked class catalog (context_only)**: Presence is required when either subject contributes a protocol or memory class, and extra catalog on a model-only refinement refuses at the member.

## Coverage

- Reconciliation: `quoin matrix --repo. --json` at frozen PR head; FR-038-AC-325 is `untagged` with zero binders, while FR-038-AC-9 is tagged by one public-reader test. The new `concrete_protocol_requires_catalog_without_protocol_target_or_memory_row` test has no `Trace:` or `#[trace]` annotation.
- The direct unit fixture calls `validate_catalog` without going through key, header, graph, protocol-body and public-read stages. It proves the helper returns a pointer, but cannot prove that an otherwise admitted checked package reaches that helper or preserves the specified earlier refusal.
- Plan completion: not assessed.
- Optional repository-wide semantic review was not requested; intent, test, and changed code were compared only for this PR diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At fix, `refinement_catalog_presence_precedes_declaration_backed_step_joins` calls the public reader and carries `Trace: FR-038-AC-325`. It asserts the absent concrete-protocol catalog pointer and code before a declaration-backed row join, demonstrates a present catalog reaches row joining, and checks model-only admission and extra-catalog refusal. The computed matrix now binds AC-325 to that test. |
| FND-001 | still-open | Hosted Rust Checks on ran the new public-reader test and failed: the first case refused at `/semantic_graph/nodes/26/semantic_type` rather than the expected refinement `/semantic_graph/nodes/27/body/class_catalog`. The fixture has an earlier defect, so it does not yet verify the catalog precedence. |

## Disposition Verdict

**PASS on reviewed fixes** — the slice-specific test gap is closed. This does not qualify all FR-038-AC-325 class reconstruction or pending memory behavior.

## Disposition Round 2

**CONDITIONAL** — the exact-head hosted test falsified the earlier code-inspection disposition for FND-001. The latest outcome is still-open until an admitted fixture reaches the intended catalog check.

## Disposition Round 3

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The replacement fixture uses `OrderFlow::Closed` from `v2_all_families`, but that clause's `over` parameter is Boolean-typed and does not denote the selected `test/refinement/Machine` model. Its catalog refusal therefore does not verify FR-375's same-model presence rule. The new test's `Trace: FR-038-AC-325` still claims more than it proves. |

**CONDITIONAL** — the latest public-reader test is tagged but lacks an admitted same-model protocol subject. The full class graph join and memory behavior remain Planned.

## Disposition Round 4

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At fix, two `#[trace("TC-048")]` public-reader tests use an admitted protocol clause with a typed owner edge to the selected model. They assert missing-catalog refusal before a declaration-backed row join, same-model presence without a step row, foreign-clause admission without a catalog, extra-catalog refusal, and foreign-row binding refusal at `steps/0/source_node`. Focused refinement tests report 12 passed. This is slice evidence only; neither FR-038-AC-325 nor FR-064-AC-5 is claimed complete. |

**CONDITIONAL** — the slice-specific public-reader gap is closed. The full class catalog join, complete inventory and memory behavior remain unclaimed under FR-038-AC-325 and FR-064-AC-5; final gates belong to the code lead.
