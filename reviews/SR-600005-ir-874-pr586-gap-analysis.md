---
id: SR-600005
title: "IR-874 PR 586 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #586 at 4a322bb13c354c04ef2c79b35c5067a693a0b310; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, crates/quire-contract-model/src/checked_package/v2/refinement.rs, tests/it/checked_package_v2_refinement.rs"
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

- Reconciliation: `quoin matrix --repo . --json` at frozen PR head; FR-038-AC-325 is `untagged` with zero binders, while FR-038-AC-9 is tagged by one public-reader test. The new `concrete_protocol_requires_catalog_without_protocol_target_or_memory_row` test has no `Trace:` or `#[trace]` annotation.
- The direct unit fixture calls `validate_catalog` without going through key, header, graph, protocol-body and public-read stages. It proves the helper returns a pointer, but cannot prove that an otherwise admitted checked package reaches that helper or preserves the specified earlier refusal.
- Plan completion: not assessed.
- Optional repository-wide semantic review was not requested; intent, test, and changed code were compared only for this PR diff.
