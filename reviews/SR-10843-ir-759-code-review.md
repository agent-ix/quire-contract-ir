---
id: SR-10843
title: "Code review — IR-759 source-obligation correspondence spec"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/spec.md"
review_set: subset
---

## Summary

The spec-only diff is structurally valid and its new census scope is distinct from FR-045's downstream accounting. One duplicated API-boundary requirement remains in FR-052 despite the repo-wide rule already owned by FR-028 and FR-019; no applicable AssuranceProfile was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-052 repeats the global model non-reexport rule already owned by FR-028 and FR-019 | FR-052, FR-019, FR-028 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | FR-052 repeats the model dependency boundary already owned by FR-028 | FR-052, FR-028 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-052 now says the exported types are listed in FR-019 and no longer repeats a no-reexport requirement. |
| FND-002 | still-open | FR-052 still states that the model must not depend on owners, parsers, or proof runners; FR-028 already imposes the global no-owner-dependency and parser boundary. This is the same boundary requirement restated for this API. |
