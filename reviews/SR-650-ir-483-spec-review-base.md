---
id: SR-650
title: "spec review of PR 238 (FR-038 Operation leaves, FR-038-AC-43, TC-048)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@b696d0e8363f01fa63de82f4cd6cd521c8d97943; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: base
---
# SR-650: spec review of PR 238

## Summary

Ticket: IR-483. Examined: the new FR-038 "Operation leaves" section, FR-038-AC-43, the TC-048 "Operation leaves" procedure, and the FR-038 and TC-048 matrix rows. Checked against QSpec FR-322 lines 174-191 and 631-633 and the upstream reference reader. No requirement was deleted. AC-43 is a single testable, falsifiable criterion that matches the head behaviour, and AC-42 is left free for PR 237. The TC-048 procedure matches the four tests. The matrix rows add AC-43 consistently. `make spec` lists the same 17 unbacked ids, none of them in this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The "Operation leaves" prose says `result_inner` compares "the inner type of a collection result". QSpec FR-322 limits it to the element type of a `set`, `bag` or `ordered_set` result. The prose restates the code's wider reading (SR-648 FND-001) as if it were QSpec's | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:482-486 |
| FND-002 | medium | "A type the walk cannot resolve decides nothing here" does not disclose two admissions. The reader admits `[]` over a cyclic type and over nesting deeper than 16, where QSpec's reference refuses. It also admits floats as having no text leaf, where QSpec makes a structural comparison reaching a float ineligible. The PR body and IR-483 state the float limit, but FR-038 does not | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:490-492 |
| FND-003 | low | The TC-048 matrix row names `tests/it/checked_package_v2_reader.rs` as its implementation. The AC-43 tests are unit tests in `operations.rs` | spec/checked_package/matrix/tests.md:23 |

## Verdict

Changes needed: FND-001 and FND-002 should be corrected together with the code fix. The limit the PR admits to (leaf paths and laws not compared) is stated honestly.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@7e10a1e3ec0d89d1d02bdf1e14f4b94dd99770b8.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The TC-048 "Operation leaves" procedure still lists only the round-0 cases. AC-43 now also covers flatten to a sequence versus a set result, the cycle and missing-node refusal, and the 12-level shared-field chain | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:108-115 |
| FND-005 | low | The "Operation leaves" prose says only that fewer leaves refuse. It does not disclose that surplus leaves are admitted: a probe admitted 2 leaves over an all-integer record, where the reference refuses `operation-law-mismatch` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:480-500 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7e10a1e: the prose now says "for a `set`, `bag` or `ordered_set` result only ... any other result, a `sequence` included, expects none" |
| FND-002 | fixed | 7e10a1e: the prose now states the cycle/unresolved `operator-ineligible` refusal, the work budget, and the float and leaf-path limits |
| FND-003 | fixed | 7e10a1e: the TC-048 matrix row now says AC-43 is verified by unit tests in `operations.rs` |
| FND-004 | fixed | 0f45961: the TC-048 "Operation leaves" procedure now covers flatten to sequence vs set/bag/ordered_set, `collection.set`, the cycle and missing-node refusals, 40-deep and 2000-deep nesting, the 12x4 chain and both ordering cases |
| FND-005 | fixed | 0f45961: FR-038 prose states that extra leaves are admitted where the reference refuses `operation-law-mismatch` |
