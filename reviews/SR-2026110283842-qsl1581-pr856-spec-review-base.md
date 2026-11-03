---
id: SR-2026110283842
title: "QSL-1581 IR carrier spec base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-052 and FR-019 changed rows; QSL FR-514, FR-057, FR-255"
review_set: subset
---
## Summary

Reviewed PR #856 at its frozen head against the QSL owner contracts and current IR neutral carrier. No applicable AssuranceProfile is declared under `spec/`. The form and capability-kind decisions are concrete, but the exhaustion carrier loses a source identity and requires fields the active composed producer does not have.

## Verdict

**FAIL** — the two exhaustion-shape defects must be fixed in this spec PR before implementation.

## Scope

Examined FR-052 Inputs, neutral stop projection, and AC-10/26/28/29/30; the planned FR-019 public-items row; QSL FR-514 requirement forms and inventory cause; QSL FR-057 capability kinds; QSL FR-255 settings; and the QSL `Exhaustion` and `SourceRegion` types. AC-28/29 and the first-cell Planned statuses are sound. The diff introduces no hash, digest, pin, or copied vocabulary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Required owner-native `stage` and `setting` cannot be supplied by the active composed-lane `Exhaustion`, which has only dimension, limit, prior, requested and site; specifying both paths without an absence case forces invented values. | spec/model/functional/FR-052-source-obligation-correspondence.md:249 |
| FND-002 | high | `SourceByteRegion` retains only start/end, but a QSL source region includes its `RawSourceRef` authority and identity; a stage limit without a unit handle cannot identify which source document those offsets describe. | spec/model/functional/FR-052-source-obligation-correspondence.md:254 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Stage, setting and observed are optional in the neutral type; the spine supplies exact Some values, while the active composed lane leaves them None and retains measured prior/requested plus its native site. |
| FND-002 | fixed | SourceByteRegion now retains exact QSL RawSourceRef authority and identity with u64 start/end; it remains absent when the owner has no source reference, and AC-30 distinguishes equal offsets on two sources. |
