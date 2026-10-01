---
id: SR-702
title: "spec review of PR 244 (IR-448 NFR-001 methods)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@e94752093bd5ed619945964b9a6a035e48af5597; spec/core/non-functional/NFR-001-determinism.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-001
    type: reviews
---
# SR-702: spec review of PR 244

## Summary

Ticket: IR-448. Reviewed head e947520 against origin/main 968ba9b. The spec diff touches only
the Measurement and Evaluation table of NFR-001. The three Method cells now hold catalogue IDs.
M-1 and M-2 use `golden-approval-testing`, and M-3 uses `integration-testing`. The M-2 Metric
cell gains a planned note.

Both method IDs exist in the verification catalogue of spec-artifacts-process
(`manifest.yaml`). `golden-approval-testing` is class Test, evidence Snapshot, and is defined as
"compare output byte-for-byte against a reviewed, checked-in artifact".
`integration-testing` is class Test, evidence Integration, and is defined as "exercise two or
more components across their real boundary". After the change, no
`uncatalogued-verification-method` warning names NFR-001.

The Statement, the Scope, NFR-001-AC-1, NFR-001-AC-2 and all three metric rows are kept. The
Target and Threshold cells are unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The concrete measurement procedures were replaced, not kept: "Repeat the complete corpus twice and compare bytes", "Compare Linux, macOS, and Windows corpus outputs when CI is enabled" and "Compare ordered diagnostic code/path tuples". A catalogue ID names a kind of evidence, not what to run. No AC states the diagnostic-order procedure, so the spec no longer says how M-3 is measured. Keep each procedure in the Metric cell or the Verification section next to the method ID | spec/core/non-functional/NFR-001-determinism.md:27-29 |
| FND-002 | low | M-3, diagnostic order equality, cites `integration-testing`. The catalogue defines that as crossing a real component boundary, which says nothing about order. The check is a comparison of ordered code/path tuples against reviewed corpus expectations, which is what `golden-approval-testing` defines (stable output) | spec/core/non-functional/NFR-001-determinism.md:29 |
| FND-003 | low | M-1, cross-run byte equality, cites `golden-approval-testing`. The verification that actually runs (tc_018) asserts a relation between two executions of the corpus. The catalogue defines that as `metamorphic-testing` (ordering, idempotence, no independent oracle). Golden is defensible only because the corpus also compares against checked-in bytes. Consider naming both | spec/core/non-functional/NFR-001-determinism.md:27 |
| FND-004 | low | The M-2 Metric cell now embeds a status: "(planned: no executable test and no macOS or Windows CI yet; TC-019 is planned)". It is minted into the obligation name NFR-001-M-2, so it will go silently stale when CI lands. Status belongs in the TM-003 NFR-001 status cell, which already says "cross-platform comparison (TC-019) planned". The note also contradicts the TC-019 tag on tc_018 (see SR-701 FND-002) | spec/core/non-functional/NFR-001-determinism.md:28 |

## Verdict

Mergeable with respect to the spec diff. The methods are catalogued and no requirement, AC or
row is removed. FND-001 is the one worth fixing in this PR: the change dropped the only
statement of how diagnostic order is measured. FND-002 to FND-004 are judgement calls.

## Dispositions

Round 1, reviewed at 4941351784736182f05385b670f7860889f04cdf (fix commit 4941351).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4941351 |
| FND-002 | fixed | 4941351 |
| FND-003 | fixed | 4941351 |
| FND-004 | fixed | 4941351 |
