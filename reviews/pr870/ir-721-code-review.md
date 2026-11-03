---
id: SR-20261103150657
title: "IR-721 PR #870 code-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; tests/conformance_qspec/monitor_plan.rs; tests/it/support/monitor_plan.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---

## Summary

Ticket: IR-721. Rust code review of PR #870: the QSpec adapter builds each published activation-status type and the public reader admits or refuses every published reader recipe.

## Verdict

**PASS** — No defects found in the scoped PR diff.

## Scope examined

```yaml
scope:
- id: FR-038-AC-421
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. The strict reader admits QSpec FR-487 MP-01
    and MP-02 as distinct temporal monitor-plan roots with their exact seventeen ordered
    bindings, State, Inputs, Scope, Results, four ordinary pure-function signatures,
    six-case outcome and exact integer/rational/text/unit premises. The anonymous
    ownerless root has generated source occurrences and semantic_type equal to State;
    generic owner/source-map checks retain precedence. A capacity edit remains admitted;
    the entries' expression bodies are checked without execution and flatten/sequence
    extension MP-32 adds no operation
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-721 lands. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    by reference from the selected QSpec checkout, constructs complete selected packages
    for the published reader admission recipes, and checks each published admission
    or refusal code, cause, primary RFC 6901 pointer and structural locus. It exercises
    MP-32 as an ordinary flatten/sequence expression without adding operation identity.
    The event_position and alternate activation-status cases are specified by AC-433.
    Execution and limit outcomes remain QSL/runtime responsibilities: their controls
    ar'
- id: FR-038-AC-433
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. The exact monitor State is now, history_start,
    the ordered buffer_<n> fields, activations, then integer event_position. By-reference
    QSpec MP-74..MP-76 refuse a missing event_position, text-typed event_position
    or placement before activations as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/1,
    retaining the root structural locus. Updated positive State contexts admit with
    no added root binding or public getter. By-reference MP-81 and MP-82 admit activation
    status typed as `record{position:integer}` and `text`, respectively, without a
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed changed files: tests/conformance_qspec/monitor_plan.rs and tests/it/support/monitor_plan.rs. CI workflow diff empty. No applicable AssuranceProfile was found under spec/. Repo AGENTS.md, CLAUDE.md, Rust style, code-review, and rust-review were applied. QSpec main supplies MP-01, MP-81 and MP-82. Focused checks were already run by the coder; aggregate gates are owned by the lead.
