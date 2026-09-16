---
id: SR-581
title: "Base review of CheckedPackage V2 consumption"
type: SpecReview
analysis: base
scope: "QCI #106; FR-038; TC-047 through TC-050; TM-002 FR-038/TC-047–050 rows; PLAN-009 E01b"
review_set: subset
evaluated_revision: "agent-e/106-checked-package-v2 based on e463103"
review_date: "2026-09-16"
---
# Base review of CheckedPackage V2 consumption

## Summary

PASS after fixes. FR-038 adds one consumer requirement for the merged QSpec I04
V2 contract (`4780a9e6`) without amending FR-035. Every acceptance criterion is
verified by exactly one planned test case, and the normative QSpec criteria
FR-322-AC-4/8/10/11/12, FR-201-AC-5 and FR-195-AC-1..5 each map to an FR-038
criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-3 and TC-048 omitted the depth limit although the behavior charges nesting against it; fixed by adding depth to the exact/one-over limit set. | FR-038-AC-3; TC-048 |
| FND-002 | low | `requires_bound` is defined locally (reachable unbounded numeric/text/collection type without a reachable typing `bounded_domain`) because QSpec FR-195 names the disposition but not the predicate; recorded as an explicit IR-owned rule rather than an inferred upstream one. | FR-038 Behavior; QSpec FR-195 |
| FND-003 | low | Two statements lacked an EARS subject and trigger (migration and lowering paragraphs); fixed with explicit `When … the migrator/lowerer shall` forms. | FR-038 Behavior |
| FND-004 | low | Vendored fixtures carry placeholder raw digests with no preimage bytes, so the read context accepts caller-attested digests as well as bytes; scoped in Inputs so staleness remains a comparison against authoritative evidence. | FR-038 Inputs |

## Checklist result

- IDs FR-038 and TC-047–TC-050 are the next free identifiers; criteria use
  `FR-038-AC-N`; no duplicates.
- Inputs, outputs, closed result sums and every refusal code are enumerated.
- Error paths: every I04 refusal code, each migration refusal code and ordering,
  and every lowering non-success disposition has an acceptance criterion.
- Boundaries: exact and one-over for every read limit and lowering work.
- Compatibility: FR-038-AC-1 pins the V1 golden and cross-version refusals, so
  the frozen FR-035/TC-044 evidence is not relabelled.
- Matrix rows are `🚧 planned` until executable `tc_047`–`tc_050` tests exist.
