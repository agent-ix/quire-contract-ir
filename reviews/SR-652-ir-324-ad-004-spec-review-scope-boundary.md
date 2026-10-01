---
id: SR-652
title: "spec review (scope-boundary) of PR 239 (AD-004 QSpec to IR checked-package seam)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@a7924dc2c05d421e4323a28373dc0c546ec9917a; spec/assurance/AD-004-checked-package-seam.md"
review_set: subset
---
# SR-652: scope-boundary review of AD-004 (PR 239)

## Summary

Ticket: IR-324. This review checks the seam boundary against the code and against the sibling
seam ADs:

- CG PR 214 (AD-002 and AD-003, head d8d55ba).
- CG PR 215 (AD-004 crate layout).
- quire-driver PR 11.
- IR PR 241 (IR-323, AD-005 and AD-006, head d784ff7).

These points hold:

- The dependency direction (QSpec text, then IR, then QSL, CG and driver) is correct.
- IR never depends on QSL. This is stated, and it is consistent with both Cargo manifests and
  with AD-005.
- The reader path through the root glob matches CG's imports.
- AD numbering does not collide: AD-004 here, AD-005 and AD-006 in PR 241. CG's AD-004 is in
  another repo, and ids are per repo.

QSL and QSpec claims that come from ticket text are labelled untrusted (STD-129, and the IR-324
comment citing IR-480).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The closed operation catalog `quire.checked-operation-catalog/v1` is missing from both "What crosses the seam" and "Dependency direction". The reader validates every `application` operation against it. It comes from a third repository, `quire-verification-contracts`, through the model crate's dependency (`crates/quire-contract-model/Cargo.toml`, `v2/operation_catalog.rs`). R-S8 and Open questions say "state it in FR-322 or the operation catalog" without naming who owns the catalog | spec/assurance/AD-004-checked-package-seam.md:47-54, 73-79, 190 |
| FND-002 | medium | R-S2 (FR-331-AC-8 "SUCCESS check" wording) and R-S3 (the obligation-identity digest domain) are not about the checked-package seam. FR-331 is the backend provider envelope. Both rows are copied word for word from CG PR 214's AD-003:298-299, and this AD gives no context for them. Routing them in two ADs risks tracking the same gap twice. List only checked-package rows and point to CG AD-003 for R-S2 and R-S3 | spec/assurance/AD-004-checked-package-seam.md:212-213 |
| FND-003 | low | R-Q6 is already routed and accepted as QSL-353 (Coding), which the AD does not cite. :169-171 decides that "the cross-repo corpus belongs in QSL". The QSL review in the IR-324 comments (untrusted ticket text) puts "the cross-repo test" in quire-integration (F5), apparently for the replay seam. Cite QSL-353 and say F5 does not apply here | spec/assurance/AD-004-checked-package-seam.md:169-171, 205 |
| FND-004 | low | :172-174 spells two internal paths of `quire-specification`, which is a private repository, in this public one. One of them, `spec/objects/temporal/`, is new: FR-040 already spells the other. This does not breach the quire-research rule, but cite by id and title instead | spec/assurance/AD-004-checked-package-seam.md:172-174 |

## Verdict

Changes needed: FND-001 and FND-002. The boundary is otherwise drawn correctly. The AD does not
decide anything that QSL or QSpec owns. It contains no roadmap, delivery order, cycle
allocation, or quire-research content.
