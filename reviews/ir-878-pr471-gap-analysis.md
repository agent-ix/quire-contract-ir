---
id: SR-12960
title: IR878 PR471 gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; PR471 head 6891f2967c22e3be09e79a17079b0fc4c298e0c7 vs merge-base 8ac25a0a; tests/it/public_interface.rs; spec/core/matrix/TC-456-public-name-portability.md; spec/core/non-functional/NFR-002-portability-independence.md
review_set: subset
---

## Summary

Ticket: IR-878. Planless gap analysis of quire-contract-ir PR 471 against NFR-002-AC-1 and its test case TC-456, which landed in spec PR 470 and is now on main. Plan completion: not assessed.

## Verdict

PASS for this delta, with two low findings. The deviations from the code review (SR-12959) are recorded there. Re-measured with quire 0.36.2 (engine 0.50.2), `quire coverage --scope . --strict`. The base 8ac25a0a has 536/607 rows backed and 42 unbacked, with NFR-002-AC-1 and TC-456 unbacked and the AC untagged. The head 6891f296 has 538/607 backed, 40 unbacked and 0 contradicted. NFR-002-AC-1 is tagged by `tc_456_public_schema_and_model_names_are_portable` (tests/it/public_interface.rs:316), and TC-456 is backed. Strict mode still exits 1 on the 40 rows that were already unbacked, none of them from this PR.

No criterion was weakened. The test is a real check, not a stub: adverse and benign controls plus mutations prove it can fail (see SR-12959). The binding is correct.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-456 lists only property, definition, title and enum entries, and the test follows it, so the 36 schema `const` discriminator values are not inventoried, for example the `kind` names `literal`, `reference` and `composite`. A `{"const": "rust_layout"}` discriminator passes the test (mutation M5). These are name-bearing in the same sense as enum entries, so the procedure has a gap. | spec/core/matrix/TC-456-public-name-portability.md:17-21; tests/it/public_interface.rs:55-68 |
| FND-002 | low | The model inventory leaves out two sets of names. The `#[cfg(feature = "fault-injection")]` root export `MappingAllocationPoint` is skipped by name, and the public fields, enum variants and associated functions of exported types are never enumerated. Examples are `SourceObligationSite::start` and `SourceInventoryOutcome::observed_count`. The phrase "every public item reachable through each public module" leaves open whether these members count. None of today's names contains a forbidden term. | tests/it/public_interface.rs:154-156, 218-247 |

## Coverage

Examined: the TC-456 procedure and expected results, the NFR-002-AC-1 row and the matrix row for TC-456. Coverage JSON from base and head was compared. The test was checked against each procedure clause; the schema-directory scope is `schemas/*.schema.json`, and `conformance-trace-map-v1.json` is a registry, not a schema, per schemas/README.md.

## Examined units

```yaml
scope:
- id: NFR-002-AC-1
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary.
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: Enumerate every repository-owned public JSON schema's name-bearing property, definition, title and enum entries. Enumerate the model crate's public root exports and every public item reachable through each public module, including re-exports and public declarations.
- id: NFR-002
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: context_only
  excerpt: NFR-002 portability and implementation independence; AC-3 and AC-4 are verified separately by TC-015 and TC-016.
```
