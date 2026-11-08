---
id: SR-202611070951309530
title: PR981 paid term fault code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1404; common.rs; v2/read_charge.rs; checked_package_v2_canonical_encoding.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Independent final-head review of PR981 at c2a28cf4 against main5f884607. The private shared-term adapter uses the existing caller-owned ReadStorage and diagnostic reservation contract; it reuses Position::Trail for exact pointer sizing and one-build construction. Checked validate_one, require_array, missing/member-reference faults, the Value bridge, RefCell borrow lifetimes, WorkMeter delegation, and budgeted_outcome paid/pending consumption. The only integration change removes the ignore from one unchanged, genuinely passing original scalar case. No public API, schema, workflow, gate narrowing, copied implementation, or new budget seam is introduced. No applicable AssuranceProfile was found under spec/. No source was changed.

## Verdict

PASS for this three-file paid-term-fault slice: no introduced defects found. This is not whole-corpus AC511 qualification or AC513 proof.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Inspected retained native controls (2 PASS), original integer-range canonical test with all original physical and semantic assertions (1 PASS, 0.27s), final workspace/all-targets Clippy with conformance-cli,test-support and -D warnings (PASS, 25.03s), scoped formatting and diff checks. These are focused evidence, not a repeated aggregate. GitHub workflow42525248886 currently reports Supply-chain job128867330226 FAILURE while Rust remains in progress; the workflow is still running and the failure log/cause is not yet available. This source-review PASS does not claim green hosted CI. No Cargo command was run by this reviewer; the user requires the aggregate after merge and bounded selections only. Selected Quire 0.38.1 computed matrix includes both new native functions under AC510/511 and the now-active original under AC79. These bindings provide partial evidence only; AC511 whole-corpus and AC513 remain UNPROVEN. The initial test-privacy compilation failure was corrected without widening production visibility and its log remains retained.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: common.rs::term_refusal
  path: crates/quire-contract-model/src/checked_package/common.rs
  role: examined
  excerpt: Pay for a term fault before building any of its owned diagnostic fields.
- id: read_charge.rs::refuse_at
  path: crates/quire-contract-model/src/checked_package/v2/read_charge.rs
  role: examined
  excerpt: Pays a bare term refusal's complete payload before building its pointer,
    boxed record or single location.
- id: tc_048_an_integer_spelled_past_the_64_bit_range_is_noncanonical
  path: tests/it/checked_package_v2_canonical_encoding.rs
  role: examined
  excerpt: 'A float that is not whole is canonical bytes: the document reaches the
    body validation it also fails.'
- id: FR-038-AC-510
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
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
  role: context_only
  excerpt: "Whole-read accounting, including each read's owned catalog. A test binary\
    \ whose global allocator counts, on the reading thread during the call, the size\
    \ of each allocation request and the new size of each reallocation request reads,\
    \ under an unlimited budget, an exact budget and each one-below budget of AC-497\
    \ through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance\
    \ packages this repository reads, and the adverse shapes \u2014 the 100000-node\
    \ reference chain, a 10000-node flat graph, a node with 10000 dependencies, a\
    \ 1 MiB string member, 10000 source-map entries, 10000 d"
- id: FR-038-AC-79
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: A package document holding 9007199254740993, -9007199254740993 or 2.0 in
    a node body, or an object with UTF-8-out-of-order member names, refuses noncanonical_wire
    before graph and body checks.
```
