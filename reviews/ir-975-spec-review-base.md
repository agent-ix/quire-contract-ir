---
id: SR-15723
title: "Base spec review — IR-975 single-body V2 wire"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #521 at f08265affe2165e852c2febd85659ef68da2e70d; FR-038, FR-051, FR-349 and FR-019 changes"
review_set: subset
---

## Summary

The new FR-038 reader and FR-051 writer statements match published QSpec FR-322, but older FR-038 acceptance criteria still require manipulating a projection member that the same PR explicitly forbids on the wire.

## Verdict

**PASS at 8521036b21b13dd3057a34cd83e3fa54ff73a85b** — AC-112, AC-123 and AC-133 now match the single-body wire; the adjacent AC-113 type mismatch found in the first fix round is also corrected.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-123 requires patching a shipped `identity_projection` before recomputing `package_id`; AC-310 and the published QSpec schema forbid that wire member. State graph-only tamper and derived-hash recomputation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4860 |
| FND-002 | high | AC-133 still requires a QSpec negative fixture with a patched shipped projection; such a fixture now refuses at closed-wire intake before the specified stale-node-key check. State the graph-derived hash input instead. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4870 |
| FND-003 | low | AC-112 says the unrefreshed fixture retains `identity_preimage.identity_projection`, although published positives have no such member; name the graph-derived identity state instead. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4849 |

## Coverage

Examined the changed FR-038 wire and AC-310, FR-051 encoder statement and AC-4, FR-349 representation statement and AC-3, and FR-019 public interface entry. Cross-checked AC-112, AC-123, AC-133 and AC-154 for consistency. QSpec FR-322 at 4c687282 and its closed `IdentityPreimage` schema are the owning wire authority. Changed spec artifacts passed Quire structural validation. Plan completion: not assessed.

## Custody correction

This artifact supersedes the same reviewer method under SR-15716 on IR-975. SR-15716 had already been assigned to IR #522; the original finding text above is unchanged; later dispositions or head confirmations record the current verdict.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | AC-113 still requires hashing `CheckedPackageIdentityPreimageV2`; that owned type now encodes only seven selections, while the reader and conformance harness hash `CheckedPackageIdentityPreimageRefV2` over the graph-derived projection. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4850 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | AC-123 now derives package identity from seven selections and mutated graph nodes in graph order, excluding occurrences; it no longer patches a shipped projection. |
| FND-002 | fixed | AC-133's negative fixture now recomputes package identity from seven selections and mutated graph nodes; it no longer requires a shipped projection. |
| FND-003 | fixed | AC-112 now names the seven retained wire selections and states that the logical projection is derived from the mutated graph. |
| FND-004 | fixed | At 8521036, AC-113 names CheckedPackageIdentityPreimageRefV2 built from seven selections and semantic_graph.nodes in graph order, excluding occurrences, then encoded through quire-canonical. This matches the conformance helper. |
