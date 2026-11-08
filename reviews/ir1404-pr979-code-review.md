---
id: SR-202611070951309522
title: "PR979 selected compiler metadata code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1404; tools/read_path_proof/compiler_metadata.rs; tools/read_path_proof/test_compiler_metadata.py"
review_set: subset
---

## Summary

Ticket: IR-1404. Independent review of frozen PR979's two new tool files. Reviewed exact-path selection, duplicate/missing/invalid root handling, local FnDef discovery, compiler-owned identities and Instance resolution, generic/indirect/missing MIR gaps, bounds, JSON escaping, failure exit, and Continue emission behavior. No provider replacement, source/config change, vendored code, or closure proof is introduced. The five retained actual-compiler controls passed (0.732s); invocation and compiler logs were inspected rather than rerun. Full Cargo/clippy/deny gates are deliberately deferred by the dispatch: only standalone tools changed; no Cargo crate or workflow changed. No source mutations made.

## Verdict

PASS for the changed feasibility primitive: no introduced defects found. AC513 remains UNPROVEN. Complete means collection completed within explicit bounds, never transitive closure. Missing upstream/native MIR, actual production root/dependency compilation, allocator objects/final-link identity, finite vtables, Drops, defaults and LLVM fixed point remain open. The older test-support model metadata cohort is feasibility only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed exact-path selection, duplicate/missing/invalid root handling, local FnDef discovery, compiler-owned identities and Instance resolution, generic/indirect/missing MIR gaps, bounds, JSON escaping, failure exit, and Continue emission behavior. No provider replacement, source/config change, vendored code, or closure proof is introduced. The five retained actual-compiler controls passed (0.732s); invocation and compiler logs were inspected rather than rerun. Full Cargo/clippy/deny gates are deliberately deferred by the dispatch: only standalone tools changed; no Cargo crate or workflow changed. No source mutations made.

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
