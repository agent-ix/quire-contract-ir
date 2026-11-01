---
id: SR-202610310142
title: "IR-1236 code-review scoped review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1236 PR 786 changed model_operations.rs, model_members.rs, public reexports, operation_census.rs, FR-355, TC-464, FR-019 inventory"
review_set: subset
---

## Summary

Reviewed borrowed declaration views, authentic selected owner joins, all resolution/refusal paths, binder/result projection, public exports and persistence retention. No concrete defect found. Existing effective-operation names retains its previous resolution behavior through the shared candidate helper.

## Verdict

PASS — no scoped findings. Review does not authorize merge; full IR integration gate remains pending with the lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Scoped independent review of the frozen candidate against the supplied base. No source changes or Cargo/backend proof runs performed, as required by the brief. Existing evidence inspected: eight default census tests pass, explicit live QSL producer test passes, TC-058 and TC-456 pass, model-member regressions pass; formatter and workspace all-target Clippy logs supplied. Full integration and cargo-deny were not independently executed. No applicable AssuranceProfile was found under spec/. Optional gap semantic fan-out was not requested and was skipped; code-test agreement was assessed under code-review. No mirror OperationDecl, copied parser, new dependency, body inference or stage-identity inference added.

## Examined units

- FR-355-AC-1 — role: examined; path: spec/checked_package/functional/FR-355-read-complete-model-operation-census.md; excerpt: A package admitted through the public reader with two declared operations and no operation anchors or bodies returns both identities and an authoritative empty census for a different empty context. Adding or removing an unreferenced declaration changes the census independently of bodies.
- FR-355-AC-2 — role: examined; path: spec/checked_package/functional/FR-355-read-complete-model-operation-census.md; excerpt: Parameters retain producer name/order and exact Boolean, bounded-integer and collection member domains; a separately present result retains its domain. Absent results and present unavailable domains remain distinguishable, without omitting operations. A missing retained parameter name refuses explicitly instead of inventing a binder.
- FR-355-AC-3 — role: examined; path: spec/checked_package/functional/FR-355-read-complete-model-operation-census.md; excerpt: Inheritance, a diamond and a most-derived redefinition retain each effective operation once with its original member and actual declaring node. An ambiguous effective name, absent declaring node or duplicate declaring-node join returns its distinct typed refusal and no partial census.
- FR-355-AC-4 — role: examined; path: spec/checked_package/functional/FR-355-read-complete-model-operation-census.md; excerpt: Foreign, unknown and wrong-kind contexts have distinct typed refusal codes. A sufficient work provision returns the full table and consumed work; one below the required provision refuses with that configured limit, and a zero provision refuses before census exposure.
- FR-355-AC-5 — role: examined; path: spec/checked_package/functional/FR-355-read-complete-model-operation-census.md; excerpt: A downstream public consumer can read the census and existing callable/dispatch APIs separately through the model crate facade. The public inventory names the new read types, which borrow retained declaration facts and expose canonical member-domain types without a copied parser, mirror operation model or added dependency.
- FR-355 — role: examined; path: spec/checked_package/functional/FR-355-read-complete-model-operation-census.md; excerpt: When a consumer requests the effective operations of an admitted model context,
Contract IR SHALL return the complete selected declaration census or a typed
refusal under the caller's work provision. The census SHALL include operations
without an executable body, source clause, operation anchor or captured call.
It SHALL read the retained selected model declarations and use FR-322's effective
member and most-derived-redefiner rules, without inferring operations from bodies.
- TC-464 — role: examined; path: spec/checked_package/matrix/TC-464-model-operation-census.md; excerpt: Exercise the admitted model operation census independently of executable bodies, operation anchors and dispatch sidecars.
- FR-019 — role: examined; path: spec/model/functional/FR-019-rust-library-interface.md; excerpt: types `CheckedModelOperation`, `CheckedModelOperations`, `CheckedOperationParameter`, `CheckedOperationReadError`, `CheckedOperationReadErrorCode`; associated model_operations read


## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | Live QSL environment-dependent control was placed in ordinary integration tests despite make test including ignored tests; missing producer input panicked and failed CI. | tests/it/checked_package_v2_model_fields/operation_census.rs:430 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Moved the unchanged control into the explicitly registered conformance_qsl_ir1236 target with test = false; removed it from it and removed ignore, preserving required-input failure in the explicit lane. |

## Disposition pass 1 evidence

Narrow correction review only: Cargo.toml, moved control, and TC-464 producer-lane procedure. Original Findings and original review comment are preserved. Makefile uses --all-targets (including --tests); Cargo documents --tests as selecting test=true targets, so this test=false integration target is excluded from ordinary tests and remains selectable by its explicit --test name. See https://doc.rust-lang.org/cargo/commands/cargo-test.html#target-selection. The new function body equals the original control, with local imports/constants replacing parent-module imports and no ignore or missing-environment success. No production or semantic acceptance criterion changes. Explicit new-target execution and full CI remain pending with the lead; no Cargo or source edits were performed by this reviewer. Final scoped disposition: PASS with FND-002 fixed by inspection.
