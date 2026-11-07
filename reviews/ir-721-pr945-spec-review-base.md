---
id: SR-2026110705000810
title: "Base checklist review of quire-contract-ir#945 tick-monitor chain spec"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #945 head cc5034a; spec/checked_package/functional/FR-361, FR-357, FR-358; spec/assurance/AD-010; spec/checked_package/matrix/TC-471, tests.md; spec/model/functional/FR-019; spec/model/matrix/tests.md; spec/spec.md"
review_set: subset
---
# Base checklist review of quire-contract-ir#945

## Summary

Ticket: IR-721. Base checklist over the PR diff: ID formats, AC/TC coverage
(TC-471 lists FR-357-AC-16/17, FR-358-AC-10, FR-361-AC-1..5; FR-361-AC-6 routed
to quire-integration; FR-019-AC-27 to TC-058), link integrity and the
pin/digest antipattern. `quire validate` passes on every changed file (only
pre-existing grammar warnings). No pin, digest, frozen SHA or second PlanId is
introduced; the receipt and context explicitly exclude them. Findings: one
unimplementable criterion (raw wire bytes the package does not retain), a stale
sibling AC, one history-narration line and one underspecified error type.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-361 `package_lock()`/`occurrence()` must return "exact canonical bytes ... as they occur in the admitted wire", borrowed, with no storage and no re-encoding (AC-3: "point into the admitted buffer"). `CheckedPackageV2` retains only the typed `CheckedPackageLockV2` and `Vec<CheckedSourceMapEntry>` (no raw input buffer), so neither value exists to borrow; the coder must either re-encode (forbidden, and then charged) or add raw-wire retention to admission (an FR-038 change nobody specced). | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:92-93,146-148,164; crates/quire-contract-model/src/checked_package/v2/mod.rs:781-795,894-925 |
| FND-002 | medium | FR-357-AC-11 still requires producer rows to reach "an IR-owned row adapter" and a closure read without the authority row, while AD-010 and driver FR-019 put the adapters in QSL and FR-357-AC-17 makes the authority row mandatory; AC-11 as written describes a read that can no longer exist. | spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md:335 |
| FND-003 | low | AD-010 Decisions narrates history ("The driver's private operating ledger is replaced by the shared NativeJoinBudget") instead of stating the decision. | spec/assurance/AD-010-tick-monitor-admission-lowering-and-execution.md:168-170 |
| FND-004 | low | `SelectedMonitorAuthorityRowError` is required to name "the component" but its shape (enum variants vs struct with component field) is not given, yet FR-019-AC-27 makes it a public export. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:61-64 |

## Verdict

Not mergeable: FND-001 is high. ID formats, matrix rows, TC-471 coverage and
link targets are otherwise correct.
