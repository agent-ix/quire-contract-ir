---
id: SR-2157
title: "EARS conformance review of IR-663"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@05afc225eba55736ffc0ffd54255e8a966cdec5f; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

The edited FR-038 requirement statements and criteria were checked with a targeted Quire grammar run and semantic reading. Quire reported 1/1 document grammar-clean and zero grammar findings.

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

PASS: the new obligation names the reader, trigger and concrete retention response. Other edited statements define observable preservation and absence rules; no EARS defect changes implementation meaning.

## Skipped methods

Rust review: no Rust or Cargo diff. React review: no TSX/JSX or React change. Gap analysis: no production-code change. Object review: no domain-object edit. Risk-complexity, spec-security and architecture evaluation: this metadata requirement adds no corresponding new analysis surface. Criterion-strength: not selected and would require an unauthorized external model. Existing base, integrity, failure-domain, evidence, dependency and scope-boundary lenses are recorded in SR-2150..2155.

## Validation

`quire validate --scope . .review-scratchpad/ir-663-spec-review-ears-conformance.md` — PASS (exit 0); module warnings only, no document validation error.
