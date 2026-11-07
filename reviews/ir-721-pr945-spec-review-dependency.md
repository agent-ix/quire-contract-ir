---
id: SR-2026110705000813
title: "Dependency review of quire-contract-ir#945 tick-monitor chain spec"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; PR #945 head cc5034a; spec/checked_package/functional/FR-361, FR-357, FR-358; spec/assurance/AD-010; spec/checked_package/matrix/TC-471, tests.md; spec/model/functional/FR-019; spec/model/matrix/tests.md; spec/spec.md"
review_set: subset
---
# Dependency review of quire-contract-ir#945

## Summary

Ticket: IR-721. Checked new relationship edges (FR-361 depends_on FR-357/358/038
and QSL FR-251; references QSpec FR-252, observation FR-004, RT FR-276, driver
FR-019; AD-010 references) and the admitted facts each new read consumes. Edges
resolve to existing owners and the model crate gains no QSL/observation/RT
dependency. Two consumed-fact sources are not named.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-361-AC-4 requires authority/clock-binding refusals to carry "the original ... locus", but no locus source is given for IR's own charges inside the closure read: FR-358 reserves `AuthorityRow` etc. for external charges and says they "do not change any native read's own loci", and no native variant (e.g. `MonitorSeed`) is assigned. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:141-148,165; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:63-72 |
| FND-002 | low | TimeSourceMismatch compares the row's kind and period with the "admitted time-source kind" and period, but `CheckedMonitorClock::ModelTime` exposes only node ids; the fact to read (the timed-model source node's `kind` and `period` bindings) is not named. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:131-133; crates/quire-contract-model/src/checked_package/v2/monitor_plan.rs:221-236 |

## Verdict

Edges are correct; FND-001 must be fixed before coding, FND-002 is a nit.
