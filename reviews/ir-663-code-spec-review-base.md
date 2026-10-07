---
id: SR-2962
title: "Spec review of the IR-663 code slice status edits"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir branch code/ir663-private-intake-retention, head commit 'Charge missing-origin group candidates with an independent visit oracle'; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-663. The spec diff goes beyond status rows, so this review applies.
Four kinds of change:

- The FR-038 AC-174, AC-175 and AC-186 to AC-196 status cells change from
  PLANNED / UNRUN to IMPLEMENTED, with evidence notes. The criterion text
  is unchanged.
- Two normative FR-038 retention paragraphs replace their PLANNED wording
  with evidence statements (lines 1155-1156 and 1240-1244).
- TC-048's section heading and introduction are reworded, step 5 gains an
  inexact-number control procedure, and step 7 gains an evidence sentence.
- The FR-038 row in `checked_package/matrix/tests.md` and the
  `spec/tests.md` index row gain the AC range and a status sentence.

Checked and clean:

- No criterion statement or verification method changed.
- The added TC-048 step 5 procedure agrees with FR-038's rule that number
  admission comes first (lines 1175 and 1219) and matches the authored
  test.
- No SHA, local path or conflict marker appears in the diff.
- `quire validate` on all four changed documents exits 0. Its seven
  module-registry notices (DuplicateArchetype, DuplicateInverseEdge,
  inline-data-schema) are identical on main, are environmental, and are
  not about these documents. This is not a clean result.
- The matrix row's range list matches the computed matrix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Evidence claims are stale at the frozen head. Several statements assert results from an earlier head: "focused Tests passed" for AC-189/AC-190, "both Clippy lanes passed" for AC-195, "seven crate-local intake tests have passed", "both Clippy commands have exited successfully", "Clippy lanes have passed", and the matrix and index row sentences. The final commit changed production code (the group visit charge) and the group test that is AC-190's only binder. The commit before it changed the metadata and pre-declaration tests. These were written before code they describe changed, so the spec certifies a run that did not happen at this head. Reword to UNRUN or "awaits full gate" until the gates run at the final head. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1242,3861,3862,3867; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1250,1335; spec/checked_package/matrix/tests.md:16; spec/tests.md:15 |
| FND-002 | low | Run status is written into normative requirement prose. The FR-038 retention section now states test and lane outcomes ("checked by the crate-local intake tests", "Code-slice Inspection confirms", "have passed"). That evidence goes stale with every head, and it conflicts with the unchanged closing sentence at line 1256, which says these remain CODE gates and are not evidence supplied by this specification. Keep evidence in the AC status cells and the matrix rows. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1155-1156,1240-1244,1256-1257 |
| FND-003 | low | AC-186's status overclaims coverage. "IMPLEMENTED; focused Tests passed" is recorded although the tagged test asserts only 2 of the 4 end-coordinate presence combinations the criterion names (see SR-2960 FND-002). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3858 |

## Verdict

Not merge-ready. There are no high findings, and the criteria and
procedures stay consistent. The status and evidence wording claims runs
that the frozen head has not had, and it must be corrected or re-earned by
the full gates at the final head.
