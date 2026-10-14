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
