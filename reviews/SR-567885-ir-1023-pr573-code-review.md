---
id: SR-567885
title: "Code review of IR-1023 QSpec refusal word mapping"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/conformance_qspec/main.rs:404; FR-038-AC-285"
review_set: subset
---

## Summary

Reviewed quire-contract-ir PR #573. The single added `WrongExport` arm matches the declared and existing `wrong-export` spelling; the match remains exhaustive and the PR changes no fixture or assertion.

## Verdict

**PASS** — no code or Rust defect found in the changed line. `git diff --check` and `cargo fmt --check` passed. The coder reported focused target compilation; the independent reviewer did not rerun the conformance target. Current QSpec fixture admission failures require separate resolution and are not caused by this mapping.

## Reviewed units

- `tests/conformance_qspec/main.rs:356-405`: `cause_word` match, examined.
- `crates/quire-contract-model/src/checked_package/shared.rs:148-150`: `WrongExport` declaration, context only.
- `tests/it/checked_package_v2_temporal.rs:1172`: existing wire-word mapping, context only.
- `tests/it/checked_package_v2_protocol_body.rs:1921`: existing refusal assertion, context only.
- `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: FR-038-AC-285, context only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
