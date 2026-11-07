---
id: SR-2026110705000811
title: "Failure-domain review of quire-contract-ir#945 tick-monitor chain spec"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; PR #945 head cc5034a; spec/checked_package/functional/FR-361, FR-357, FR-358; spec/assurance/AD-010; spec/checked_package/matrix/TC-471, tests.md; spec/model/functional/FR-019; spec/model/matrix/tests.md; spec/spec.md"
review_set: subset
---
# Failure-domain review of quire-contract-ir#945

## Summary

Ticket: IR-721. Examined identity confusion and foreign-authority admission in
FR-361 (authority row, ForeignEmission/Clause/Occurrence/TimeSource checks,
clock-binding encoding) against QSL FR-251 (AC-45..48), QSpec FR-252, the IR
admitted clock/time-source facts, driver #129 FR-019 and RT interface-012 on
main. Two identity-confusion defects: the clock-binding value drops FR-252 key
components, and ForeignEmission cannot detect a handoff from another emission.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The canonical clock-binding member set diverges from QSpec FR-252: FR-252's model-time `time_source` is "the model's qualified name and its time source kind, with the tick operation for a digital source" and `period` is a separate key component, but FR-361 encodes `time_source` as only `kind` (+ nested `period`) and omits model and tick operation, although IR admits both (timed-model source bindings kind/unit/model/operation/period). Two monitors over different timed models or tick operations with equal kind/unit/period get byte-identical `clock_binding`, which RT compares byte-for-byte (interface-012: "exactly the components admitted by the selected QSpec FR-252 profile"), so a foreign clock's input context can match. Contradicts the lead decision "FR-252 members". | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:99-110; crates/quire-contract-model/src/checked_package/v2/timed_model.rs:345-358 |
| FND-002 | high | ForeignEmission is vacuous against the failure it is claimed to catch. The row's `admission` is the consumer's own strict-read capability, and driver FR-019 P4 builds it from `package.admission_binding()` — the same package whose plan the closure reads — so `retains` always passes; the order/association bytes carry no emission binding IR can check. A handoff taken from a second emission of the same source, paired with the first emission's package, binds successfully. FR-361-AC-6 and driver FR-019-AC-4 ("second emission refuses ForeignEmission") are therefore unachievable by the specified mechanism. The capability must originate in the QSL handoff (QSL's strict read joined against) and FR-361 must say so, or the claim must be dropped. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:54,124-126,167 |

## Verdict

Not mergeable: two high identity-confusion defects. The ClauseMismatch,
OccurrenceMismatch and TimeSourceMismatch checks, their order and loci, and
the exclusion of names/indices/equal contents are otherwise sound.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | Clock-binding `model` has two sources: the member table reads it from 'the timed-model root's model reference', while the prose says kind, model, operation and period are read from the time-source node's bindings 'and for tick its model, operation and period bindings'. A tick source node carries its own `model` binding (timed_model.rs:352), so two implementers can pick different nodes and emit different bytes RT compares exactly. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:130,136-139 |
| FND-004 | low | ForeignEmission's second clause ('the plan's root, four entry and clause node IDs do not resolve to the same structural keys in that admitted package') cannot fail once retains() passes, because a retaining package shares the plan's Arcs; no AC-2 case exercises it. State it as an invariant or drop it. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:170-173 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
