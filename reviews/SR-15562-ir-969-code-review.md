---
id: SR-15562
title: "Code and Rust review — IR-969 typed V2 graph joins"
type: SpecReview
analysis: code-review
scope: "quire-contract-ir PR #515; checked_package/v2/{mod,structural,owner,extension,contract_package,typed_terms}.rs and changed tests"
review_set: subset
---

## Summary

Reviewed the diff for decoder placement, V1 structural reuse, and changed typed graph joins. The new decode follows each node's flat grammar scan and preserves the abstraction relation's later semantic refusal; the V1 path supplies the same typed terms to its shared structural validator. The changed structural module now duplicates shape rules in typed and raw readers; later V2 validators still call the raw copies.

## Verdict

**FAIL** — the new typed validators duplicate owned structural rules while the raw validators remain active. The focused review did not run an aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Typed anchor, state-clause and union shape readers duplicate active raw readers, so one rule now has two implementations | `crates/quire-contract-model/src/checked_package/v2/structural.rs:657`, `:742`, `:825`, `:866`; raw callers in `state.rs:66`, `:326` and `operations.rs:1765`, `:1793` |

## Rust review

Inspected ownership and borrowing of typed terms, fallible decoding and refusal mapping, bounded term depth after flat validation, iterative application-dependency traversal, integer conversion and panic surfaces, source duplication, and repository conventions. No new `unsafe`, unchecked cast, panic on untrusted input, or unbounded recursive preadmission traversal was identified in the changed lines. The structural-shape duplication is the finding above. `git diff --check..` passed. Final aggregate gates remain with the author after custody commit.

## Finding detail

`typed_anchor_body`, `typed_clause_body`, `typed_union_type_body`, and `typed_union_value_body` each reimplement the corresponding raw reader in the same module. The raw readers are still called by the V2 `state` and `operations` semantic checks after `validate_structural_nodes` has already checked the typed form. This introduces two implementations of each rule; a future shape fix can change one refusal path without the other. Route the later semantic consumers through the validated typed term or consolidate the rule at one owner. The untrusted flat-wire/first-refusal intake remains a separate parser and does not require removing it.

## Head confirmation

The.. delta only changes the structural unit test to decode three JSON fixtures into `CheckedTerm` before calling `application_dependencies`. It leaves the production paths and the code finding unchanged. `git diff --check` on the delta passed.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | Typed parameter-binding parser duplicates the raw parser still used by the admitted-package accessor | `crates/quire-contract-model/src/checked_package/v2/structural.rs:519`, `:547`; `crates/quire-contract-model/src/checked_package/v2/parameter_declaration.rs:55` |

The typed `parameter_defect` calls `typed_parameter_binding` during V2 structural validation, while `CheckedPackageV2::parameter_declaration` calls the raw `parameter_binding` on the same admitted node. Both functions enforce the identical name/level closed shape. This is another duplicate rule introduced by the typed migration. Route the public accessor through the retained typed term and remove the raw duplicate, while preserving its typed errors for test-only mutated packages.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |, the raw anchor, state-clause, union-type and union-value readers are removed. State, union operations/leaf walks, and comparison lookup now use retained typed terms. |
| FND-002 | still-open | The raw `parameter_binding` remains in use by `CheckedPackageV2::parameter_declaration`, beside the new typed structural reader. |

## Narrow review evidence

Reviewed `git diff..` (five files); diff check passed. The changed `state` and `operations` consumers receive the already decoded `terms` vector; the admitted package accessor reads its retained vector. No aggregate gate was run in this reviewer lane.

## Dispositions — round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed |, `parameter_declaration` reads `self.terms` and calls the single `typed_parameter_binding` owner. The raw `parameter_binding` and its raw literal helper are removed. The retained-term mutation test and synchronized malformed-body controls exercise the new accessor path. |

## Current verdict

**PASS** at exact — both findings are fixed; no new finding in the two-file fix diff. `git diff --check..` passed. The author owns the final-head aggregate gate.
