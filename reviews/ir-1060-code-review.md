---
id: SR-610049
title: "Code and Rust review of IR-1060 PR 608"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/timed_model.rs; tests/conformance_qspec/{dimension_equivalence,extension,model_root,timed_model}.rs"
review_set: subset
---

## Summary

Reviewed PR #608 against IR main and QSpec. Ticket: IR-1060. The production match arm correctly exposes the typed operation-anchor members; two new test traces overclaim their criterion evidence.

## Verdict

**CONDITIONAL** — correct the two trace bindings on the original PR before the final gate. The author's focused conformance log records 74 passed, 0 failed, 1 ignored; the reviewer did not rerun Cargo while the shared aggregate build held the lock. No CI workflow or gate was changed.

## Coverage

Examined IR FR-038-AC-318, FR-038-AC-322, FR-038-AC-323, FR-065-AC-1, FR-065-AC-3, FR-065-AC-4, FR-065-AC-5, and QSpec FR-322-AC-132/134/135/136/137/138. Checked the published model-root vector, all seven changed fixture references, the public reader and typed term, existing FR-065 integration tests, source-map assertions, diff whitespace, Rust error paths, and repository conventions. The new vector test reaches all seven published MR rows and asserts refusal code, cause and pointer. The positive fixture test checks root order, owner, generated occurrence, source entry, derived key and package digest. No new copied fixture or schema appears in the PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new healthy tick test claims FR-038-AC-323 but never checks the selected base Time dimension or a wrong-dimension refusal; that criterion belongs on the existing TM-23..29 test, while this test exercises tick anchor retention and admission. | tests/conformance_qspec/timed_model.rs:17 |
| FND-002 | medium | Both new model-root tests claim FR-065-AC-1, whose external typed encoder and canonical read/write/read obligations neither test performs. The vector test reads published JSON and the fixture test inspects JSON and derives a root key; existing integration evidence owns the full AC-1 path. | tests/conformance_qspec/model_root.rs:37,80 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |, the tick test traces FR-038-AC-318 and FR-038-AC-322, while the existing TM-23..29 test retains FR-038-AC-323. The new writer/readback assertions also exercise AC-322. |
| FND-002 | fixed |, both model-root conformance tests trace FR-065-AC-3 only; the existing typed external encoder integration test retains FR-065-AC-1. |
