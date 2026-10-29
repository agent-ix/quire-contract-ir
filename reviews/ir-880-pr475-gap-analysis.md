---
id: SR-12996
title: "gap analysis of PR 475 (IR-880 TC-456 public-name inventory test)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/core/matrix/TC-456-public-name-portability.md, spec/core/matrix/tests.md, spec/tests.md, tests/it/public_interface.rs, tests/it/identity.rs"
review_set: subset
---
# SR-12996: gap analysis of PR 475

## Summary

Ticket: IR-880. PR agent-ix/quire-contract-ir#475. This analysis is planless (Plan completion: not assessed). It checks every procedure obligation of the merged TC-456 against the test, using `quire matrix --scope .` and `quire coverage --scope . --strict` at the head and at the base. Strict coverage reports 40 unbacked rows and 0 contradicted statuses on both. Rows backed drop from 538 to 536 because NFR-002 falls from 3/3 to 1/3: the AC-1 and AC-3 tags are withheld, as intended. The head adds one new diagnostic: the TC-456 status cell matches no status class in the declared vocabulary. The spec-file changes are status prose only, so no spec-review sub-analysis was run.

## Examined scope

```yaml
scope:
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: "Enumerate every exported item of the model crate's macro-expanded public API, including feature-gated exports, re-exports and public items reachable through public modules. ... a newly declared public module, feature-gated export or unrecognized public declaration must fail the inventory until its names are checked."
- id: TC-456
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| TC-456 | ... | NFR-002-AC-1, NFR-002-AC-3 | Implemented for the source/schema inventory, schema `const` values, public associated items, the seven-table STD-001 registry reconciliation, and extractor-fed adverse and mutation controls. ... |"
- id: Core
  path: spec/tests.md
  role: examined
  excerpt: "the public-name inventory (TC-456) is implemented, while NFR-002-AC-1 and NFR-002-AC-3 credit is withheld until IR-881 removes the temporary registered-code entry"
- id: TC-015
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "NFR-002-AC-3 credit and its tag in `tests/it/identity.rs` are withheld, blocked by IR-881"
- id: NFR-002-AC-1
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: "Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary."
- id: NFR-002-AC-3
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: "Public JSON field names, identity kinds, anchors, clause kinds, dependency kinds, and diagnostic codes contain no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary."
- id: NFR-002
  path: spec/core/matrix/tests.md
  role: context_only
  excerpt: "| NFR-002 | ... | AC-4 implemented; AC-1 and AC-3 credit withheld, blocked by IR-881 ... |"
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | One TC-456 procedure obligation is unmet: feature-gated exports with their enum variants, and the rule that an unrecognized public declaration fails the inventory. The source enumerator does not enter inline modules and silently skips unknown item kinds. So the variants of `MappingAllocationPoint`, the one `#[cfg(feature = "fault-injection")]` export, are never checked. The reviewer reproduced this with a forbidden variant, and TC-456 stayed green (SR-12995 FND-001). | spec/core/matrix/TC-456-public-name-portability.md:24-35; tests/it/public_interface.rs:319-436 |
| FND-002 | medium | The status wording overstates coverage. The TC-456 row, the TC-456 `## Status`, and the Core rollup in `spec/tests.md` all say "Implemented" for the inventory and the associated items. That is not true while FND-001 stands. The row also dropped its 🚧 marker: at the head, `quire coverage --strict` emits a new diagnostic that is absent on base, "TC-456 (spec/core/matrix/tests.md:48) has status `Implemented for ...`, which the declared vocabulary classes as nothing". | spec/core/matrix/tests.md:48; spec/core/matrix/TC-456-public-name-portability.md:95-101; spec/tests.md:12 |

## Verdict

CHANGES REQUESTED. Met obligations, verified against the tree:

- Schema `const` values: 36 string consts across 3 schemas, with a const adverse control that a mutant shows is load-bearing.
- Public fields, variants, associated functions and associated constants at the top level of every model source file. The weak controls are SR-12995 FND-005.
- Exported traits, including `CanonicalBody`. `macro_rules!` expansion for the nine generating macros. The weak control is SR-12995 FND-006.
- The seven-table `Code`-row extractor, and `REGISTERED` = extracted rows minus `DiagnosticCode::ALL`, compared element by element in sorted order.
- A matching `Std001Code` constant for each difference element.
- Exactly one allowlist entry, `KANI_SOLVER_ABSENT`/`kani_solver_absent` with IR-881 and an expiry comment. Its exactness is unguarded: SR-12995 FND-004.
- The jsonpointer benign control through the schema walker.
- Adverse controls injected through the schema walker and the Rust enumerator, not passed straight to `portable_name_violations`.
- The unregistered `solver` mutation, with the allowlist present.
- Removing the allowlist exposes the name, the value and the `REGISTERED` element through the real STD-001 extractor.
- The NFR-002-AC-1 tag is withheld (`tests/it/public_interface.rs`) and the AC-3 tag removed (`tests/it/identity.rs`). `quire matrix` shows both `untagged`, and NFR-002 1/3 in coverage.
- Tests use `#[trace]` only.

The strict count of 40 is unchanged, as the author reports, but one new status-vocabulary diagnostic appears (FND-002).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1. The source enumerator walks inline modules and admits gated `pub use` re-exports. Unknown item kinds panic, and two tests cover that. The real-source forbidden variant on `MappingAllocationPoint` now fails TC-456. The residual impl-branch control gap recorded as SR-12995 FND-009 was closed in round 2 by `tc_456_reexported_private_type_impl_is_inventoried`. |
| FND-002 | fixed | Round 1. The TC-456 row reads `🚧 partial:...`, the TC-456 Status reads `🚧 Partial:...`, and the Core rollup says the inventory "is partial". `quire coverage --scope . --strict` again reports 40 unbacked rows and 0 contradicted statuses, with no TC-456 status-vocabulary line. NFR-002 stays at 1/3 and AC-1 and AC-3 stay untagged. |
