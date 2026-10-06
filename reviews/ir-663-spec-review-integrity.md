---
id: SR-2151
title: "Integrity review of IR-663"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@05afc225eba55736ffc0ffd54255e8a966cdec5f; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Reviewed only the frozen two-file public diff at 05afc225eba55736ffc0ffd54255e8a966cdec5f against 3ed1f7ceeb682745629d2b99fd517d30689e87fb. Private authority checked locally: QSpec #191 in /home/peter/dev/worktrees/qspec-ir658-merged and /tmp/ix-handoff/ir663-refusal-retention-source-plan.md; no private content is reproduced here. Public source: spec/checked_package/functional/FR-038-consume-checked-package-v2.md and spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md.

## Examined public units

### FR-038-AC-167 — examined

PLANNED / UNRUN (IR-661). An unresolved `sourceEnd.type` or `targetEnd.type` refuses `missing_declaration`/`missing-name`; a type naming a relationship or another declaration of the wrong meaning refuses `invalid_model_binding`/`malformed-declaration`, independent of declaration order. A source end naming another owning type, a malformed end or multiplicity refuses `invalid_model_binding`/`malformed-declaration`; `lower > upper` alone refuses `invalid_model_binding`/`unpreserved-model-meaning`. Each declaration refusal retains its typed code, cause and selection-row pointer. The planned public refusal also retains the authentic identity of the actual relationship declaration and its valid origin verbatim under the declaration-refusal retention contract; source coordinates remain supplied, generated origin gains no span, and missing/malformed origin remains absent. Selection-row path and existing graph-node locus meaning stay separate; no metadata is fabricated. All added retention checks remain PLANNED / UNRUN. 

### FR-038-AC-173 — examined

PLANNED / UNRUN (IR-661). Removing direction, category, composite or origin, setting each to null or a wrong type, using an unsupported direction/category value, or supplying a malformed common-schema origin branch refuses `invalid_model_binding`/`malformed-declaration` at the selection row before any application resolution, retaining code, cause and selection-row pointer under AC-167's existing refusal contract. The planned refusal retains any authentic declaration identity; a missing or malformed origin retains no origin, including both-branch and partial-branch cases. Schema-valid source and generated origins admit without a default direction; a later declaration refusal retains their valid origin exactly, with no span invented for generated origin. Retention remains PLANNED / UNRUN and does not change the existing code, cause, selection-row pointer or check order. 

### FR-038-AC-174 — examined

PLANNED / UNRUN (refusal origin retention). Every located model-declaration refusal retains its authentic IR node identity and valid full typed Source or Generated origin through selected-document release and the public CheckedPackageRefusal boundary. A nested relationship retains its own metadata, not its owner's. Source coordinates and independently optional end members remain exact, including the existing admitted 2^53 boundary; generated input identity order/multiplicity and version text remain exact with no invented span. Missing or malformed origin retains none, never a salvaged branch. Existing admission/refusal codes, causes, row/member order and bounded accounting remain unchanged.

### FR-038-AC-175 — examined

PLANNED / UNRUN (refusal propagation). Every refusal constructor and public reader/dispatch/typed handoff consumer explicitly preserves available declaration_identity/declaration_origin or authentic absence. A located declaration identity may not become an empty/default/inferred URI, a graph digest or an enclosing-owner identity. Pre-node byte/admission failures retain absence and existing code/path/cause/locus/contract_version/document_pointer semantics. Dropping either field, normalizing an origin, inventing a generated span or reclassifying an earlier failure must fail an independent constructor/consumer oracle; no compatibility layer or diagnostic-prose parsing supplies retention.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS: the two new criteria and the amended existing criteria agree on authentic declaration identity, independently optional source ends, ordered generated inputs, absence for invalid origin, and preserved refusal order.

## Validation

`quire validate --scope . .review-scratchpad/ir-663-spec-review-integrity.md` — PASS (exit 0); emitted module duplicate-archetype/inverse-edge and inline-data-schema warnings, with no document validation error.
