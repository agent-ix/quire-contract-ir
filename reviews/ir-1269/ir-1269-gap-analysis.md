---
id: SR-2026110101299
title: "IR-1269 PR 829 schedulability prefilter gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #829; FR-038-AC-398; FR-038-AC-399; schedulability body validation and public reader test"
review_set: subset
---

## Summary

Ticket: IR-1269. Plan completion: not assessed. Checked the changed reader behavior, criterion text, existing and new trace tags, and public reader assertions. No applicable AssuranceProfile was found under spec/.

## Verdict

**PASS for the PR diff** — FR-038 owns schedulability body validation and refusal precedence, and the new public reader test binds to FR-038-AC-398 and AC-399 with a fail-capable exact refusal assertion. No changed behavior lacks an owning requirement, and the changed code and test contain no stub. The computed repository matrix has 808 tagged, 131 untagged, 22 method-without-symbol, and 1 tagged-by-ignored-test criteria; all four examined criteria are tagged. The pre-existing repository-wide gaps are outside this PR diff and are not claimed resolved.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-398, role: examined. Excerpt: Wire and graph/aggregate validation precede carrier joins; declaration precedes occurrence, repeated key precedes order, coverage follows all rows, and carrier validation precedes FR-482 body validation. Combined defects distinguish each adjacent check. Each visited row charges one existing validation visit before inspection.
- FR-038-AC-399, role: examined. Excerpt: The reader admits the full FR-482 schedulable/response bodies and validates its shape, literal vocabulary, form fit, strict task order, common time unit and policy-specific task-set premises in that order after the protocol body step.
- FR-038-AC-380 and FR-038-AC-381, role: context_only. The existing 100,001-node test supplies the performance regression signal.
- Reconciliation: quire matrix (static axis, no run evidence) over the repository root. FR-038-AC-380, AC-381, AC-398 and AC-399 are tagged. The full matrix reports 808 tagged, 131 untagged, 22 method-without-symbol and 1 tagged-by-ignored-test criteria; these broader statuses were not newly introduced by this two-file diff. The evidence-backed quoin matrix did not complete before the static run.
- Inventoried changed behavior: one internal claim-position selection optimization; untraced behaviors: 0; changed source stubs: 0; changed test stubs: 0.

## Evidence

The new test is tagged TC-048, FR-038-AC-398 and FR-038-AC-399, drives CheckedPackageV2::read through the public path, and asserts exact InvalidPackage / InvalidValue and the earlier structural claim body member 1 locus. Existing long-index test is tagged FR-038-AC-380 and AC-381. The new filter is inside validate_bodies, which the public read path calls after validate_carrier and protocol body validation. No plan was supplied and no plan completion claim is made.
