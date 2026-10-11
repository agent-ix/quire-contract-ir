---
id: SR-6514
title: scope-boundary review of planned application inventory correction
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir@7c760bc7b7e0c7b4216297db15ca4a61df50abd6; spec/model/functional/FR-019-rust-library-interface.md;
  spec/model/matrix/TC-058-model-crate-public-interface.md; FR-046 context; TC-058
  source oracle
review_set: subset
---

## Summary

Ticket: IR-738. Review is limited to PR 344's 5 added / 3 removed FR-019 lines and directly affected consumer procedure. The FR-046 contract and runtime status remain unchanged.

## Verdict

**PASS** — No defects in this method's changed-scope checks.

## Examined Scope

```yaml
scope:
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: '[FR-046](./FR-046-derive-candidate-application-node-identity.md) are

    PLANNED/UNRUN. They borrow a draft node and its ordered graph context, and return

    only a candidate key. A provisional self selector is not a verified semantic

    identity. The explicit canonical byte ceiling does not bound graph-context

    validation or body-copy memory. Reader admission remains required after

    final-key/reference substitution. Upon implementation, `quire_contract_model`

    shall export the function `derive_application_node_id` and types

    `ApplicationNodeDraft`, `ApplicationIdentityContext`, `CandidateApplicationNodeId`'
- id: FR-019-AC-5
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: '| FR-019-AC-5 | The `quire_contract_model` crate root has no glob re-export,
    and its default-feature public items are exactly those the Public items table
    lists, checked by a public-item inventory that fails on an added or missing item;
    no item in the table is reachable through a `quire_contract_ir` path. | Test (TC-058)
    |'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: context_only
  excerpt: 'upon implementation; this specification does not claim present exports
    or

    executed runtime evidence.

    '
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-738). Verify

    [FR-046](../functional/FR-046-derive-candidate-application-node-identity.md),

    AC-1, and the corresponding planned row in

    [FR-019](../functional/FR-019-rust-library-interface.md).

    Keep the existing source/spec inventory procedure scoped to its legacy items;

    this slice does not extend that source-inspection gate to the new candidate API.'
- id: tests/it/public_interface.rs
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "fn expected_items() -> BTreeMap<String, BTreeSet<String>> {\n    let (_,\
    \ after_heading) = FR_019\n        .split_once(\"### Public items\")\n       \
    \ .expect(\"FR-019 public items table must exist\");\n    let (table, _) = after_heading\n\
    \        .split_once(\"## Acceptance Criteria\")\n        .expect(\"FR-019 acceptance\
    \ criteria must follow the table\");\n    let mut items = BTreeMap::<String, BTreeSet<String>>::new();\n\
    \    for row in table.lines().filter(|line| line.starts_with(\"| `\")) {\n   \
    \     let cells: Vec<_> = row.split('|').collect();\n        let module = quoted_names(cells[1])\n\
    \            .next()\n    "
- id: crates/quire-contract-model/src/lib.rs
  path: crates/quire-contract-model/src/lib.rs
  role: context_only
  excerpt: '//!     authority: "a".into(),

    //!     identity: "i".into(),

    //!     digest_domain: "d".into(),

    //!     digest: "x".into(),

    //!     revision: "r".into(),

    //! };'
```

## Evidence

TC-058 expected_items splits after `### Public items` and before `## Acceptance Criteria`; only table rows beginning `| ` with a backticked module contribute. Independent replay against frozen source gives 249 exports and 249 table names, equal with no missing or extra names. Base ecd9ad2 has 254 names, missing only the five planned FR-046 exports; its parent 7805ce8 has 249 equal names. This is a read-only source check, not a runtime execution or new gate.

No Rust source, Cargo policy, CI, dependency edge, schema, reader admission rule, or resource contract changes. Source-oracle checks are folded into code-review. Cargo/Kani/full gates and full matrix were deliberately not run for this documentation diff. No applicable AssuranceProfile was found in spec. Exact configured model ID is unavailable; host identifies this agent only as Codex based on GPT-6. GitHub API verification was unavailable in this subagent (HTTP 401); the dispatching lead independently verified public repository visibility and PR 344 frozen head/base through its configured GitHub connector. PR body contains Part of IR-738, so no GitHub ticket comment is needed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
