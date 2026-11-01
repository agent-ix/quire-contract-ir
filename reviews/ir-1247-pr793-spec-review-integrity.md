---
id: SR-202611010112
title: "IR-1247 PR 793 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #793; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Checked the new dimension carrier, positive and adverse criteria, and planned public API for consistency and first-refusal order against QSpec. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**CONDITIONAL** — the carrier and direct-dependency boundary are sound, but AC-389 incorrectly groups a wrong-kind operand case with pair-rule outcomes after operand resolution.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-389 groups XP-05 with pair outcomes after operand resolution even though XP-05 is a wrong-kind operand that QSpec refuses at the operand before pair validation; separate XP-05 from the pair cases to preserve its exact locus and order. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4591 |

## Coverage

- FR-038 dimension operand paragraph, role: examined. It distinguishes the local and package carriers, direct selection, and operand-first joins.
- FR-038-AC-387, role: examined. XP-01/10 demonstrate positive local and package admission without minted graph nodes.
- FR-038-AC-388, role: examined. Package selection and name refusals precede pair checks.
- FR-038-AC-389, role: examined. Its wrong-kind sentence and broad XP-02..09 pair sentence disagree for XP-05.
- FR-019 planned API paragraph, inventory row, and AC-17, role: examined. The public type inventory stays conditional on code landing.
- QSpec FR-322-AC-299 and dimension-equivalence body rule, role: context_only. They state operand checks before pair rules and exact operand loci.

## Evidence

QSpec dimension-equivalence-vectors XP-05 names acme/rpc::s, a unit rather than a dimension. QSpec FR-322 requires a package operand resolving to a non-dimension node to refuse `invalid_semantic_graph` at that operand; the pair rules run after operand checks. AC-389's first sentence reflects this, but its XP-02 through XP-09 grouping includes XP-05 as a pair outcome.
