---
id: SR-202611070951309494
title: IR-1398 PR949 spec-review-ears-conformance
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir; IR-1398; own PR949 source/spec/test delta above
  IR-1396 owner branch, current-main interactions
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-038
  type: references
---

## Summary

Ticket: IR-1398. Reviewed frozen 304de1e7 against the actual IR-1396 owner stack, excluding inherited main artifacts and unchanged owner implementation. No applicable AssuranceProfile exists under spec/. This qualification is on the unmerged owner stack; merged-main composition and the root aggregate gate remain pending.

## Verdict

PASS — No defect found in this lens.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-038-AC-515
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented and qualified on the IR-1396 owner stack; merged-main qualification
    pending (IR-1398). For QSpec TC-187 CS-01, the strict reader SHALL admit each
    Integer, Decimal and Rational carried-sum application and retain the exact operation
    identity, ordered collection and binder operands, binder bound, type_argument,
    result_type, rounding mode, empty laws and empty leaves. The bound, member and
    result each resolve to the same carried node Q: Integer[-5,5] with exact, Decimal[-500,500;2,2]
    with nearest-even, and Rational[-3,7;1,12] with exact respectively.'
- id: FR-038-AC-516
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented and qualified on the IR-1396 owner stack; merged-main qualification
    pending (IR-1398). For every CS-02 variant, the reader SHALL refuse invalid_package/operation-mode-type-mismatch
    at the application: wrong mode for the Integer or Decimal positive; binder alone
    or result alone changed to Qn (nearest-even); and a unit-node binder beside the
    non-exact Decimal result. Mode pinning precedes binder/member and result/member
    equality and reads no type_argument.'
- id: FR-038-AC-517
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented and qualified on the IR-1396 owner stack; merged-main qualification
    pending (IR-1398). For CS-03, the reader SHALL refuse a null mode as invalid_package/operation-mode-mismatch
    and a laws entry as invalid_package/operation-law-mismatch at the application.
    In combined-defect controls laws win over null mode and type mismatch, and null
    mode wins over a binder/member mismatch.
- id: FR-038-AC-518
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented and qualified on the IR-1396 owner stack; merged-main qualification
    pending (IR-1398). With pinned mode agreeing, the reader SHALL refuse every CS-04
    binder/member mismatch as ill_typed/operator-ineligible at the application: a
    different carrier bound Q4, unit Qc, carrier kind Qk, or unit-node Um; and binder
    and result both Qn under nearest-even while type_argument remains Q. The bound_is_member
    constraint compares the resolved bound against the member, not against the result
    alone.'
- id: FR-038-AC-519
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented and qualified on the IR-1396 owner stack; merged-main qualification
    pending (IR-1398). With pinned mode agreeing, the reader SHALL refuse every CS-05
    result/member mismatch as ill_typed/operator-ineligible at the application, for
    result Q4 and result Um. The primary pointer names the application node, never
    its body/result_type member.
- id: FR-038-AC-520
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented and qualified on the IR-1396 owner stack; merged-main qualification
    pending (IR-1398). For every CS-06 variant, the reader SHALL refuse ill_typed/operator-ineligible
    at the application: member, binder and result all Um, all compound-unit Cm, or
    all bounded-quantity Bm under exact mode; and sum.quantity over the otherwise
    valid carried member Q. Carried-sum membership consumes IR-1396 carried_member
    enforcement; ordinary sum.quantity cannot accept a carried type_argument even
    when operand and result checks otherwise pass.'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: Resolve bound/member/result and assert the same exact carried node, unit,
    carrier kind and all numeric bounds as values, including decimal scale bounds
    and rational denominator bounds. Admission alone is insufficient.
```

## Coverage

Own production changes: application refusal pointer mapping, binder/result mode pinning before owner carried-member enforcement, ordinary quantity-sum exclusion, catalog BoundIsMember identity comparison and binder type lookup. Five exhaustive test-consumer CatalogUnavailable adaptations preserve failure outcomes. No public API or owner catalog/helper definition was added. Source paths were checked for panic, duplicated rules, stage order, and work growth; new checks use bounded operand access and existing graph lookup.

Reviewer execution: cargo test --workspace --test conformance_qspec carried_sum_cs passed six groups (21 published variants and three precedence controls); log /tmp/ir1398-review/reviewer-cs.log. Existing coder logs for fmt, workspace Clippy, string-edge, two independent regressions, fixture-domain controls and four owner controls were inspected. Root owns the later full gate; no make ci, Kani or ignored backend proof was run. The old-reader log records CS-01 already passing and five adverse groups failing; this is group-level rather than per-variant red evidence.

Scoped validation passed; its sole ac:non-canonical-shape warning is on unchanged line 5946, outside this feature. Changed statements have a named reader, concrete responses and appropriate event/condition grammar.

