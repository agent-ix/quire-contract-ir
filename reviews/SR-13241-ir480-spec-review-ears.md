---
id: "SR-13241"
title: "IR PR 480 EARS conformance: FR-035 read-limit statement and AC-11 to AC-16"
type: SpecReview
analysis: ears-conformance
review_set: subset
scope: "agent-ix/quire-contract-ir PR #480; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md lines 142-159 and 175-177 (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13241: IR PR 480 EARS conformance

## Summary

Ticket: IR-889. PR: agent-ix/quire-contract-ir#480 at the reviewed PR head.

Examined: the five sentences added to the FR-035 statement, the amended FR-035-AC-11, and the new
FR-035-AC-15 and AC-16.

- AC-11 keeps its state-driven "Given ..., `read` returns ..." shape.
- AC-15 and AC-16 are condition-and-response criteria. Each has one named reader, a stated
  precondition ("otherwise valid canonical package"), and paired boundary outcomes (admit at the
  total, `Incomplete` one below). Those are a single boundary obligation, not a compound one.
- None of the three uses a vague term.
- `quire validate` (quire 0.36.2, engine 0.50.2) raises no new warning on the three files.

## Verdict

ACCEPTABLE WITH ONE LOW FINDING. The criteria are well formed. The added statement sentences
mix normative SHALLs with rationale and change history, and some name no system as subject.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Three of the new FR-035 statement sentences do not name the system as the SHALL subject: "The counter SHALL continue", "The occurrence limit SHALL likewise report", and "A package previously admitted ... SHALL now return `Incomplete`". The statement also embeds rationale ("as the limit's documented meaning and the V2 reader's single-meter practice require") and transition history ("previously admitted", "now", "These decisions change ..."). That history describes the change, not the requirement, and goes stale once the code lands. The fix is to restate each with Contract IR (or the V1 reader) as subject and move the rationale and history to a note or the PR. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:145-157 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 36d5036 rewrites the statement as six "The V1 reader SHALL ..." sentences and removes the rationale and the previously/now transition history. |
