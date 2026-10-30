---
id: SR-2026102939
title: "Trace and evidence gap review of V1 package digest removal"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@5afef435521b14806f3177cc33b9ee558728fd9f; PR #708 code and tests; IR-1141"
review_set: subset
---

## Summary

Ticket: IR-1141; PR #708 at frozen head 5afef435521b14806f3177cc33b9ee558728fd9f. Audited changed test tags against the computed Quire matrix and the named QSL conformance target. No plan was supplied.

## Verdict

**FAIL** — `FR-049-AC-6` remains tagged to a conformance target whose stale call to the removed `identity()` method prevents that control from compiling. The changed FR-035 bindings are otherwise coherent; withdrawn `FR-035-AC-5` has no remaining changed test tag.

## Examined units

- `FR-035-AC-2`, `FR-035-AC-3`, `FR-035-AC-4`, and `FR-035-AC-12` (`examined`): changed integration bindings and live reader/producer controls.
- `FR-035-AC-5` (`context_only`): withdrawn digest AC and removed tags.
- `FR-049-AC-6` and `TC-447` (`examined`): QSL conformance control and contextual V1 identity removal.
- Changed source and test paths (`examined`): all twelve PR diff files and the targeted caller `tests/conformance_qsl_ir769/main.rs`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-049-AC-6 QSL producer control is still tagged but cannot compile after the V1 identity API deletion. | tests/conformance_qsl_ir769/main.rs:197; Cargo.toml:48 |

## Coverage

The computed `quire matrix --scope . --format json` reports `FR-035-AC-5` untagged (withdrawn), live `FR-035-AC-2/3/4/12` tagged, and `FR-049-AC-6` tagged to the conformance target plus two ordinary integration tests. The named target's compile break makes its QSL-authored control hollow until the assertion compares complete canonical values instead. This is a diff-scoped assessment; pre-existing unbacked repository rows are outside this PR review. Plan completion: not assessed. Optional repository-wide semantic review: not run; the dispatch required a static PR-diff review while the shared build lock was occupied.
