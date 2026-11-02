---
id: SR-202611010319
title: "Rust code review of IR-1226 operation-effect reader PR 812"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{intake.rs,mod.rs,model_members.rs,operation_effects.rs,runtime_type_projection.rs}; crates/quire-contract-model/src/lib.rs; tests/conformance_qspec/operation_effects.rs; tests/it/checked_package_v2_operation_effects.rs; tests/it/checked_package_v2_model_fields/operation_census.rs; accompanying construction-site test updates"
review_set: subset
---

## Summary

Ticket: IR-1226. Reviewed frozen PR #812 head `5bc5ee187623c2bd19b6fb53e50d9876b3dd1a6b` against IR main `924bf3298ee914bbd45454049998411ab53bd742` and QSpec FR-322-AC-303/TC-977. Applied repository AGENTS.md/CLAUDE.md and Rust review/style rules to the full changed code and test diff. No Cargo, workflow, lint-rule or dependency-file changes are present. The coder's focused checks are reported, not independently repeated while the shared build lock runs the planner's main-head verification.

## Scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-406 | examined | The strict reader admits OEC-01..07 by reference; a present row exposes exact producer entries and provenance through a borrowed typed read, while absence is unavailable rather than empty. |
| FR-038-AC-407 | examined | The strict reader applies OES-01..14 and OER-01..26 to complete packages, checking published code, cause and primary locus; null, empty or non-array carrier is malformed wire. |
| FR-038-AC-408 | examined | Each visited carrier row and effect entry consumes one validation visit before inspection; one below the required work returns incomplete at the first uncharged locus. |
| FR-019-AC-21 | examined | A default-feature producer constructs typed operation effect rows for the optional borrowed writer input, and an external consumer receives borrowed admitted rows only after strict admission. |
| FR-354-AC-10 | examined | Ledger pay and payTaxed are exposed in own declaration order with admitted parameters, result and producer effect; inherited visibility does not add an own operation. |
| FR-354-AC-11 | context_only | Missing or conflicting operation/effect facts and unrepresentable signatures refuse without a partial view, with bounded reads and clone equality. |
| FR-322-AC-303 | examined | The optional operation_effects carrier has exact selected-document frame lift, closed effect/provenance, canonical key order and complete coverage. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Effect-entry shape scans run before their validation-work charge, so a tight limit may return a schema refusal after inspecting uncharged entries instead of incomplete at the first entry. | crates/quire-contract-model/src/checked_package/v2/operation_effects.rs:542 |
| FND-002 | medium | The unknown-member remap matches an unrelated top-level key such as `operation_effects_extra` and changes its `unknown_member` refusal to `malformed_wire`. | crates/quire-contract-model/src/checked_package/v2/intake.rs:187 |
| FND-003 | medium | The TC-977 harness omits the cause assertion when a vector expects `null`, so an extra cause on an invalid-semantic-graph refusal passes despite the claimed exact cause check. | tests/conformance_qspec/operation_effects.rs:274 |

## Verdict

CHANGES REQUESTED. The typed carrier, source-frame comparison and borrowed view follow the intended admission boundary on the examined positive paths. Fix the three concrete boundary/test defects in the original PR, then recheck the same findings on its updated head. The separate gap-analysis artifact records criterion-to-test gaps.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The row and every effect entry now charge before shape inspection; the later semantic pass no longer charges a second time. |
| FND-002 | fixed | Only the exact carrier path and descendants remap to malformed_wire; operation_effects_extra remains unknown_member in a regression test. |
| FND-003 | fixed | The TC-977 harness compares Option cause to the vector expectation unconditionally, including null. |

## Disposition verdict

Round 1 at `8503368b7f2a15b8bc168530a67353bbd4bb1904`: all recorded findings fixed. Focused reader, TC-058, TC-977, string-edge and formatting checks passed according to the coder; the aggregate gate remains with the lead.
