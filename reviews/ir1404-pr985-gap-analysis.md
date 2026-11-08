---
id: SR-202611070951309537
title: PR985 empty-name diagnostic gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-1404; v2/mod.rs header refusal
review_set: subset
---

## Summary

Ticket: IR-1404. Scoped changed-code reverse traceability, computed-matrix and evidence audit for PR985. The header and diagnostic reservation behavior is owned by FR038; the native physical counter exercises the production path and the unchanged original envelope remains a genuine end-to-end oracle. No requirement or matrix was authored. Optional whole-repository semantic audit was not requested and is skipped; no global assurance-completion claim is made.

## Verdict

PASS for this header-family fix: no introduced findings. AC511 whole-corpus and AC513 proof remain UNPROVEN.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Inspected retained selected 1.98.1 workspace native control (1 PASS; compile 21.90s, test 0.01s), unchanged exact nonignored original envelope test (1 PASS; compile 14.22s, test 0.12s), workspace all-target Clippy with conformance-cli,test-support and -D warnings (PASS, 16.32s), scoped formatting and diff checks. The native control executes the real decoded all-families wire and real caller budget; it verifies final code/cause/path/no node locus, exact physical bytes, no work change, zero/one-short no-allocation denials with pending MalformedWire and PackageRead locus, repeated cumulative charging, and version-before-name priority. The original envelope retains all physical and semantic assertions. No Cargo or aggregate was run by this reviewer: the user requires bounded checks and aggregate after merge. Selected Quire 0.38.1 computed matrix binds the new native control to AC510/511; this is partial header-family evidence, not whole-read AC511 qualification. Existing global gaps and AC513 remain unproven. Hosted Supply-chain job is RED and Rust job pending; no hosted-green claim is made. No applicable AssuranceProfile was found under spec/. No spec, plan, public API, dependency, workflow or gate changed.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: check_package_header
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: examined
  excerpt: Checks the contract version and nonempty package name.
- id: validate
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: examined
  excerpt: check_package_header(wire, &mut WorkMeter::over(budget))?;
- id: header_fault_storage_tests::empty_name_diagnostic_is_paid_before_allocation_and_keeps_header_order
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: examined
  excerpt: fn empty_name_diagnostic_is_paid_before_allocation_and_keeps_header_order()
    {
- id: FR-038-AC-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Each positive V2 package fixture this repository builds from its own public
    vocabulary admits; an unknown, empty, absent or malformed `contract_version` refuses
    as `unknown_contract_version` or `malformed_wire` before any version-specific
    decoding; and the strict parse (duplicate-member, noncanonical) refusals occur
    before a version is selected.
- id: FR-038-AC-510
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: On a budgeted entry, a work denial whose diagnostic is affordable returns
    `BudgetIncomplete` with `denied` `Work` and its amount, the budget's configured
    work limit, cumulative work plus the denied amount, and the value's pointer; a
    read-storage denial likewise returns `BudgetIncomplete` with `denied` `ReadStorage`.
    When the diagnostic's own request is denied, or the denied request was a refusal
    diagnostic, the read returns `Exhausted` holding the original denied resource
    and amount, its configured bound (including zero), its `PackageRead` locus and
    the pending outcome class (refusal code or s
- id: FR-038-AC-511
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Whole-read accounting, including each read's owned catalog. A test binary\
    \ whose global allocator counts, on the reading thread during the call, the size\
    \ of each allocation request and the new size of each reallocation request reads,\
    \ under an unlimited budget, an exact budget and each one-below budget of AC-497\
    \ through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance\
    \ packages this repository reads, and the adverse shapes \u2014 the 100000-node\
    \ reference chain, a 10000-node flat graph, a node with 10000 dependencies, a\
    \ 1 MiB string member, 10000 source-map entries, 10000 d"
```
