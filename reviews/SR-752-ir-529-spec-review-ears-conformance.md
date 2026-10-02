---
id: SR-752
title: "EARS and AC-shape review of PR 251 (IR-529 artifact references)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038 statement, FR-038-AC-46 through FR-038-AC-64)"
review_set: subset
---
# SR-752: EARS and AC-shape review of PR 251

## Summary

Ticket: IR-529. The FR-038 requirement statement is unchanged and stays EARS
event-driven ("When Contract IR receives ... the consumer shall"). The 19 new ACs are
syntactically valid EARS (When/If-then ... shall). However, quire's grammar treats an
acceptance criterion as a verification statement, whose canonical shape is a direct
assertion of the outcome. All 182 existing ACs in this repo are written that way (for
example FR-038-AC-45: "An owner of kind `model` that carries a `version` member refuses
as `unknown_member` at that member"). On origin/main the grammar report has zero
`ac:non-canonical-shape` findings. At the head it has 19, one per new AC.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-46 through AC-64 are obligation-shaped ("the reader shall ..."). The repo's AC convention and quire's grammar (`ac:non-canonical-shape`, 19 new warnings against a clean baseline) want a direct assertion of the outcome. These are real violations of the repo's AC shape and should be fixed in this PR. Rewrite each AC as a direct assertion, e.g. "A definition reference carrying `revision` ... refuses as `unknown_member` at that member". The "shall" belongs in the FR prose, which is EARS. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:840-858 |

## Verdict

One medium finding. The fix is mechanical: rephrase the 19 rows, with no change in
meaning, and the grammar report returns to its baseline.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f (delta from 0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; base origin/main e80ea70ab8874676de47ac1b7fb439497ced7991 unchanged).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
