---
id: SR-14451
title: "IR-539 required-nullable operation fields: gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-539; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; operations.rs and reader order context"
review_set: subset
---

## Summary

Reviewed IR-539 specification PR 490 against QSpec and the actual reader. Only FR-038 and TC-048 Markdown changed. Rust, manifests, lockfiles, generated code and runtime behavior are unchanged. No applicable AssuranceProfile was found under spec/. No Cargo, formatting, native test or Kani command was run: the code lead owns the single heavy slot. This is a specification-readiness review, not runtime acceptance or whole-repository assurance. No new IDs, AC-294/295 changes, copied fixture, discriminator, alternate identity preimage, runtime shim, new dependency or source catalog was introduced.

## Verdict

**FAIL** — The documented spec is ready, but the planned coding and inherited repository coverage gaps are not accepted runtime behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The newly planned public-reader omission/refreshed-identity/restoration controls are not implemented or asserted by the existing AC-81 binders; IR-539 coding and acceptance remain required before ticket closure. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4721 |
| FND-002 | medium | The repository matrix retains 67 untagged criteria and one criterion tagged only by an ignored test; the representative FR-035-AC-9 is untagged, so this spec-only PR cannot claim a repository coverage PASS. Inherited coverage is owned by IR-878. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:183 |

## Coverage

Computed matrix: 589 criteria; 510 tagged, 67 untagged, 11 method-without-symbol, one tagged-by-ignored-test. AC-81 computes tagged through four pre-existing private-operation unit tests; those assert the older unknown identity, operator class, admission and bare-string branches, not either newly planned public-reader omission branch. AC-83 and AC-84 retain real explicit-null semantic-cause/pointer assertions. Reverse inspection found the owning implementation in operations.rs:204-211 and operation_defect:601-630; no new production code or stub was added. General inherited coverage belongs to IR-878. Plan completion: not assessed. No separate optional repository semantic audit was requested or claimed; the changed criterion and its actual binders were read for the required code/test alignment check.

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
- `FR-035-AC-9` (examined) `spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md`: The typed profile exposes `required_source_selections: Vec<CheckedSelection>` and the IR adapter conforms to QSpec FR-330-AC-10 and TC-935: input refusals use `InvalidInput { node_id }`/`invalid_input`, work stops use existing `Failed`/`Work` with unchanged key/limit/consumed counters, and successful checks continue the existing FR-038 meter without reset. The adapter preserves exact source provenance and emits only `profile.reference`, with no duplicated requirement wire field or new refusal payload.

## Method limits

Only the changed obligation and relevant source/order/test context were substantively examined. Full unrelated requirement semantics were not re-reviewed. Jev criterion-strength was not selected or run; no new dependency relationship or domain object warrants a separate dependency/object review.

## Dispositions

Round 1: the reviewed spec-only source remains unchanged. Both gaps have explicit durable owner handoffs; neither is fixed. The original repository gap verdict remains **FAIL**; the spec-only PR is mergeable with these deferrals, and IR-539 must remain open for coding and runtime acceptance.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Deferred to the same open IR-539 coding/acceptance stage. Verified owner handoff 7032370b-556c-4a34-b50f-a28d733d3fa3 explicitly requires public-reader independent omitted-mode/member refusals, fresh affected node/package identities, exact operation-object refusal, null restoration controls, preserved AC-83/84 semantics and unchanged OperationLeafWire scope. No runtime fix or executed acceptance is claimed by PR 490. |
| FND-002 | deferred | Deferred to existing IR-878 inherited coverage triage. Verified owner handoff 621913a6-f808-4803-a365-c1f864dbb590 preserves the 67 untagged plus one ignored-only criterion debt, representative FR-035-AC-9, and distinguishes this inherited scope from the same-IR539 coding obligation. No whole-repository coverage PASS is claimed. |
