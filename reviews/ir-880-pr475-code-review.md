---
id: SR-12995
title: "code review of PR 475 (IR-880 TC-456 public-name inventory test)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@6ec7f1912e9870b491986dda1ce3a9cddcaf7ca2; tests/it/public_interface.rs, tests/it/identity.rs"
review_set: subset
---
# SR-12995: code review of PR 475

## Summary

Ticket: IR-880. PR agent-ix/quire-contract-ir#475, head 6ec7f19, diff `c9d2290...HEAD`. This file covers the code-review method with its rust-review lane. It reviews the TC-456 inventory test in `tests/it/public_interface.rs`, which has a schema walker, a Rust source enumerator over every model `.rs` file, macro_rules handling, a seven-table STD-001 extractor, the allowlist, adverse controls and mutations. It also reviews the removal of the NFR-002-AC-3 tag in `tests/it/identity.rs`.

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
