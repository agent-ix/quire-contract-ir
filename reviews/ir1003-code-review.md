---
id: SR-202611070951309509
title: "Code review of IR-1003 guarantee and composition typed values"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_guarantee_composition.rs; FR-038-AC-531 and FR-038-AC-532"
review_set: subset
---

## Summary

Ticket: IR-1003. Reviewed PR #963's two new integration tests using code-review with the rust-review and rust-style lane, and manual acceptance-criteria alignment. No applicable AssuranceProfile exists under spec/. The branch changes only the two tests; no gate, source, fixture helper, or specification changed.

## Verdict

**PASS** — no defects found. The empty-list guarantee test checks the actual admitted typed variant, both empty lists, and full serialized typed-body equality. Each composition rule has an exact string assertion, distinct premise-value assertions and full typed-body equality. Expected bodies are cloned from the input package before admission; they do not use the decoder or an admitted output to construct expectations.

The composition fixture distinguishes Consumer/safety from Producer/liveness and reverses their on-target components. Replacing all rules with assume-guarantee fails the other three iterations; dropping, merging, swapping or copying one premise list into the other fails the value assertions. Losing any other body member fails complete equality. Both let-else branches panic on a wrong typed variant, and admission failure cannot skip assertions. Existing shared fixture helpers are reused, and no copied implementation or new duplicate helper was added.

## Examined scope

- FR-038-AC-531 — role: examined; path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md; excerpt: A package holding one `correspondence`/`implementation_guarantee` node over a clause selecting the infinite-trace profile, an `abstraction_relation` node, a `predicate` node and the `Order` object type admits for a body with `initial`, a `decreases` and a `bound` progress member and a fairness and an assumption environment member, and the typed term reproduces the body; a body with empty `progress` and `environment` lists admits.
- FR-038-AC-532 — role: examined; path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md; excerpt: A package holding one `correspondence`/`component_composition` node admits for a body with two components, two guarantees, one `relies`, one `liveness_dependencies`, one `supplies` and one `environment` entry, the typed term keeps the premise lists separate, and each of the four rule spellings admits.

The existing TC-474 guarantee and composition admission tests were read as context for the complete ACs, including populated progress/environment. The new tests preserve exact two-ID trace attributes and bind only their corresponding criterion. The four composition rule literals are independent behavioral expectations, not a second decoder. No production API, concurrency, unsafe operation, external service, or new panic path is introduced.

## Verification

Inspected the supplied /tmp/ir1003-main-focused.log: workspace focused compile and Clippy completed successfully, and all 17 guarantee/composition tests passed, including both added tests. Formatter and diff-check success were reported by the coder. No redundant Cargo rebuild was run. No make ci, Kani, hosted CI, aggregate gate, matrix or full coverage recalculation was run, following the dispatch constraints. The measured baseline remains 1080/1196; this review claims no newly backed row. IR-724 owns the one post-landing strict measurement.

The PR body preserves the exact withdrawn fourteen FR-038 rows, deferred AC-195, and cross-repository FR-350 integration ownership and open credit question. The requested generation footer and commit co-author trailer are present. Full gap-analysis is inapplicable to this tests-only change; spec-review is inapplicable because no normative artifact changed. Manual AC alignment is recorded above rather than a new coverage measurement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
