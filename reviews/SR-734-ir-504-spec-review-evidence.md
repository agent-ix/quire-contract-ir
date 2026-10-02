---
id: SR-734
title: "evidence review of PR 249 (IR-504 content-only ModelOwner identity)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@fdad364d27f9e77c4525d06f2653eb2e69eca569; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-45 verification cell; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md TC-048 row"
review_set: subset
---
# SR-734: evidence review of PR 249

## Summary

Ticket: IR-504. FR-038-AC-45's method, `Test (TC-048)`, fits. The criterion is a
deterministic admit/refuse outcome over constructed packages, which is the same evidence
kind as AC-5, the sibling criterion it extends. TC-048 is a Property test case and
already owns the nominal model-owner procedure (lines 41-46, which already describe the
owner as `(identity, node)`). The new procedure paragraph builds the content-only owner,
checks the key does not change across a version-only change, and checks the `version`
member refusal.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The TC-048 procedure for AC-45 does not exercise the AC's empty `identity` and empty `node` refusals as `invalid_semantic_graph`. The existing model-owner procedure covers "empty the node" but not an empty identity. TC-048's Expected Results and Description do not mention AC-45's outcomes. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:13-23,48-52,54-68 |

## Verdict

Evidence method is correct. One low gap: the procedure covers three of AC-45's four
clauses.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b091fe5 |
