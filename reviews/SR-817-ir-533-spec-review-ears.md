---
id: SR-817
title: "EARS and AC-shape review of PR 258 canonical encoding (IR-533)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@f3606b05780f22897c15fccc0eb2756ab284f14e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---
# SR-817: EARS and AC-shape review of PR 258 canonical encoding

## Summary

Ticket: IR-533. The amended statement keeps FR-038's "The reader shall recompute
`package_id` ..." form and adds the encoder as a means clause. The new section is
descriptive prose under that statement, as FR-038's other sections are. FR-019's paragraph
is a declarative surface statement, as its neighbours are.

AC-73 to AC-78 are direct assertions with no "shall", which is this repository's AC
convention. Each names its subject, its input and its observable result: the bytes, a
digest, a refusal code and pointer, or the absence of a stack overflow. AC-76 and AC-77
are compound, but they are no more compound than AC-70 to AC-72 on main. `make spec`
reports no grammar finding in the changed files; the only finding is FR-014's existing
one, which was there before and after this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean for requirement grammar and AC shape.
