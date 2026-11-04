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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The initial adapter allowed MP-82 to share MP-01’s resolved public root key because the activation-status change was absent from Activation, Activations, and State structural paths; the test exempted MP-81/82 from the key inequality assertion. | tests/it/support/monitor_plan.rs:336; tests/conformance_qspec/monitor_plan.rs:146 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The existing content-bearing rekey helper is applied to Activation, Activations, and State in dependency order. The public `plan.structural_key()` inequality assertion runs for MP-81/82 again, and the focused recipe test passes. |

## Follow-up review

Targeted review of the follow-up diff. The root path resolves the rekeyed State reference, whose path resolves the rekeyed Activations and Activation types, so changing status type is represented by structural operands. The adapter uses the existing path and graph-index convention; it adds no hash, digest, pin, copied upstream fixture, or case-ID-derived identity. MP-74..76 are now explicitly required to appear and refuse, while the generic loop continues to assert their exact code, cause, pointer, and root locus. No production reader code or CI workflow changed. The artifact’s original clean row and initial verdict are historical; this section records the later-discovered defect and its current fixed outcome.
