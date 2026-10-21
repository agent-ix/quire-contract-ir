---
id: SR-14452
title: "IR-539 required-nullable operation fields: base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-539; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; operations.rs and reader order context"
review_set: subset
---

## Summary

Reviewed IR-539 specification PR 490 against QSpec and the actual reader. Only FR-038 and TC-048 Markdown changed. Rust, manifests, lockfiles, generated code and runtime behavior are unchanged. No applicable AssuranceProfile was found under spec/. No Cargo, formatting, native test or Kani command was run: the code lead owns the single heavy slot. This is a specification-readiness review, not runtime acceptance or whole-repository assurance. No new IDs, AC-294/295 changes, copied fixture, discriminator, alternate identity preimage, runtime shim, new dependency or source catalog was introduced.

## Verdict

**PASS** — No defect found in this method’s examined spec change; this verdict does not accept unimplemented behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Base checklist applied to the amended operation rule and AC-81/83/84 plus TC-048. Existing IDs and relationships are unchanged; no new reserved criterion allocation, public ABI or authority edge. Omission and null are distinguished, failure code/cause/pointer and first-check boundary are concrete, and the Test procedure is realizable with the existing identity owners. Legacy Verification cells and other criteria remain inherited context, not a claimed whole-repository cleanup. Canonical package/node identity refresh is the existing semantic identity feature, not a new revision-tracking catalog.

## Examined scope

- `FR-038-AC-81` (examined) `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: PLANNED (IR-539 required-presence correction): through `CheckedPackageV2::read`, independently omitting `operation.mode` or `operation.member` from an otherwise valid application refuses `invalid_semantic_graph`, with no cause, at `/semantic_graph/nodes/{i}/body/operation`, before catalog semantic checks; the otherwise identical application with both fields present as `null` admits when its catalog entry requires neither. Re-derive all affected application/node and package identities after each mutation so a stale identity cannot mask the operation shape refusal. Explicit-null required-mode an
- `FR-038-AC-83` (examined) `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: Observed at the operation check, `quire.op.decimal.add`, which catalogues the mode kind `rounding`, with `mode` `null` refuses `invalid_package`/`operation-mode-mismatch` at `operation.mode`; with the mode `{kind: rounding, value: toward-zero}` over a first operand typed at a `bounded_domain` over a `decimal` whose body binds `rounding` to `nearest-even`, reached by a `reference` to the domain and, separately, by a `literal` whose `type` names it, it refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`.
- `FR-038-AC-84` (examined) `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: Observed at the operation check, `quire.op.quantity.convert`, which catalogues the member kind `type_argument`, with its `rounding` mode supplied and `member` `null` refuses `invalid_package`/`operation-member-mismatch` at `operation.member`; `quire.op.record.project` with a `field` member whose `name` is a field the declaring record type does not declare refuses `ill_typed`/`operator-ineligible` at `operation.member.name`; and `quire.op.model.reaches_field` with a `field` member whose `declaration` names no node of the graph refuses `ill_typed`/`operator-ineligible` at `operation.member.decla
- `TC-048` (examined) `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`: Use the repository's owning identity helpers to re-derive the affected
application/node identity, rewrite its references where necessary, preserve
canonical node ordering and refresh the package identity after each mutation.
- `FR-038` (examined) `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`: QSpec requires `mode` and `member` as members of `operation`, each nullable.
When either member is omitted from that closed object, the reader SHALL refuse
`invalid_semantic_graph`, with no cause, at the application's `operation`
object, before the catalog identity, law, mode and member semantic checks.
Explicit `null` is a present wire member: it SHALL continue through those
semantic checks, which admit it where the catalog requires no mode or member
and refuse it with the existing mode or member mismatch where one is required.
This required-presence correction is PLANNED under IR-539; the cu
- `FR-322` (context_only) `/home/peter/dev/quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md`: `mode` is `null` when the entry names no mode kind

## Method limits

Only the changed obligation and relevant source/order/test context were substantively examined. Full unrelated requirement semantics were not re-reviewed. Jev criterion-strength was not selected or run; no new dependency relationship or domain object warrants a separate dependency/object review.
