---
id: SR-958001
title: "code-review of QSL-1541/QSL-1542 guarantee and composition reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs, crates/quire-contract-model/src/checked_package/v2/guarantee_terms.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/typed_terms.rs, shared v2 match arms"
review_set: subset
---

## Summary

Code-review (with the rust-review lane) of quire-contract-ir PR #958 at e2553549 against origin/main 22b659ef, judged against QSpec origin/main 7e63321f (FR-490 and FR-491 are absent from the local QSpec checkout at b09800f7). Ticket QSL-1541, with QSL-1542. Interop judged against QSL PR #2549 (branch qsl-1541-1542-planner) node writers.

Focused runs: `cargo test -p quire-contract-model --lib tc_225_exactly_one_kind` FAILS; the TC-474 integration tests were run separately.

## Examined scope

- FR-038 "Implementation guarantee and component composition bodies" (examined): the new reader section and FR-038-AC-531..538.
- QSpec FR-490 "Checked-package node" (examined) and QSpec FR-491 "Checked-package node" (examined).
- FR-346-AC-2 (examined): exactly one kind has the abstraction relation body.

## Verdict

FAIL. The typed terms, encode and node-id remapping are complete and consistent, and the shared-file match arms add no behaviour. But the PR breaks an existing unit test, refuses every composition whose components live in other packages, and disagrees with QSL's writer on guarantee dependencies and on supplies order.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | body_grammar now maps three kinds to BodyGrammar::AbstractionRelation; the existing FR-346-AC-2 unit test tc_225_exactly_one_kind_has_an_abstraction_relation_body fails | crates/quire-contract-model/src/checked_package/v2/mod.rs:2825-2829, 5052-5063 |
| FND-002 | high | A composition whose subject or clause targets are nodes of another package is refused invalid_semantic_graph at dependencies[i] by the graph stage (a dependency not in the local index), before the guarantee step; QSpec FR-491 resolves them through FR-322 dependency packages. Tests use only local subjects | crates/quire-contract-model/src/checked_package/v2/mod.rs:3732-3760 |
| FND-003 | high | The reader requires a guarantee's fairness environment context in dependencies; QSL's ImplementationGuaranteeTerm::targets excludes it, so a QSL guarantee with a fairness premise whose context is not also a progress context refuses at the node | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:89-108 |
| FND-004 | medium | supplies order compares kind and granularity by enum discriminant (weak<strong, whole<each); QSL sorts by spelling bytes (strong<weak, each<whole), so two supplies on one operation emitted by QSL refuse out of order | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:556-565 |
| FND-005 | medium | A composition's environment entries get none of FR-490's checks: no assumption-is-predicate target check and no duplicate-name conflicting-binding | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:573-631 |
| FND-006 | medium | A guarantee's progress and fairness operations are resolved against their own context, not checked to be operations of the clause's model subject (QSpec FR-490 check 4) | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:472-491 |
