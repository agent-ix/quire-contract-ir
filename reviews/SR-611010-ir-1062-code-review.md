---
id: SR-611010
title: "Code and Rust review of IR-1062 PR 616"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{model_members,model_fields,derived_keys,frame,operations,state}.rs; tests/it/checked_package_v2_{model_fields,model_members,owners,refinement}.rs"
review_set: subset
---

## Summary

Reviewed PR #616 at 22b74cb4 against FR-068-AC-1..5, QSpec FR-322-AC-63/65/72 and FR-208-AC-5/6/7. The new owner-key mappings authenticate the selected meaning, but intake admits incomplete process, state-machine and persistence-interface declarations as valid selected owners.

## Verdict

**FAIL** — the new checked owner path accepts declarations that QSpec requires the selected-document intake to refuse. Rust code, tests and relevant callers were reviewed statically. Cargo gates were deferred under the host backoff instruction.

## Coverage

The diff adds no copied QSpec artifact, compatibility layer, duplicate evaluator or CI gate change. Examined process and persistence owner mapping, reference derivation, Text record typing, member resolution, wrong-kind clause edges and typed refusal tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Selected owner intake authenticates process, state-machine and persistence-interface keys without validating those meanings' required declarations. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:2790 |

## Detail

FND-001: `read_semantic_ir` adds every persistence interface to `model.persistence_interfaces` on meaning alone and skips its body in the second pass. It parses process and state-machine `fields`/`operations` but never checks the required process `identityFields`/`steps` or state-machine `states`/`transitions`/operations. `selected_form_document` supplies a `Flow` without identity fields or steps, a `Machine` with no states, transitions or operations, and a `Store` with no operations or `persists`; the positive reader test expects admission. QSpec FR-208-AC-5/6/7 and FR-154 require typed refusal at selected-document intake. The newly admitted model nodes therefore give false checked evidence for these malformed declarations. Validate the required meaning at intake through the authoritative source before adding its owner key, and use genuinely valid selected documents for the controls.

## Dispositions

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | `check_selected_meaning` validates required Process, StateMachine and PersistenceInterface members, references and forbidden contracts before owner admission. `selected_form_document` now declares valid Flow, Machine and Store controls; `selected_process_machine_and_persistence_require_complete_meaning_declarations` proves seven missing-member refusals at the selection stage. |
