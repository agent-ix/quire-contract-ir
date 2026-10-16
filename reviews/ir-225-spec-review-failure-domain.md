---
id: SR-11672
title: IR-225 failure-domain review of catalogue-driven member-result admission
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir@39627960464edcc4138c29d20c18dbbcbefc603a (PR 408, diff vs origin/main c2b2a8b, merge-base 055ad1f); spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
review_set: subset
---

## Summary

Ticket: IR-225. Failure modes of moving the member-result check from an identity allowlist to catalogue-entry selection. Review-only.

## Verdict

**CONDITIONAL**: one low finding. The main failure modes are recorded in SR-11670 and are not repeated here: unresolved type families (FND-001), absent `declaration` (FND-002) and an allowlist implementation passing the cases (FND-003). The identity-confusion risk is handled: the check is keyed on the result form and member kind, and the five other member kinds are named and excluded.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Catalogue drift is no longer refused. Today's check refuses a to_float32/to_float64 application when its catalogued entry stops being `result: member`/`type_argument` or loses `member_family` (operations.rs:542-555, "refuse the typed package instead of silently skipping it"). Under entry-driven selection, a catalogue bump that changes an entry's result form, or drops its `member_family`, silently removes or weakens the result check, and AC-247 does not say whether that is intended. Failure: a later `quire-verification-contracts` revision mistakenly drops `member_family` from `ieee.to_float32`. The reader then admits a Float64 `result_type` with a matching Float64 type argument, and no test notices because AC-247's to_rational cases still pass. Say that the catalogue is authoritative and the drift guard is intentionally retired, or keep an AC-223-style regression on the IEEE pair that pins its family. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3526-3536,4326 |

## Coverage

Considered unresolved families, absent or misspelled members, an unknown `member_family` family or group name (`family_fits` returns false, so that case refuses, which is acceptable), cycles in bounded-domain chains (`resolve_family` returns `None` and lands in SR-11670 FND-001), refusal order (SR-11670 FND-004) and catalogue drift (above).

## Examined units

```yaml
scope:
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "this check is selected by the catalogue entry rather than the operation name. A mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus."
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Selection of this check follows the catalogue entry, not the operation identity. Other `member` kinds (`field`, `position`, `element`, `relationship_end` and `operation`) retain their member-specific result rules; their declarations need not themselves be the result type."
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: context_only
  excerpt: "Other member kinds retain their member-specific result rules. Compare typed code, cause, pointer and locus."
bindings: []
```

## Dispositions

Round 1; reviewed 9e608222751efbe09fe78c8260e3c5bcec143292 (diff 3962796..9e60822).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |

## Disposition round 1 verdict

**PASS**. The bullet and FR-038-AC-251 make catalogue drift fail closed for the currently catalogued type-argument member-result entries. A changed result form or member kind, or a removed required `member_family`, refuses the typed package. A member-kind change that disagrees with the wire member refuses first as `invalid_package`/`operation-member-mismatch` at `operation.member`, which matches the existing member-kind match at operations.rs:737-758. Otherwise the shape guard refuses `ill_typed`/`operator-ineligible` at `result_type`, keeping the outcome of today's guard (operations.rs:542-555) and AC-223. The TC exercises it at an isolated catalogue-entry seam. No new findings here; the `model_population` outcome gap is SR-11670 FND-006.

## Disposition round 2 verdict

Round 2 reviewed 5918af852bbdbe600181906bd4b71be371902762 (9e60822 plus 05014ba plus a merge of origin/main c2b2a8b). **PASS**: the round-1 `fixed` outcome still holds on the merged tree, no row was added and there are no new findings.
