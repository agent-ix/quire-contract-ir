---
id: SR-15327
title: "IR-571 PR 503 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #503 six changed Rust source/test files; IR FR-038, FR-346-AC-7, TC-225 and QSpec FR-154-AC-14"
review_set: subset
---

## Summary

Changed tests cover single-defect names and collision priority, but do not discriminate name-vs-reference precedence.

## Verdict

**CONDITIONAL** — Changed tests cover single-defect names and collision priority, but do not discriminate name-vs-reference precedence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The nine public-reader adverse cases mutate names or identities only; all leave parameter typeRef valid. None checks an invalid name plus a missing typeRef, or a repeated identity plus a missing typeRef, in one selected declaration. The source ordering regression where missing-reference wins is invisible to tagged TC-225. Add combined-defect controls requiring malformed-declaration and conflicting-binding respectively at the selection row. | tests/it/checked_package_v2_abstraction_relation.rs:1771-1846 |
## Coverage

Plan completion: not assessed

Changed criterion FR-346-AC-7 has a new public `#[trace("TC-225", "TC-280", "FR-346-AC-7", "FR-154-AC-14")]` test. The repository-wide computed matrix was attempted but did not complete during this bounded PR review; this is a manual changed-criterion trace and oracle assessment, not a repository-wide PASS claim.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The tagged public TC-225 table now includes an invalid name plus missing typeRef and a repeated identity plus missing typeRef, with expected malformed-declaration and conflicting-binding causes. Its shared assertions also require InvalidModelBinding at `/lock/model_selections/0`. |
