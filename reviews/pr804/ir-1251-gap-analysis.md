---
id: SR-202611010301
title: "IR-1251 PR 804 model-owner harness gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #804; FR-038-AC-304 through AC-307; tests/it/checked_package_v2_owners.rs; QSpec population vectors"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---

## Summary

Ticket: IR-1251. Compared the changed TC-229 harness with FR-038-AC-304 through AC-307, the computed Test Matrix, the QSpec published positive fixture and 15 population-reference cases, and the reader's selected-document lookup. The matrix reports all four criteria tagged; the change repairs the host-evidence key and keeps the adverse oracles live. Plan completion: not assessed.

## Reviewed units

- **FR-038-AC-304 (examined):** joins population `owner.node` to the selected document's raw population identity and refuses invalid document entries or owners.
- **FR-038-AC-305 (examined):** published population operands are admitted for both model queries with distinct application identities.
- **FR-038-AC-306 (examined):** population references are rejected at ineligible operand and member sites.
- **FR-038-AC-307 (examined):** direct QSpec fixture/vector use, positive admission, and exact adverse code, cause, and locus.
- **QSpec FR-322 / FR-154 (context only):** host document selection by model identity rather than `dependency_selections`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found (placeholder) | - |

## Coverage

- Plan completion: not assessed.
- `quire matrix --scope . --format json` reports FR-038-AC-304, AC-305, AC-306, and AC-307 as tagged. The changed TC-229 tests bind AC-304 through AC-307; an existing model-member test also binds AC-307.
- The published positive fixture and selected domain document are read from `QUIRE_SPECIFICATION_DIR`; no fixture was copied into IR. The positive test now supplies the host document under `acme/orders`, the model root's selected identity.
- The negative controls assert missing host evidence as `missing_import`/`missing-selection` and a foreign document as `invalid_model_binding`/`wrong-model-selection`, both at the root-owner identity locus. The 15 QSpec population cases check admitted and refused outcomes, exact code, cause when published, and locus.
- The source change touches tests only; no new production behavior lacks a requirement. Optional semantic review was skipped; no plan was selected.

## Verdict

**PASS** for the reviewed subset. The changed test evidence reaches the real reader and closes the host-evidence harness gap without weakening the published QSpec adverse checks.
