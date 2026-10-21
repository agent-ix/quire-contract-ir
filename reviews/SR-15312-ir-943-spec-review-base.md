---
id: SR-15312
title: "IR-943 PR 502 QSpec criterion consistency review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@95483029112d50a0f7ac743e0410668fa325f6e1; QSpec spec/objects/interfaces/FR-322-checked-package-artifact.md FR-322-AC-84..87; three-file IR PR diff"
review_set: subset
---

## Summary

Ticket: IR-943. Base spec review of merged QSpec FR-322-AC-84..87 as the governing text for the PR. The criteria are distinct, have explicit expected outcomes, and agree with the surrounding dependency-reference and function-parameter rules. The PR changes no specification text.

## Verdict

**PASS** — no defect in the examined criterion wording or cross-reference. The separate gap review records partial test evidence for AC-87.

## Coverage

- Examined FR-322-AC-84: lock order and first-failure cases are concrete.
- Examined FR-322-AC-85: three refusal loci are distinguishable and testable.
- Examined FR-322-AC-86: signature node set and both adverse/positive cases are explicit.
- Examined FR-322-AC-87: compiler step order, ordinal, and source-region controls are explicit; compiler ownership is outside this IR PR.
- Checked IDs, references, criterion separation, and test method labels. No new digest, pin, or tracking record is introduced by this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
