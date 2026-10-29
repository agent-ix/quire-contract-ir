---
id: SR-12995
title: "code review of PR 475 (IR-880 TC-456 public-name inventory test)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/public_interface.rs, tests/it/identity.rs"
review_set: subset
---
# SR-12995: code review of PR 475

## Summary

Ticket: IR-880. PR agent-ix/quire-contract-ir#475, diff `origin/main...HEAD`. This file covers the code-review method with its rust-review lane. It reviews the TC-456 inventory test in `tests/it/public_interface.rs`, which has a schema walker, a Rust source enumerator over every model `.rs` file, macro_rules handling, a seven-table STD-001 extractor, the allowlist, adverse controls and mutations. It also reviews the removal of the NFR-002-AC-3 tag in `tests/it/identity.rs`.

The reviewer ran its own mutants in a throwaway worktree, one rebuild per source mutant, with an env-gated harness for the test-side mutants. Killed: drop schema `const` extraction, drop public-field extraction, drop variant extraction, drop impl-const extraction, drop the across-words matcher, drop the `registered_codes!` constant rows. Survived: drop associated-const value extraction (M5); widen the allowlist to the prefix `KANI_` with any value (M6); drop the STD-001 row condition of the allowlist (M7); drop camelCase splitting (M9); drop the macro_rules escape check (M10); drop trait-fn extraction (M11); drop the allowlist value equality (M16). A source mutant adding the variant `SolverRuntimeBuffer` to the feature-gated export `MappingAllocationPoint` also survived.

## Examined scope

```yaml
scope:
- id: tests/it/public_interface.rs::source_items
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "fn source_items(file: &syn::File) -> Vec<PublicName> { ... Item::Macro(item) => macro_items(item, &definitions, &mut out), _ => {} }"
- id: tests/it/public_interface.rs::macro_items
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "fn macro_items(item: &syn::ItemMacro, definitions: &BTreeMap<String, String>, out: &mut Vec<PublicName>)"
- id: tests/it/public_interface.rs::std001_code_rows
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "fn std001_code_rows(document: &str) -> BTreeMap<String, BTreeSet<String>>"
- id: tests/it/public_interface.rs::registered_code_violations
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "fn registered_code_violations(names: &[PublicName], tables: &BTreeMap<String, BTreeSet<String>>) -> Vec<String>"
- id: tests/it/public_interface.rs::inventory_violations
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "// IR-881: removed when IR-881 merges. let allowed_constant = allow_solver && solver_registered && entry.owner == \"Std001Code\" && entry.name == \"KANI_SOLVER_ABSENT\" && entry.value.as_deref() == Some(\"kani_solver_absent\");"
- id: tests/it/public_interface.rs::tc_456_public_schema_and_model_names_are_portable
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "#[trace(\"TC-456\")] #[test] fn tc_456_public_schema_and_model_names_are_portable()"
- id: tests/it/identity.rs::tc_015
  path: tests/it/identity.rs
  role: examined
  excerpt: "#[trace(... \"FR-012-AC-6\", \"StR-001-VC-1\", \"StR-001-VC-2\")] (NFR-002-AC-3 removed)"
- id: crates/quire-contract-model/src/output_mapping.rs::allocation
  path: crates/quire-contract-model/src/output_mapping.rs
  role: context_only
  excerpt: "mod allocation { pub enum MappingAllocationPoint { RequestObligations, ... PackageIdentity } } #[cfg(feature = \"fault-injection\")] pub use allocation::MappingAllocationPoint;"
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `source_items` iterates only top-level `file.items` and drops every other item kind with `_ => {}`. It never enters inline modules and does not fail on them. So the only feature-gated export, `MappingAllocationPoint`, has none of its 8 variants inventoried: it is defined inside the inline private `mod allocation` and re-exported under `#[cfg(feature = "fault-injection")]`. Reproduced: adding a variant `SolverRuntimeBuffer` to it leaves TC-456 green. This breaks TC-456's feature-gated-export and enum-variant scope, and its rule that an unrecognized public declaration must fail the inventory. | tests/it/public_interface.rs:319-436; crates/quire-contract-model/src/output_mapping.rs:1165-1203 |
| FND-002 | medium | The "feature-gated export" adverse control `#[cfg(feature = "fault-injection")] pub struct RuntimeExport;` is vacuous for its purpose. `source_items` ignores attributes, and the control is a plain top-level struct. It never exercises a `#[cfg]`-gated `pub use` or a gated definition outside the top level, which is the real shape that escapes in FND-001. | tests/it/public_interface.rs:1038 |
| FND-003 | medium | The PR deletes every matcher assertion that `tc_456` had: PathBuf/path_buf/MacOS/macOS/mac_os, RunTime/run_time/USize, PointerWidth64/PathBufs/path_bufs/RunTimes/run_times/MacOSX, filePath/FilePath/osPath/file_system, and the false-positive guards profile_path/repos_path/profile_system. No other test calls `portable_name_violations` or `name_words`. A mutant that disables camelCase splitting survives. The doc comment at lines 20-23 now asserts behaviour that nothing checks. | tests/it/public_interface.rs:20-95, 944 |
| FND-004 | medium | Nothing tests that the allowlist is exact. A mutant that widens it to the prefix `KANI_` with any value survives. So does one that drops the value equality, and one that drops the STD-001 `kani_solver_absent` row condition. TC-456 requires an exact name and value, no prefix, and an extracted row. There is no near-miss control: a different value, a `KANI_SOLVER_*` sibling, or an STD-001 fixture without the row fed through `std001_code_rows`. | tests/it/public_interface.rs:746-765 |
| FND-005 | medium | Associated-constant value extraction is never what catches a control. A mutant that drops `strings.visit_expr` in `constant_names` survives, because every Rust const control (`SOLVER = "solver"`, `EXTRA_SOLVER = "solver"`, the trait `SOLVER`) also has a forbidden name. A mutant that drops trait-fn extraction also survives, because the trait control carries a forbidden const too. TC-456 requires each associated constant's value to be read as well as its name. | tests/it/public_interface.rs:296-307, 367-369, 1035-1039 |
| FND-006 | medium | The macro_rules body escape check (`macro_static_public_names`, the "escaped inventory" assert) and the unknown-`macro_rules!`-with-`pub` assert have no adverse control. A mutant that disables the escape check survives. The macro-generated control (`diagnostic_codes! { SolverLeak => ... }`) exercises only row parsing, so the hand-written per-macro member lists (`["ALL", "as_str"]`, `identifier_type`'s `new`/`as_str`, and the rest) are guarded only by an untested check. | tests/it/public_interface.rs:488-512, 583-636 |
| FND-007 | low | In the allowlist-removal mutation, the `kani_solver_absent` assertion is a substring match over all violations. The `Std001Code::REGISTERED: kani_solver_absent: solver` violation satisfies it on its own, so it does not show that the `KANI_SOLVER_ABSENT` constant's value is rejected separately from the array element. | tests/it/public_interface.rs:1046-1054 |
| FND-008 | low | Two match arms in the exported-trait member match panic with the same message: `TraitItem::Macro(_) \| TraitItem::Verbatim(_)` and `_`. The first arm is redundant. | tests/it/public_interface.rs:385-388 |

## Verdict

CHANGES REQUESTED. The PR is not mergeable as-is because of FND-001. The structure is sound. The schema walker now reads `const` (36 string consts across the three schemas). The seven-table STD-001 extractor asserts that every table exists and is non-empty, rejects duplicate rows, and subtracts `DiagnosticCode::ALL`. `REGISTERED` is compared element by element against the sorted difference and against its source rows. Each registered constant's name and value are checked exactly once. The single allowlist entry carries its IR-881 expiry comment. Both mutations run through the real extractor. Tags use `ix_trace_rs::trace`, there are no `/// Trace:` lines, and the NFR-002-AC-1 and AC-3 tags are withheld (`quire matrix` lists both as `untagged`). Gates the reviewer ran on the head: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --test it -- identity:: public_interface::` 14 passed. Every macro_rules definition is invoked in the file that defines it, so the same-file `definitions` lookup holds today.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | medium | The fix adds a branch for inherent impls in a non-public inline scope: they are inventoried only when their type is re-exported. No control tests that branch. Mutant N5, which skips every inherent impl in a non-public scope, survives all eight TC-456 tests. Together with N5, a real-source mutant that adds `pub fn solver_probe(self) {}` to `impl MappingAllocationPoint` passes TC-456. So the public associated items of the feature-gated export, the FND-001 class, are guarded by nothing. Without N5 the real-source mutant is caught. | tests/it/public_interface.rs:473-500, 1202-1214 |
| FND-010 | low | The fix's re-export resolution branches have no controls, and four mutants survive. N4 does not walk an inline `pub mod` in a public scope. N6 ignores a glob re-export. N7 resolves a renamed re-export by its alias, not its source name. N9 drops the alias names of `pub use` in non-root files. None of these shapes occurs in the model tree today. | tests/it/public_interface.rs:319-361, 410-433 |

Re-measured the PR head from a throwaway worktree on origin's PR head. All 14 earlier mutants are now killed: M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M13, M16, and the real-source `SolverRuntimeBuffer` variant on `MappingAllocationPoint`. New mutants on the fix's code:

- Killed: N1 (do not recurse into inline modules), N2 (ignore exports in `visible`), N3 (fail-closed item panic made silent), N8 (fail-closed trait-member panic made silent), and the real-source `pub fn solver_probe` on the gated type's impl.
- Survived: N4, N5, N6, N7 and N9.

The test build has `fault-injection` on: the root dev-dependency enables it, and `let _: Option<quire_contract_model::MappingAllocationPoint> = None;` compiles. The inventory itself is parsed from source, so it does not depend on the feature. Gates: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --test it -- identity:: public_interface::` 21 passed. Tags are `#[trace("TC-456")]` only, there are no `/// Trace:` lines, and NFR-002-AC-1 and AC-3 are still `untagged` in `quire matrix`.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1. `source_items_in_scope` recurses into inline modules and admits items re-exported by `pub use`. Unknown item kinds hit `_ => panic!("unrecognized declaration kind in source inventory")`. The real-source variant `SolverRuntimeBuffer` on `MappingAllocationPoint` now fails TC-456 with `MappingAllocationPoint::SolverRuntimeBuffer:... solver`. |
| FND-002 | fixed | Round 1. The control is now `mod allocation { pub enum Point { Safe, SolverRuntimeBuffer } } #[cfg(feature = "fault-injection")] pub use allocation::Point;`, and `tc_456_gated_reexport_checks_inline_enum_variants` asserts that the variant is extracted. N1 and N2 are killed. |
| FND-003 | fixed | Round 1. `tc_456_name_matcher_covers_case_boundaries_and_near_misses` restores the matcher cases and the false-positive guards, and adds `name_words("filePath") == ["file", "path"]`. M8 and M9 are killed. |
| FND-004 | fixed | Round 1. `tc_456_solver_allowlist_requires_exact_name_value_and_std001_row` checks two near misses and an STD-001 copy without the row, fed through `std001_code_rows`. M6, M7 and M16 are killed. |
| FND-005 | fixed | Round 1. `tc_456_constant_values_and_trait_functions_reach_the_inventory` uses a benign-name const with value `"solver"` and a trait whose only member is `fn runtime`. M5 and M11 are killed. |
| FND-006 | fixed | Round 1. `tc_456_macro_body_cannot_add_an_uninventoried_public_member` puts `pub fn solver_escape` in an `identifier_type!` body. M10 is killed. |
| FND-007 | fixed | Round 1. The assertion now requires a violation starting with `Std001Code::KANI_SOLVER_ABSENT: kani_solver_absent:`. |
| FND-008 | fixed | Round 1. The redundant `TraitItem::Macro(_) \| TraitItem::Verbatim(_)` arm is removed, and `tc_456_unrecognized_exported_trait_member_fails_closed` covers the remaining panic (N8 killed). |
| FND-009 | fixed | Round 2. `tc_456_reexported_private_type_impl_is_inventoried` uses `mod allocation { pub struct Point; impl Point { pub fn solver_probe(self) {} } } pub use allocation::Point;` and asserts that `Point::solver_probe` is extracted and rejected. N5 is now killed. |
| FND-010 | fixed | Round 2. New controls: `tc_456_public_inline_module_is_inventoried`, `tc_456_glob_reexport_is_inventoried`, `tc_456_renamed_reexport_checks_source_name`, and `tc_456_non_root_reexport_alias_is_inventoried` (a temp-dir file fed through `model_source_inventory`). They kill N4, N6, N7 and N9 respectively. Re-measured: all 22 mutants (M1-M11, M13, M16, N1-N9) are killed, 13/13 TC-456 tests pass unmutated, fmt and clippy are clean, strict coverage reports 40 unbacked rows, and NFR-002-AC-1 and AC-3 are untagged. |
| FND-009 | fixed | Round 2. `tc_456_reexported_private_type_impl_is_inventoried` requires a private inline module's re-exported type and public `solver_probe` method to pass through `source_items` and the portability check. Skipping its impl makes the control fail. |
| FND-010 | fixed | Round 2. `tc_456_public_inline_module_is_inventoried`, `tc_456_glob_reexport_is_inventoried`, `tc_456_renamed_reexport_checks_source_name`, and `tc_456_non_root_reexport_alias_is_inventoried` each require a forbidden name from the source enumerator and portability check. |
