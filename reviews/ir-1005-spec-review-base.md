---
id: SR-202610310110
title: "IR-1005 PR 749 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #749; FR-350, TC-461, TM-010 placement correction"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: references
---

## Summary

Ticket IR-1005. Reviewed frozen PR #749 head `981a27da480ef597a84098b028349ad4da11468b` against main `154227dcbcda09f3abf635375777893132fc7686`. The three-document diff moves the QSL-typed FR-350 bridge and TC-461 execution to `agent-ix/quire-integration` while leaving the Contract IR requirement as the cross-repository contract. FR-028-AC-3 and TC-041 prohibit QSL dependencies in either IR package. No code, copied artifact or new owner dependency is introduced. Targeted Quire validation reports 3/3 grammar-clean and zero grammar findings. No applicable AssuranceProfile was found.

## Verdict

CONDITIONAL. FND-001 leaves the edited TestMatrix internally inconsistent about withdrawn AC-7.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | TM-010 still lists withdrawn FR-350-AC-7 in its Functional Requirement Coverage row with TC-461 as the test case. The same edited matrix removes AC-7 from TC-461’s `Traces To` row, and FR-350 marks AC-7 withdrawn. The two rows disagree about what TC-461 covers. Remove AC-7 from the active coverage row or give it a separate withdrawn row without a test case. | spec/temporal/matrix/tests.md:12,18; spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md:279 | wrong-requirement |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| TM-010 | examined | \| FR-350 \| FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-7, FR-350-AC-8 \| TC-461 \| 🚧 Planned in `agent-ix/quire-integration` (IR-358). AC-1 through AC-6 and AC-8 remain unbacked until the QSL-typed bridge and executable correspondence tests carry their criterion trace tags there; AC-7 is withdrawn. FR-028-AC-3/TC-041 continue to prohibit QSL dependencies in both Contract IR packages. \| |
| TC-461 | examined | \| TC-461 \| Project native temporal subjects and join owner results through correspondence \| Integration \| FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 \| 🚧 Planned in `agent-ix/quire-integration` (IR-358); covers QSL classification, strict `/2` inputs, every bounded past operator, TL owner readers, native/TL joins, mutations and limits. \| |
| FR-350-AC-7 | examined | Withdrawn by the IR-1141 no-hash decision under QSpec #517/#518: the fixed 1429-byte vector, SHA-256 known answers and derived correspondence reference are removed. This id is retained and never reused. |
| FR-028-AC-3 | context_only | The root package's own manifest names no dependency in any kind on a crate from the agent-ix/quire-spec-language repository, by name or by git source. |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at head `b7d07bab311c3d1a4c6047ade54c2ce3ca0fe3d6`: TM-010's Functional Requirement Coverage row now lists only FR-350-AC-1 through AC-6 and AC-8 with TC-461, matching its Test Case Summary row. The status explicitly says AC-7 is withdrawn and has no TC-461 coverage. The one-file fix leaves the FR-350 and TC-461 placement text unchanged. |
