---
id: SR-2026102713
title: "IR-1096 PR #662 spec-review/evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Reviewed PR #662 exact head 98c4c975db84499b7388de9aacacf3ae91318f82 against QSpec #485 merged at 81078f141e1de6a9d6560c5b6b95de5775956f77. spec-review/evidence examined FR-038-AC-342, its behavior, and TC-048; no applicable AssuranceProfile found. The earlier-stage control is weaker than the criterion. `quoin advise --json` did not finish after more than one minute and was stopped; this finding is a manual criterion-to-plan evidence check, not a deterministic advisor verdict.

## Scope

- FR-038 (examined): For each of the timed root's `clock_places`, `resets`, `guards`, `invariants`,
`urgencies` and `constraints` reference aggregates, and for each
`clock_place`'s `guards` reference aggregate, the reader SHALL require the
members in strictly ascending order of their referenced node's source-region
key. That key is the minimum (`source.identity`, `start`, `end`) triple across
every region of every `source_map` entry for that node; identities compare as
UTF-8 bytes and positions as integers. Equal keys or a later member whose
key is below its predecessor SHALL refuse `invalid_se
- FR-038-AC-342 (examined): PLANNED/UNRUN (IR-1096). The public V2 reader admits QSpec's selected `AO-01` timed-slot package and `AO-05` tie-on-start package, including six root reference aggregates and a place's `guards` in strictly increasing minimum source-region order. It refuses each identity-current `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07` patch as `invalid_semantic_graph` with no cause at the vector's exact first out-of-order member locus; `AO-03` does not admit merely because references are in node-id digest order, `AO-06` compares ends when starts tie, and `AO-07` considers the earlier 
- TC-048 (examined): Status: PLANNED/UNRUN (IR-1096). Read the `aggregate_order` section of QSpec's
published `proposals/checked-package-v2/timed-slot-selector-vectors.json`
from the selected QSpec checkout. Apply `AO-01` through `AO-07` to the public
V2 reader without copying the selected package or patches into IR. Assert
`AO-01` and `AO-05` admit; assert `AO-02`, `AO-03`, `AO-04`, `AO-06` and
`AO-07` refuse with the vector's exact code, null cause and RFC 6901 locus.
Inspect the root's six reference aggregates and each place's `guards`: their
member order uses the minimum (`source.identity`,
- FR-322-AC-260 (context_only): The order key is the minimum triple (`source.identity`, `start`, `end`) over every region of every `source_map` entry of the node.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-048 permits just one of wire-shape, dependency or stale-key mutations ("or") for an order case. FR-038-AC-342 says all three earlier refusal stages win in each case. Require an independently discriminating control for each earlier stage on every AO order case, with the expected earlier refusal code and locus, so a timed-order verdict cannot mask a stage. | TC-048 |

## Verdict

REVISE. 1 finding(s).

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | AO-06 and AO-07 change only source_map. Reusing the root's pre-patch structural key cannot make that key stale; the required stale-node-key controls for these two AO cases do not exercise the earlier key stage. |
| FND-001 | fixed | TC-048 now makes AO-06/07 stale by appending a final clock-place reference while retaining their earlier order defect; the referenced node is already in the dependency set, so graph-shape stays satisfied and the retained root key becomes stale. |

## Current verdict

PASS at PR #662 head 3f04cba98b7649bea5d368b486c19e2f14e68aff. FND-001 is fixed in disposition round 2; the earlier still-open row remains as history.
