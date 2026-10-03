---
id: SR-1016
title: "spec review of PR 266's matrix edits and the FR-038 text it implements (IR-274 part A)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@476e9b418e69aaaf7e796e3923e3d8321a812d50; spec/checked_package/matrix/tests.md (FR-038 and TC-048 rows), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, read against spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Canonical encoding of the wire types; Every identity digest is computed through quire-canonical; A model document is a value inside a supplied document; AC-89..95) and spec/assurance/AD-005-qsl-consumption-seam.md"
review_set: subset
---
# SR-1016: spec review of PR 266

## Summary

Ticket: IR-274 (part A). The PR edits two matrix files and leaves the FR text
alone. I checked the edited rows against the code in this PR, and the merged
FR-038 text against what part A can satisfy.

**AC-91 and AC-92 need no amendment.** AC-91 names `canonical.rs`,
`binding.rs` and `output_mapping.rs`. AC-92 names the bound identity envelope.
These are parts B and C of the same ticket, which the spec announces as
"spec then three code changes". Neither AC can pass in part A alone, and the
PR correctly leaves both 🚧 with the scope stated. That is the intended split,
not a spec gap.

**Counts.** `make spec` passes validate with 1 grammar finding (FR-014, not
this PR). `--strict` reports 23 unbacked rows and 0 contradicted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-038 matrix row this PR edits still says "that upstream change (quire-canonical #7) is open and pending merge, so until it lands AC-77 and AC-78 stay verified against this repository's own walker (`value_to_vec` in `encode.rs`), which stays; AC-91's no-walker and feature clauses wait for it, and code change A is blocked on it". The same PR deletes `value_to_vec`, enables the `serde_json` feature, and pins the lock to #7's merge commit b4bb97a5 (#7 merged 2026-10-03). The row now contradicts the code beside it, and says AC-77/78 are verified against a walker that no longer exists. Rewrite the sentence in this PR | spec/checked_package/matrix/tests.md:15 |
| FND-002 | medium | Spec gap, to route to the IR spec lane. FR-038 says an over-ceiling lowered node or package is reported `failed`, but `CompleteLoweringRecordV2::Failed` has only `{node_id, limit, consumed}` and is documented as "This request exceeded its work budget". The spec does not say what a byte-ceiling failure records. The code reports `limit: profile.work_limit` with `consumed: 0` for the package case, and with the work spent for the node case. So a byte-limit failure reads as a work-budget failure, and the package case is self-contradictory (consumed below limit, yet failed). The sentence to amend is in "Every identity digest is computed through quire-canonical": "A lowered node whose preimage's canonical bytes exceed the retained limit is reported `failed`, and a lowered package whose bytes exceed it is reported `failed` for every requested record". Say which limit kind and values it carries (the retained `bytes` limit and the preimage length), or add a distinct record variant, and mirror it in AC-95 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:450 |
| FND-003 | low | Stale prose, to route to the IR spec lane: the FR text still describes #7 as pending. FR-038 "Canonical encoding of the wire types" says "That upstream change is quire-canonical #7, open and pending merge, and the code change that depends on it is blocked until it merges" and "`value_to_vec` ... which stays; this requirement does not specify removing it before the upstream `Encode` exists". AC-91 ends with "Planned, and blocked on `quire-canonical`'s pending `Encode` for `serde_json::Value` ... until that upstream change lands the scan excludes `value_to_vec`". AD-005 line 131 also says "open and pending merge". All of this is now false. The code's deletion is the stated target, so the code is right and the text is stale | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:338 |

## Verdict

Changes requested for FND-001, which sits in this PR's own diff. FND-002 is a
real spec gap. It goes back to the spec lane, and the code's `limit` and
`consumed` values should follow whatever the spec settles. FND-003 is a
stale-text amendment for the spec lane, and need not block part A.

## Dispositions

Round 1, reviewed at 249acbcd028536586d0e8162b1a15ddfc59e75a9.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |
| FND-002 | deferred | The spec amendment is quire-contract-ir #267 (open, head ec21b9c), which names `limit_kind`, `limit` and `consumed` for a byte failure. Code and #267 still differ on non-canonical-bytes encoder refusals (SR-1014 FND-006) |
| FND-003 | deferred | quire-contract-ir #267 rewrites the #7-pending prose in FR-038 "Canonical encoding of the wire types", AC-91 and AD-005:131 |
