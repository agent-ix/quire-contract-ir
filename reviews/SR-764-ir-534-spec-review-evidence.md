---
id: SR-764
title: "evidence review of PR 252 (IR-534)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@31d2f9654bd4136ae64c1500074ce01793382af8; spec/checked_package/functional/FR-038-consume-checked-package-v2.md AC-2/10/19/20/27/31/32/45/62/63/64 verification column; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md; make spec coverage before/after; tests/it tags for the amended ACs"
review_set: subset
---
# SR-764: evidence review of PR 252 (IR-534)

## Summary

Ticket: IR-534. All amended and new criteria stay `Test (TC-048)`, which is the right method:
each is an observable reader outcome over a self-built package. TC-048 gains a "Selections bind by
identity" procedure that exercises every clause of AC-62..64, the same-identity pair in both orders,
and the `digest_domain_mismatch` outranking. The 🚧 markers are honest: AC-62..64 have no test,
and the matrix names IR-535 as the code ticket.

`make spec` before and after:

| | origin/main | head |
| --- | --- | --- |
| validate | passes | passes |
| grammar findings | 1 | 1 |
| strict unbacked rows | 23 | 23 |
| rows backed | 163/201 | 163/204 |
| FR-038 backed | 42/59 | 42/62 |

`make spec` exits 2 at both, on the 23 baseline unbacked rows. None of them is in
`spec/checked_package`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The amended criteria keep their existing `#[trace]` bindings: AC-10 in tests/it/checked_package_v2_reader.rs:1358, AC-27 in checked_package_v2_model_members.rs:282, AC-32 in checked_package_v2_dependency_selections.rs:124,186,210, and AC-20 and AC-19 likewise. Those tests assert the pre-amendment shape, for example the same-identity, different-version `malformed_wire` for AC-20 and the version-less entry for AC-32. `quire coverage --strict` therefore reports them backed (42/62), with 0 contradicted. The matrix prose admits it, but the mechanical count overstates backing until IR-535. Either accept this and say so in the matrix cell ("backed count includes tests asserting the earlier shape"), or have IR-535 re-tag in the same change. Record which in the cell so the count is not cited as evidence for the amended text. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:874 |

## Verdict

Adequate. One low finding: amended criteria still count as backed by tests that assert the old shape.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@a1f44f02c78fa786dc3e5582a45f46a069d74bd5 (delta from 31d2f9654bd4136ae64c1500074ce01793382af8; base origin/main eedc378d7fe879e0d107e04318773528f9595422 unchanged; merge clean). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/204 rows backed, FR-038 42/62. No new finding.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
