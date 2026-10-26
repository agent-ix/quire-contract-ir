---
id: SR-567893
title: "IR-874 PR 578 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #578 frozen head 600f2608a924ca9e93ddcc3c78599eb3982178bb against f1ff2218"
review_set: subset
---

## Summary

Reviewed the selected whole-model role admission diff, the reference-model path, refusal ordering and Rust resource bounds against IR FR-038 and QSpec FR-465.

## Verdict

**CONDITIONAL** — the new selection lookup performs unmetered repeated work on caller-controlled lock rows.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Whole-model role lookup scans every model selection without charging work; many roles repeat the scan while only one work unit is charged per role. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:2448 |

## Scope

- **FR-038-AC-26 (examined)**: Each one-over limit other than the byte limit returns `incomplete` carrying the RFC 6901 pointer of the value whose charge failed, which resolves in the document read: nodes at the first node past it, edges at the dependency and occurrences at the source-map entry or region that took the count past it, diagnostics at the first entry past it, and work at the value whose validation took the meter past it; the byte limit carries none.
- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-286 (examined)**: A role admits the FR-465 static and replicated shapes only, including a model of an eligible form, replication maximum at least one, a reference-typed binder and an `until` term exactly for an `until` lifetime; a channel admits only same-clause role endpoints, an eligible message type, explicit delivery and ordering, and a FIFO key with equality-stable type; a queue admits one channel, literal or symbolic capacity, explicit overflow and a loss selector exactly for `loss`. The reader admits the positive variants and refuses the wrong form, missing premise, wrong binder type, non-Boolean lifetim
- **FR-465-AC-9 (examined, QSpec)**: A static role whose `model` is the text `identity` of a `model_selections` row of the lock is admitted as a role on a whole model, with no `model` dependency, whatever the clause's `over` type (a model type, a reference, a scalar, a record or a `Set<M::Cell>`); one clause holding `role w on M` and `role u on N` for two selections is admitted, and the two roles, and a role on a whole model and a role on a model type of the same selection, have different node keys. A text naming no `model_selections` row refuses `missing_declaration`/`missing-selection` at the role node, and so does such a text 

## Coverage

The changed Rust model and public-reader tests were reviewed; no CI, dependency, spec or plan files changed. The selected-text branch preserves typed missing-selection and nonempty-replication refusals at the role node; the reference branch remains in place. Focused tests and formatting/Clippy are reported by the coder; no aggregate gate was run in this review. The broader FR-038 protocol criteria remain Planned.
