---
id: SR-202611070951309523
title: "PR979 selected compiler metadata gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1404; tools/read_path_proof/compiler_metadata.rs; tools/read_path_proof/test_compiler_metadata.py"
review_set: subset
---

## Summary

Ticket: IR-1404. Independent review of frozen PR979's two new tool files. Computed matrix saved as matrix.json using quire 0.38.1. FR-038-AC-513 is Inspection/method-without-symbol, with existing binders; this is not production proof credit. New Python controls are scoped feasibility evidence and are not listed as new AC-513 binders by the matrix. Manual source/test examination finds no introduced hollow implementation or false completion assertion. This review is confined to the two-file primitive, not a repository assurance PASS. Plan completion: not assessed. Optional repository-wide semantic review not run.

## Verdict

PASS for the changed feasibility primitive: no introduced defects found. AC513 remains UNPROVEN. Complete means collection completed within explicit bounds, never transitive closure. Missing upstream/native MIR, actual production root/dependency compilation, allocator objects/final-link identity, finite vtables, Drops, defaults and LLVM fixed point remain open. The older test-support model metadata cohort is feasibility only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Computed matrix saved as matrix.json using quire 0.38.1. FR-038-AC-513 is Inspection/method-without-symbol, with existing binders; this is not production proof credit. New Python controls are scoped feasibility evidence and are not listed as new AC-513 binders by the matrix. Manual source/test examination finds no introduced hollow implementation or false completion assertion. This review is confined to the two-file primitive, not a repository assurance PASS. Plan completion: not assessed. Optional repository-wide semantic review not run.

Full examined scope:

```yaml
scope:
- id: compiler_metadata.rs
  path: tools/read_path_proof/compiler_metadata.rs
  role: examined
  excerpt: Selected-rustc metadata feasibility exporter, not a reachability proof.
- id: test_compiler_metadata.py
  path: tools/read_path_proof/test_compiler_metadata.py
  role: examined
  excerpt: This is a bounded metadata feasibility check, never production closure
    credit.
- id: FR-038-AC-513
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: The V2 read path (`CheckedPackageV2::read` from entry through returning
    its result, including the owned catalog, and every function it reaches) calls
    no `serde_json` parser, deserializer or `RawValue`, constructs no `serde_json::Value`,
    calls no `num-bigint` parse or arithmetic, sorts only with `sort_unstable*`, and
    builds no `BTreeMap`, `BTreeSet`, `HashMap` or `serde_json::Map`; its tables are
    sorted exact-capacity slices and its containers are reserved at exact capacity
    before they are filled. Every read parses the catalog with the shared `quire_canonical`
    reader and charges its own catalog
```
