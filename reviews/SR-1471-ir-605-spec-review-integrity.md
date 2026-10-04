---
id: SR-1471
title: "integrity review of PR 291 (typed STD-001 code, FR-044)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@c109f7eaba2d15f2d8d526f0b0f9dfdfc240a0dd; git diff origin/main...HEAD (base 6fb6e974efd6b9a9c74515ee7e07df250fa4aacf): spec/core/matrix/tests.md, spec/core/matrix/TC-442, spec/kani/matrix/tests.md, spec/kani/matrix/TC-223, spec/tests.md, spec/spec.md, FR-044, FR-030, FR-039, FR-019, STD-001, AD-006"
review_set: subset
---
# SR-1471: integrity review of PR 291

## Summary

Ticket: IR-605. Structural and cross-document consistency of the matrices,
index rows and the FR-019/FR-030/FR-039/AD-006 edits.

Measured: FR-044's three ACs map to TC-442 in the core matrix, TC-442's
Traces To is FR-044 and its frontmatter `verifies` FR-044; FR-030-AC-6 maps
to TC-223 in the kani matrix, TC-223's row lists AC-4 to AC-6 and its
Description and Procedure cover AC-6. All new rows say planned and every AC is
a direct assertion with `Test (TC-...)`. `spec/tests.md` lists FR-044 under
Core and FR-030-AC-6 under Kani. Strict coverage moves 23 to 29 unbacked; the
delta is exactly FR-044, FR-044-AC-1..3, FR-030-AC-6 and TC-442. FR-019's
Public items row (`code`: `Std001Code`, `Std001CodeError`) equals FR-044's
Contract `types`. FR-039's "names but does not re-export" agrees with FR-044
Home and merged FR-039's no-re-export rule. FR-030's paragraph, FR-030-AC-6,
STD-001's cause table and AD-006 Decision C agree with each other and with
merged FR-030-AC-4/AC-5. `quire validate` is clean for the PR's files.

Examined: the five matrix and index rows the PR edits, TC-442, TC-223 edits,
FR-019 serde paragraph and Public items row, FR-039 Error surface, spec.md
paragraph.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Two prose rows lag the PR. The spec.md sentence now reads "STD-001 is the stable diagnostic code registry, and FR-044 exports its code as the typed `Std001Code` ..., which `KaniOutcome.code` carries; STD-003 the closed output-mapping refusal registry", which has lost its verb in the second clause. The core matrix StR-001 row adds FR-044 and TC-442 to its traces but its status prose still lists the planned items without the typed code | spec/spec.md:95-97; spec/core/matrix/tests.md:12 |

## Verdict

Structurally sound; one low wording finding. The matrix, TC, index and
coverage changes are consistent with each other and with merged text.

## Dispositions

Round 1, reviewed at cace2678410ee69ae71e656e26f80541ba2e31e4.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | cace267 |
