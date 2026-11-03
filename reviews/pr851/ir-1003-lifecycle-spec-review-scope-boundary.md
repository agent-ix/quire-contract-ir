---
id: SR-2026110283834
title: IR-1003 lifecycle spec-review/scope-boundary
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir; PR 851; ten historical Verification cells
review_set: subset
---

## Summary

Ticket: IR-1003. Reviewed PR 851 against main. Only ten Verification cells change across FR-035, FR-038 and FR-058; the statements, replacement references and identifiers remain byte-for-byte unchanged. No implementation credit is created. quire 0.38.1 (engine 0.51.0) used the explicit merged spec-artifacts-process provider contract from PR 128 and installed ISO module. Independent strict coverage matches the supplied candidate exactly: 99 to 89 unbacked, 22 to 32 retired, 890 backed and 966 criteria unchanged, zero status lies. Strict still exits 1. Seven obsolete TC-048 edges disappear as historical rows retire. All three changed documents validate. No Rust gate applies to this metadata-only scope.

## Verdict

PASS. FR-035-AC-12 already names replacement AC-31, whose six-member omission and added-identity controls remain live. FR-038-AC-10, 11 and 19 already withdraw impossible model_selections-array cases; AC-62 still rejects that removed field. AC-215 and 216 already withdraw digest comparisons; AC-27 still requires malformed-wire refusal. AC-59 already withdraws the absent preimage duplicate and names AC-366; AC-90 already withdraws the absent digest ceiling and names FR-051-AC-2. FR-058-AC-2 and 7 already declare their old same-node/different-point and occurrence-keyed-list cases infeasible and superseded by AC-9 and 10. Those replacements remain Test, with their PLANNED/UNRUN statements unchanged. None of these ten rows introduces a retirement decision.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

No AssuranceProfile document applies: none is declared under this repository spec tree. No active exception was used. Production gap-analysis, Rust, EARS, object, dependency, risk and failure-domain lenses are inapplicable because no source, requirement statement, domain object, relationship, boundary or failure behavior changes. Scope-boundary review confirms the same IR-owned reader/accessor and external QSpec contracts.

## Examined scope

```yaml
scope:
- id: FR-035-AC-12
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'Retired with replacement by FR-035-AC-31: its five-member omission list
    predates FR-330''s required `source_keys` frame. The omission and added-`identity`
    refusals remain required by AC-31; this id is not reused.'
- id: FR-038-AC-10
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: the lock holds no domain-package row and no digest, so there
    is no `lock.model_selections` array whose entries can repeat.'
- id: FR-038-AC-11
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: the lock holds no domain-package row and no digest, so there
    is no `lock.model_selections` array to order by defect class.'
- id: FR-038-AC-19
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: the lock holds no domain-package row and no digest, so there
    is no `lock.model_selections` array whose entries can carry classed defects.'
- id: FR-038-AC-215
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: a selected domain package document has no digest to recompute,
    compare or refuse as `content-mismatch`.'
- id: FR-038-AC-216
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: a selected domain package document has no digest, so there
    is no `byte-digest-mismatch`; bytes that do not read refuse `malformed_wire`/`malformed-json`
    (FR-038-AC-27).'
- id: FR-038-AC-59
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: no identity preimage mirrors definition selections and no stale-selection
    refusal exists. FR-038-AC-366 checks the single authoritative lock row.'
- id: FR-038-AC-90
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: `NominalIdentityPreimage::digest` and its byte ceiling do not
    exist. Canonical envelope byte-limit behavior is required by FR-051-AC-2.'
- id: FR-058-AC-2
  path: spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
  role: examined
  excerpt: 'SUPERSEDED by FR-058-AC-9 under QSpec #449 / IR-1040. Original planned
    text: For each of an invariant and a constraint node, two authentic claim occurrences
    share one node and body anchor but bind respectively to initialization `startup`
    and handler `tick`; the accessor returns distinct exact points by claim. Replacing
    `tick` with another handler name changes only that claim''s result. No clause-kind
    or anchor-derived default returns either value. The invariant and constraint have
    different node ids, as QSpec FR-341-AC-13 requires. The original same-node/different-point
    case is infeasible because the point now contributes to node identity; no implementation
    credit is claimed.'
- id: FR-058-AC-7
  path: spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
  role: examined
  excerpt: 'SUPERSEDED by FR-058-AC-10 under QSpec #449 / IR-1040. Original planned
    text: Two equal or conflicting point entries for one claim refuse reader admission
    with `InvalidExecutionPointCarrier`/`DuplicatePoint`; an entry for a non-claim
    or another node''s claim refuses with `InvalidExecutionPointCarrier`/`NonClaimPoint`.
    Each result names the offending clause/occurrence locus, returns no admitted package
    and creates no first-wins point. QSpec #449 has no point-entry list or occurrence-keyed
    carrier, so these exact cases are infeasible; no implementation credit is claimed.'
```
