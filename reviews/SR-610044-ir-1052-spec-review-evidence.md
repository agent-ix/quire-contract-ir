---
id: SR-610044
title: "spec-review/evidence review of IR-1052 PR #605"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md; spec/checked_package/functional/FR-067-admit-boolean-state-transition.md; spec/checked_package/matrix/TC-064-boolean-state-transition-reader-writer.md"
review_set: subset
---

## Summary

IR-1052 PR #605 at 385c3c392637836ad390bd3e77a9d151135ff07e; spec-review/evidence examined FR-040, FR-067 AC-1..7 and TC-064 against QSpec aa3ce8f4. 2 evidence gap(s) found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The procedure groups XT-08 and XT-09 with refusal vectors, although QSpec defines both as successful identity comparisons. It also omits explicit field, input, pre, post and profile rekey checks required by FR-067-AC-5. Separate identity outcomes from refusal cases and name all required rekey mutations. | spec/checked_package/matrix/TC-064-boolean-state-transition-reader-writer.md:28-36 |
| FND-002 | medium | The read/write/read procedure checks one claim occurrence only. FR-067-AC-6 requires two equal-content source claims to share one transition node while retaining both occurrence ordinals and regions; add that positive multi-occurrence case and its read/write/read assertion. | spec/checked_package/matrix/TC-064-boolean-state-transition-reader-writer.md:22-27 |

## Verdict

The evidence procedure needs the two changes above; remaining examined obligations align with QSpec.
