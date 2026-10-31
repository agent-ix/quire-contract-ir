---
id: SR-2026103042
title: "IR-1203 field read gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 745 field read source and tests; IR-1203"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: reviews
---

## Summary

Ticket: IR-1203. Planless review of the computed matrix and the PR's changed requirements, tests, and implementation. Tests bind FR-349-AC-8 and QSpec FR-322-AC-34 to real strict-reader behavior. The full repository matrix has historical untagged criteria beyond this PR's changed scope; no new trace gap was found in this diff.

## Verdict

**CONDITIONAL** — changed behavior has trace tags and behavioral assertions, but repository-wide matrix completeness cannot be claimed from this diff-scoped review. The required string-edge gate failure is recorded in SR-2026103041. No aggregate gate was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-349-AC-8, role: examined. Excerpt: “An external crate reads every admitted operation member and constraint production, identity, mode, ordered laws and ordered leaves through one borrowed typed view.” Tagged conformance and integration tests inspect the admitted typed value.
- FR-349-AC-9, role: examined. Excerpt: “The FR-019 exact public-item inventory and external compile probes show that IR exports the admitted read views but no QSL checked-expression constructor, `TermOperation` mirror, checker, or evaluator.” The changed public export and inventory were compared.
- FR-019-AC-5, role: examined. Excerpt: “The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision.” The existing public-interface test remains the inventory oracle.
- QSpec FR-322-AC-34, role: examined. Excerpt: “A missing `read_form`, an inapplicable `read_form` (including one on an undereferenced reference operand or an ordinary record projection), and an unrecognized value refuse `malformed_wire`.” Published RP-09 through RP-12 and RP-14 are consumed from the sibling checkout; an isolated reader test covers an undereferenced model reference.
- FR-038-AC-107, role: context_only. The published positive-state-transition fixture lacks the newly required form; its unmodified full positive sweep will fail until corrected upstream. The new test repairs its in-memory setup before applying published vector rows.
- Reverse trace: the new enum, intake conversion, and typed read field are owned by FR-349 and QSpec FR-322-AC-34; no QSL operation mirror or unowned evaluator was added.
- Semantic review: not separately requested; code-test alignment was checked as part of required code review.
- Plan completion: not assessed.

## Disposition pass 2 review

The fixture repair at e6d5aabf0db516af030b969b6887534956c7408d changes two existing traced integration tests. Each requires strict reader admission before its typed-member assertion can execute. The step-relation test follows both field reads through the admitted relation body, and the structural-key test checks the retained bare form in each selected range case. The new assertions fail if typed retention drops or changes the form. No new diff-scoped trace or test-oracle finding arose. The original clean placeholder remains a placeholder, not a finding needing disposition.
