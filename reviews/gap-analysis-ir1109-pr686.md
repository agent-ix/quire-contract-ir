---
id: SR-2026102750
title: "IR-1109 PR 686 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #686; tests/conformance_qspec/timed_model.rs"
review_set: subset
---

## Summary

Reviewed the one-file test diff at exact against merged IR FR-038 and QSpec. QSpec CL-25/54, GD-02/03 and TM-12 now have exact public-reader code, cause and locus controls; the clause fixture’s selected document digest is independently recomputed. No copied QSpec file, production code, dependency, CI workflow or gate changed.

## Verdict

**CONDITIONAL** — Two newly added trace claims have incomplete positive retention evidence; the repaired vector oracles themselves are sound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new AC-320 tag is on a refusal-only vector loop. It never admits CK-25 or inspects the admitted model’s normalized constraints, although AC-320 requires both. The positive round-trip test does not carry AC-320 and only checks retained raw node bodies. Add an independent normalized-constraint oracle and bind it, or keep AC-320 untagged until backed. | tests/conformance_qspec/timed_model.rs:1082-1084 |
| FND-002 | medium | The newly added AC-360 tags cover CL-25/CL-54 refusal and admission but never assert that the admitted anchor retains its declaration occurrence exactly once or that operation urgency retains its model-plus-name body. Those are explicit parts of AC-360. Add public-reader retention assertions or remove AC-360 from these partial controls. | tests/conformance_qspec/timed_model.rs:115-118,199-201 |

## Coverage

- Examined FR-038 AC-320, AC-336/340/342 and AC-357 through AC-362; QSpec FR-322 AC-120/124/260/262/266 and the published CL-25/54, GD-02/03, TM-12 vectors.
- Plan completion: not assessed.
- `quire matrix --scope . --format json` completed. AC-320 is tagged only by the refusal-vector loop; AC-336/340/342 remain untagged. Planned AC-357 through AC-362 are not active matrix rows.
- Rust review: loaded rust-review/rust-style and repository AGENTS.md/CLAUDE.md; inspected test assertions, fixture loading, trace tags, canonical digest helper, and absence of vendoring. The lead owns the final aggregate gate.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The refusal-only TM vector loop no longer carries FR-038-AC-320; positive CK-25/normalized-constraint evidence remains outside this PR’s new trace claim. |
| FND-002 | fixed | The new public-reader positive test checks four clause-to-anchor references, one retained anchor occurrence with Anchor role and ordinal zero, no execution point and root-model context, plus the operation urgency’s exact kind/model/name body. |
| FND-003 | still-open | The schema-refusal arm does not assert the required null cause for CL-17..20/27/28; AC-362's IMPLEMENTED status is premature until this oracle is added. |
| FND-003 | fixed | The schema-refusal arm now asserts `refusal.cause == None` alongside exact `MalformedWire` code and locus for CL-17..20/27/28; the focused vector test passed at 72334867. |
| FND-004 | fixed | Removing the blank line before AC-343 restored Quire minting for AC-343 and AC-357..362; matrix reports AC-343 untagged and the six implemented criteria tagged. |
| FND-005 | fixed | The three named positive/guard/equal-region tests now carry AC-357, AC-358 and AC-359 trace tags respectively; Quire trace resolves their claims. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | AC-362 is now marked IMPLEMENTED, but the public-reader vector loop checks only `MalformedWire` and locus for `refused-by-schema` CL-17..20/27/28. It omits the exact null cause required by AC-362's code/cause/locus oracle for CL-01..54. Assert `refusal.cause == None` for these six schema cases before claiming the criterion fully backed. | tests/conformance_qspec/timed_model.rs:326-336; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5588 |

## Final delta review

At frozen, AC-357..362 remain verbatim after their status/evidence prefixes. The two-unlisted-node public-reader control distinguishes digest from graph order and checks the exact first `/body` refusal; the positive clause test checks all seven closed forms and exact writer round-trip. QSpec CL-33 is an admitted `eq` constraint outside invariants, covered by the 54-case vector loop. FND-003's null-cause assertion completes the six schema-refusal oracles. All three findings' latest dispositions are fixed. The lead owns the final aggregate gate.

## New findings (disposition pass 4)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | A blank line before AC-343 split the criterion table, so AC-343 and AC-357..362 were not minted even though the latter had Rust trace tags. The matrix could not bind the newly IMPLEMENTED criteria. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5581 |
| FND-005 | medium | The IMPLEMENTED evidence named three tests whose trace tags did not bind their criteria: AC-357 positive closed-body round-trip, AC-358 omitted-guard refusal, and AC-359 equal-region ordering. The other named controls were bound. | tests/conformance_qspec/timed_model.rs:45,966,1791 |

## Final trace disposition

At the reviewed PR head, Quire resolves AC-357 with both named tests, AC-358 with all four named tests and AC-359 with all three named tests. AC-360..362 named tests were already bound. AC-343 is minted and untagged, so it remains honestly unbacked. The code/test behavior is unchanged by these final three trace edits.
