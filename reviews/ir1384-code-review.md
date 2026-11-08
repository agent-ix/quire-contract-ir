---
id: SR-202611070951309517
title: IR-1384 code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1384; changed model authority, fixture tests,
  standalone conformance lane, TC-471 and matrix status
review_set: subset
---

## Summary

Ticket: IR-1384. Frozen PR #975 source reviewed read-only, including dependency refresh and fixture conversion. No applicable AssuranceProfile exists under spec/. No source or build cache was modified. Production authority rows keep private fields, borrow byte values unchanged, and add no QSL/observation/runtime dependency. No decode/re-encode authority bridge or second plan identity was found.



## Verdict

**CONDITIONAL** — Actual code findings require correction; qualification remains blocked.

Full FR-361 acceptance is not claimed: layout-context AC-3 and non-authority AC-4 remain separately owned; full two-occurrence AC-1, refusal permutations AC-2 and per-charge AC-4 are partial. AC-6 final quire-integration qualification remains with that owner. Published QSL fails compilation with CheckedJson/Value and Dependency exhaustiveness errors (compile2.log); genuine receipt tests are unrun. Workspace check and one strict fixture test pass in existing logs; focused tests and Clippy remain queued. No aggregate/Kani gate run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Clause identity comparison is charged as one work unit before comparing caller-controlled package/path text and nested identity components; a large equal-prefix clause therefore performs work outside FR-358 byte/element accounting. Admit the structural comparison footprint before inspection. | crates/quire-contract-model/src/checked_package/v2/monitor_plan/authority.rs:239-240 |
| FND-002 | medium | same_period scans admitted period text with split_once before reserving its width-based work. An exhausted comparison allowance still scans the entire decimal numerator before refusing; reserve scan work using lengths before locating the separator. | crates/quire-contract-model/src/checked_package/v2/monitor_plan/authority.rs:183-195 |
| FND-003 | medium | Every semantic authority refusal unconditionally clones its structural node and allocates a Box without admitting diagnostic storage. A budget exhausted by preceding model/clock reads can therefore allocate an owned diagnostic despite having no remaining storage. Reserve the complete diagnostic floor and return a compact typed refusal when unaffordable. | crates/quire-contract-model/src/checked_package/v2/monitor_plan/authority.rs:162-174 |
| FND-004 | low | The standalone conformance crate fails cargo fmt --manifest-path tests/conformance_qsl_ir1384/Cargo.toml --all --check. Workspace fmt excludes this crate, so format its new receipt test with the repository rustfmt rules. | tests/conformance_qsl_ir1384/tests/receipt.rs:186-187,361-374 |

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
- id: FR-358-AC-3
  path: spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md
  role: examined
  excerpt: Every IR-owned row, key, handle, container and map/set entry retained or
    cloned by the budgeted path, and each byte of copied dynamic payload, is charged
    by the defined logical-storage rule before the corresponding reserve, clone or
    insert. On a budget denial the complete owned diagnostic floor is charged before
    any owned diagnostic allocation; at zero storage the refusal allocates no owned
    diagnostic. The tests make a prior nonempty map require an `Arc::make_mut` copy
    and show that a limit one below that copy's charge refuses before the new row
    appears. A borrowed inspection incurs no storage
```

## Method evidence

Validation: workspace cargo fmt --all --check passed. Standalone fmt failed as recorded in FND-004. Clippy, cargo deny and full tests were not duplicated while the global job queue is active; no aggregate gate was run. Existing compilation and fixture logs are evidence only for their stated lanes. Rust API, implementation, safety, wire and testing references and repository AGENTS/CLAUDE/lints were read. Static dependency/byte-flow inspection passed the FR-361-AC-5 slice.

## Test bindings

Correct trace bindings support partial slices only; they do not establish complete AC coverage.

```yaml
bindings:
- test_id: genuine_qsl_authority_binds_dense_and_tick_emissions
  ac_id: FR-361-AC-1
  trace: correct
- test_id: genuine_qsl_authority_binds_dense_and_tick_emissions
  ac_id: FR-361-AC-4
  trace: correct
- test_id: genuine_qsl_authority_binds_dense_and_tick_emissions
  ac_id: FR-361-AC-6
  trace: correct
- test_id: mismatched_emissions_refuse_with_typed_authority_errors
  ac_id: FR-361-AC-2
  trace: correct
- test_id: mismatched_emissions_refuse_with_typed_authority_errors
  ac_id: FR-361-AC-6
  trace: correct
- test_id: exact_period_comparison_rejects_invalid_denominators_and_text
  ac_id: FR-361-AC-1
  trace: correct
- test_id: exact_period_comparison_reserves_work_and_scratch_at_the_authority_locus
  ac_id: FR-361-AC-4
  trace: correct
```
