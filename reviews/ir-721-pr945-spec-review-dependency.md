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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The cross-owner seam fixes PlanRoot::admit_authority(self, PlanAuthoritySelection, Option<progress::View>, &mut NativeJoinBudget) and requires the time source kind and period 'on AdmittedPlanAuthority', but the QSL implementation of this seam (quire-spec-language#2489 head 01145fe, monitor_root/progress.rs:357-411) takes two more parameters (validation: PlanAuthorityValidationCaps, locus: NativeJoinLocus) and exposes no time-source accessor (only root()). Its admission_binding() doc also makes the caller account for the cloned handle, while driver FR-019 P5 charges no storage. Either cite QSL as owner of the signature or match it. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:78-92 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | AD-010 P5 says the driver's authority-row phase charges no storage, but driver #129 c7caa55 FR-019 P5 charges one handle storage unit for the cloned admission_binding() before cloning it, as QSL #2489's admission_binding() doc requires. The architecture's phase table disagrees with the phase owner. | spec/assurance/AD-010-tick-monitor-admission-lowering-and-execution.md:136 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | IR's own charges now use MonitorSeed(k) with forms selected_authority / selected_layout_context / selected_clock_binding, but k is 'the selected occurrence's index in clocks()'. The first step-3 charges (ForeignEmission, ClauseMismatch, OccurrenceMismatch inspections) happen before the occurrence is resolved, and an OccurrenceMismatch has no index at all, so the locus FR-361-AC-4 asserts for those charges is still undefined (FR-361.md Charging). |
| FND-002 | fixed | - |
| FND-001 | fixed | - |
| FND-003 | still-open | The two missing parameters were added, but in the wrong order. FR-361.md:79-81 states admit_authority(self, PlanAuthoritySelection, Option<progress::View>, &mut NativeJoinBudget, PlanAuthorityValidationCaps, NativeJoinLocus); QSL #2489 head 01145fe (progress.rs:404-411) is (self, selection, progress, validation: PlanAuthorityValidationCaps, budget: &mut NativeJoinBudget, locus). The text calls the signature QSL-owned yet restates it differently (driver #129 c7caa55 FR-019:89 copies the same wrong order). Swap the two arguments or cite the signature without restating it. checked_time_source() does not exist in #2489 either, but 'QSL fixes the final names' covers that. |
