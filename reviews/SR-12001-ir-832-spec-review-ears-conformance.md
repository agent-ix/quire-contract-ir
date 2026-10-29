---
id: SR-12001
title: "IR-832 EARS review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir (PR #425); spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Ticket: IR-832. Each new normative sentence in FR-038 "Typed parameter declaration" was checked for a named subject, one `shall` obligation, a concrete response and a condition where one applies. Scoped `quire validate` exits 0 on the four changed files. The ACs are verification statements and are judged for falsifiability in SR-12003, not as EARS statements.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| API statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | `CheckedPackageV2::parameter_declaration(...)` shall return it as typed Rust values, so the consumer does not decode the package's `body`. |
| Identity statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The accessor shall identify a parameter by node id alone and shall return no enclosing-function identity. |
| UnknownNode statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | An unknown id shall refuse `UnknownNode`. |
| NotParameter statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A node whose tag and form are not `value`/`parameter` shall refuse `NotParameter`, whatever its body. |
| MalformedBody statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A parameter node whose body is no longer the admitted two-binding shape, which only a crate-internal post-admission mutation can produce, shall refuse `MalformedBody`. |
| Refusal-properties sentence | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Each refusal returns no partial result, the accessor never panics, and none changes package admission or the V2 reader's refusal order. |
| Caller-join paragraph | context_only | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Binding concrete values is the caller's join and not an IR API ... |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The refusal-properties sentence carries three obligations (no partial result, never panics, admission and refusal order unchanged) in indicative mood with no `shall`. Neighbouring sentences use `shall`, so a reader cannot tell whether these are normative. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3160-3161 |

## Verdict

PASS WITH FINDINGS. The API, UnknownNode, NotParameter and MalformedBody statements are ubiquitous or event-driven obligations with a named subject and an observable response. The identity statement joins two `shall` clauses that make one design decision, which is acceptable. The caller-join paragraph is correctly non-normative.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 : the sentence is split into three `shall` statements: no partial declaration, no panic, admission and refusal order unchanged. |
