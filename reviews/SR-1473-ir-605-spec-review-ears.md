---
id: SR-1473
title: "EARS and criterion review of PR 291 (typed STD-001 code, FR-044)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@c109f7eaba2d15f2d8d526f0b0f9dfdfc240a0dd; git diff origin/main...HEAD (base 6fb6e974efd6b9a9c74515ee7e07df250fa4aacf): FR-044 Description and AC-1 to AC-3, FR-030 new paragraph and AC-6"
review_set: subset
---
# SR-1473: EARS and criterion review of PR 291

## Summary

Ticket: IR-605. Requirement grammar of the new statement and the four new
ACs.

Measured: FR-044's Description is a ubiquitous "shall" statement naming one
system and one response. AC-2, AC-3 and FR-030-AC-6 are direct assertions
with concrete inputs and observable outputs, each failable; AC-3 writes out
the fifteen codes and a negative membership case; FR-030-AC-6 includes a
compile-fail probe and a negative deserialization. `quire validate` reports no
grammar finding on the PR's files.

Examined: FR-044 Description, FR-044-AC-1, AC-2, AC-3, FR-030 new paragraph,
FR-030-AC-6.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-044-AC-1's "a 65-byte string" does not say its bytes. Unless it is 65 bytes that would otherwise be valid (for example 65 `a`), a refusal can come from another rule and the 64-byte ceiling is not pinned; the one-mebibyte vector has the same gap. Write "a 65-byte string of `a`" | spec/core/functional/FR-044-typed-std001-code.md:117 |

## Verdict

Conformant; one low precision finding.

## Dispositions

Round 1, reviewed at cace2678410ee69ae71e656e26f80541ba2e31e4.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | cace267 |
