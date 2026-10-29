---
id: SR-16304
title: "IR-987 PR 529 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; 17 changed Rust files under crates/quire-contract-model/src and tests/it; IR-987 AC-1..4; FR-019, FR-023, FR-038, FR-049, FR-053"
review_set: subset
---
# SR-16304: IR-987 PR 529 code and Rust review

## Summary

Independent diff-scoped code and Rust review of PR #529 at its frozen head. All 17 changed paths were examined. The five residual owner paths are model field lookup, scalar application/child/type lookup, result-context source lookup, selected-population source lookup, and projection declaration lookup. FR-019's public-item inventory lists the enclosing types and explicitly treats inherent methods as part of them; the two new public methods do not require separate table rows. No spec, CI, dependency, wire or charge-rule file changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- `model_object_fields` uses the retained admitted node index; unknown-node and ambiguous-field branches stay distinct.
- `scalar_application_operands` and its child/type resolution use admitted node positions. The typed error map and argument order remain unchanged; the post-admission dangling-child test still reaches `MissingChild`.
- `ContractPackageV1::source_occurrences` indexes the first source-map row per node with `or_insert`, preserving the former `.find` first-match behavior and the exact borrowed occurrence slice. Its six internal callers retain the same error paths. The index is built on lower and strict read, outside canonical wire and validation charging.
- Selected population indexes `(node, Type, ordinal)` with first position, retaining exact selection and source provenance; missing and nonpopulation outcomes remain distinct.
- `Environment::value_declaration(&str)` reads the existing binding-admission index. The public projection consumer checks found and absent names. `SymbolName: Borrow<str>` enables a borrowed map lookup; no duplicate map is introduced.
- No copied upstream artifact, untracked public crate-root item, unsafe block or gate narrowing was introduced.

## Gate Evidence

Review was read-only. The author reported focused checks on IR-987; this review does not independently certify those runs. The author owns the one final-head aggregate gate after review.

## Verdict

PASS for code and Rust. The separate gap analysis records evidence defects in the new performance and parity tests.
