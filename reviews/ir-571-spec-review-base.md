---
id: SR-15320
title: "IR-571 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038, FR-346; spec/checked_package/matrix/TC-225; current TC-225 implementation; QSpec FR-154-AC-14"
review_set: subset
---

## Summary

The parameter-name rule matches QSpec, but TC-225 reports planned cases as implemented.

## Verdict

**CONDITIONAL** — The parameter-name rule matches QSpec, but TC-225 reports planned cases as implemented.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-225 Status says Implemented even though its newly added selection-row and precedence cases are absent from the cited test. The existing test asserts the old frame-entry result. Change Status to planned or partial until the new public-reader cases land. | spec/checked_package/matrix/TC-225-checked-package-v2-abstraction-relation-body.md |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-225 Status now says Partially implemented, identifies the new selection-row and case-only controls as planned, and names the existing test that still asserts the former frame-entry refusal. |
