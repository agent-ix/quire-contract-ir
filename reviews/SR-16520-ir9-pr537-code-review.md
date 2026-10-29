---
id: SR-16520
title: "IR-9 PR 537 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #537 @; crates/quire-contract-model/src/checked_package/v2/extension.rs"
review_set: subset
---
# SR-16520: IR-9 PR 537 code and Rust review

## Summary

Independent diff-scoped code and Rust review of the PR diff. The changed production path uses the retained node-kind vector for scalar, integer-range and selected-record joins and for refusal details. The extension body, expected type grammar, open extension-name comparison and admission stages are unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope examined

- `type_matches` checks the target's retained kind before joining. The indexed type node is read with the corresponding retained kind; scalar aliases, bounded integer terms and selected record conformance preserve their previous branches. Unknown extension names remain data in the retained `CheckedTerm`.
- `actual_type` uses the same retained type-node kind for native diagnostic names. The deliberately damaged private metadata controls distinguish this from rereading `node_tag` and `semantic_form`. The fallback spelling for other type kinds is unchanged.
- Five new `#[trace("TC-459")]` controls call the real extension `validate` function, including selected owner conformance, type mismatch details, work charge, missing-selection precedence and open extension names. The private damaged states do not claim reader admission; existing TC-459 public reader controls remain in scope as context.
- No public API, wire schema, lock, rule, attribute, unsafe code or gate configuration changes. `git diff --check` passed. The final aggregate gate belongs to the reviewed final PR head.

## Verdict

PASS for the exact PR diff. This review does not claim FR-056-AC-8 source cleanliness.
