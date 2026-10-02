---
id: SR-732
title: "EARS review of PR 249 (IR-504 content-only ModelOwner identity)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@fdad364d27f9e77c4525d06f2653eb2e69eca569; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (statement unchanged; new Behavior paragraph lines 271-283; edited lines 215-227, 259-269; FR-038-AC-45)"
review_set: subset
---
# SR-732: EARS review of PR 249

## Summary

Ticket: IR-504. The FR-038 requirement statement ("When Contract IR receives
checked-package bytes, the consumer shall ...") is unchanged and is an event-driven EARS
form. The edited "When a node is ... the reader shall reconstruct ... require the owner to
join ..." sentence keeps its `shall`. FR-038-AC-45 is a criterion, not a requirement
statement. Its trigger, condition and response are explicit: emitted under content-only
`ModelOwner`, it admits; with a `version` member, it refuses `unknown_member`; with an empty
`identity` or `node`, it refuses `invalid_semantic_graph`. It is compound, which matches
the file's own long multi-clause criteria (AC-43, AC-44).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new normative paragraph states its obligations descriptively ("The reader re-derives the node key from those three members alone", "refuses as `unknown_member`") with no `shall`. The neighbouring Behavior paragraphs use `shall`. Read as present tense, the paragraph also describes the reader as already doing what the matrix says is planned for IR-505. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:271-283 |

## Verdict

Acceptable with one low wording finding. Restating the paragraph as "the reader shall
re-derive ... and shall refuse ..." makes it an obligation and removes the implemented-tense
reading.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b091fe5 |
