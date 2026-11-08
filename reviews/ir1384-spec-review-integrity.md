---
id: SR-202611070951309520
title: IR-1384 spec-review-integrity review
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; IR-1384; changed model authority, fixture tests,
  standalone conformance lane, TC-471 and matrix status
review_set: subset
---

## Summary

Ticket: IR-1384. Frozen PR #975 source reviewed read-only, including dependency refresh and fixture conversion. No applicable AssuranceProfile exists under spec/. No source or build cache was modified. Production authority rows keep private fields, borrow byte values unchanged, and add no QSL/observation/runtime dependency. No decode/re-encode authority bridge or second plan identity was found.

TC-471 retains all eight planned procedures and expected results; the changed qualification paragraph distinguishes local receipt handoff from final integration and layout qualification. No requirement statement, domain object or dependency edge changed. Extension failure, identity, purity and topology probes found no additional defect in the edited qualification prose.

## Verdict

**PASS** — No defect found in edited specification qualification text; this does not assert implementation readiness.

Full FR-361 acceptance is not claimed: layout-context AC-3 and non-authority AC-4 remain separately owned; full two-occurrence AC-1, refusal permutations AC-2 and per-charge AC-4 are partial. AC-6 final quire-integration qualification remains with that owner. Published QSL fails compilation with CheckedJson/Value and Dependency exhaustiveness errors (compile2.log); genuine receipt tests are unrun. Workspace check and one strict fixture test pass in existing logs; focused tests and Clippy remain queued. No aggregate/Kani gate run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage



## Examined scope

```yaml
scope:
- id: FR-361-AC-1
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: An IR-owned strict admitted Timed monitor package with two claim occurrences
    of one clause and a row naming the second occurrence returns a closure whose receipt
    holds that occurrence, its own clock read and the row's exact byte slices; a row
    naming the first returns the first. Dense and tick sources with equal and different
    exact periods give the specified success or `TimeSourceMismatch`.
- id: FR-361-AC-2
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: Each of an independent strict read of equal package bytes, another package,
    a different clause, an absent ordinal, a configured clock, a dense row against
    a tick source and a different exact period returns its named `SelectedMonitorAuthorityErrorCode`
    at the stated locus, with charged work and no closure. Paired defects show the
    stated authority check order; a model-row defect beside an authority defect reports
    the model row, and an authority defect beside a source-row or equality defect
    reports the authority. `try_new` returns `SelectedMonitorAuthorityRowError` with
    component `Occurrence`, `O
- id: FR-361-AC-3
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: The root, entry and clause keys equal the plan's resolved structural keys.
    `package_lock()` and `occurrence()` are byte-equal to the corresponding member
    bytes of the admitted input. `profile()` and `clock_binding()` for dense and tick
    sources equal independently written expected RFC 8785 values with exactly the
    specified members; changing the model, tick operation, unit, period, source kind
    or profile each changes `clock_binding()`.
- id: FR-361-AC-4
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: 'The read succeeds at its exact measured work and storage allowance. One
    unit below the cumulative charge before each authority, layout-context and clock-binding
    charge refuses at that charge with its kind, amount and exact locus: `MonitorAuthority`
    for the admission, clause and occurrence inspections (including at an `OccurrenceMismatch`),
    and `MonitorSeed(k)` with the tabled form after resolution; no partial receipt
    escapes. The same read on a package clone yields equal values and charges.'
- id: FR-361-AC-5
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: API and dependency inspection shows `SelectedMonitorClosureRows::authority`
    is a required field, the row has private fields and one fallible constructor and
    borrows its byte values, the model crate gains no QSL, observation or runtime
    dependency, no byte value is decoded or re-encoded, no raw input buffer is retained,
    and no `PlanId`, digest or second identity appears.
- id: FR-361-AC-6
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: In `quire-integration`, a genuine `AdmittedPlanAuthority` row binds through
    the closure read on its own `admitted_package()`; the receipt bytes equal QSL's
    order value and the owner's association value. The same row against the package
    of a second admission of the same plan root's bytes refuses `ForeignEmission`.
    Depends on the QSL seam above.
```

## Method evidence

Edited qualification prose reviewed against unchanged FR-361/FR-357/FR-358; no changed normative statement, relationship or domain-object shape.

## Final disposition evidence (round 1)

Final disposition round 1 examines the frozen source at 14d0804dbec41811cc60f1cadf7a38766242742d. Both workspace and standalone conformance formatting passed (fix-format.log exit 0). Workspace locked all-targets check with conformance-cli, 31 monitor-focused tests (31 passed, zero failed/ignored), and workspace all-targets conformance-cli Clippy with warnings denied passed (fix-focused.log exit 0). Static final diff inspection found formatting changes only after the prior source fix. No native build was duplicated. This evidence supersedes earlier queued-check statements in this artifact. The changed IR receipt scope is review-clear and suitable for the user merge request; no aggregate or whole-ecosystem premerge gate is required by this disposition. Genuine QSL lane qualification remains externally blocked in the published QSL compiler, owned by draco/terra; it is not an IR production compilation failure and no QSL workaround or action was taken. No full FR-361, layout-context AC-3/non-authority AC-4, complete AC-1/2/per-charge AC-4, or final quire-integration AC-6 qualification is claimed.
