---
id: SR-2026102714
title: "IR-1096 PR #662 spec-review/failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Reviewed PR #662 exact against QSpec #485 merged. spec-review/failure-domain examined FR-038-AC-342, its behavior, and TC-048; no applicable AssuranceProfile found.

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
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. No findings.
