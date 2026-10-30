---
id: SR-2026102967
title: "spec-review/base review of IR-1141 PR #717"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Reviewed PR #717 exact frozen head against IR main and QSpec main. The CheckedNodeId independent byte oracle is concrete, and recursion-group graph-order ordinals are preserved. QSpec FR-322 still contradicts the new ungrouped ordinal admission controls. No applicable AssuranceProfile was found.

## Examined scope

- `FR-038 structural ordinal behavior` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-038 application ordinal behavior` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-038-AC-76` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-038-AC-80` (context_only): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `FR-038-AC-364` (examined): spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- `TC-048 ordinal controls` (examined): spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
- `TC-048 canonical-byte controls` (examined): spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
- `QSpec FR-322 structural ordinal` (context_only): agent-ix/quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-364 requires admission of an ungrouped singleton with ordinal 7 and same-path nodes at ordinals 7 and 23, while QSpec FR-322 says ordinal 0 unless nodes share package+path and otherwise assigns graph-order positions. The reader may shape-check this input, but calling it otherwise valid contradicts QSpec’s producer rule; align QSpec before this IR requirement becomes authoritative. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4751 |

## Verdict

FAIL until the QSpec ordinal rule is reconciled and this test oracle matches it.

## Disposition scope

Checked amended FR-038-AC-364, TC-048, FR-038-AC-61 and merged QSpec FR-322-AC-51/52/272/273. AC-76 and the recursion-group rule remain as reviewed. The new source-owner and DefinitionOwner cases have distinct falsifiable refusal stages.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | QSpec FR-322 now assigns ungrouped ordinals to the producer, admits NO-05/06 and refuses exact duplicates NO-07/08; IR AC-364 and TC-048 cite and test those controls. |

## Disposition verdict

PASS at amended head 7c46f1faffe1eae55471e61ce893ce5bc59be18c; no new findings.
