---
id: SR-202611070951309506
title: IR-1388 spec-review/ears-conformance review
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir; IR-1388; direct model enum projection, fixtures,
  FR-352 statement and AC-30
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-352
  type: reviews
---

## Summary

Independent review of frozen PR #962. Ticket: IR-1388.

## Verdict

PASS

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Coverage

- FR-352-AC-30 (examined), spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md: Direct projection of an admitted selected `model/variant_type` with its complete installed model enum shape returns `ValueType::Enum` equal to that shape, retaining ordering and every opaque variant identity. A package clone agrees. The reached-model join and following projection share one `NativeJoinBudget`: their measured exact cumulative work and storage succeed, and one unit below either refuses at the first unpaid kind and locus without a partial projection. An absent shape returns `UnavailableIdentityWitness`, and identities installed for an independent admission refuse `ForeignNode`. A
- FR-352 (examined), spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md: A selected `model/variant_type` whose complete shape was joined by the reached
model read SHALL map to `Enum` with that installed opaque `EnumShape`, retaining
its ordering and every producer-issued variant identity. The projection SHALL
check the witness's admission binding and borrow the installed shape, charging
its inspected case bytes and owned result before copying it. An absent installed
shape SHALL return `UnavailableIdentityWitness`; no declaration key, case name,
source-enum row or fallback shape SHALL substitute for it.

## Evidence

Production branch borrows the private installed shape, checks admission binding, preflights inspected case bytes, charges owned Enum storage, then clones. Existing native storage policy counts shared opaque NodeKeys as handles. No dependency or gate change. Genuine fixture preserves producer component identities; authored refusal fixture is explicitly separate. Both exact-filtered workspace library tests independently passed. Existing coder logs show genuine pre-fix UnsupportedExactType and post-fix pass, workspace library/tests Clippy with conformance-cli pass, and formatter pass. Final aggregate gate remains lead-owned. Fixture/provenance additions are expressly required by the owner brief; no new implementation or QSL dependency is copied. No applicable AssuranceProfile exists. Scoped deterministic engine reports 1/1 docs grammar-clean and zero grammar findings. New conditional model projection statement has concrete outputs, typed refusals, and budget obligations.

## Disposition pass 1 evidence

The fix changes only the AC-30 table separator and adds the six original review artifacts unchanged. Production code and test fixtures match the reviewed candidate. The original findings and historical evidence above remain unchanged. Scoped FR-352 validation passes with 30 criteria. The independently recomputed matrix recognizes AC-30 and both genuine-Tier and authored-owner test binders. No new defects found. No Cargo or aggregate gate was run for this prose-only fix; final aggregate verification remains lead-owned.
