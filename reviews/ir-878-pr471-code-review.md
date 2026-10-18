---
id: SR-12959
title: IR878 PR471 code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; PR471 head 6891f2967c22e3be09e79a17079b0fc4c298e0c7 vs merge-base 8ac25a0a; tests/it/public_interface.rs
review_set: subset
---

## Summary

Ticket: IR-878. This is a non-author code review of quire-contract-ir PR 471, with the rust-review lane folded in. The diff is test-only: one file, tests/it/public_interface.rs, +158/-2. It adds `tc_456_public_schema_and_model_names_are_portable`, tagged `#[trace("TC-456", "NFR-002-AC-1")]` through `ix_trace_rs::trace`. It also adds `FORBIDDEN_PUBLIC_NAME_PARTS`, `portable_name_violations`, `collect_schema_names` and `public_schema_names`. It hardens `exported_items` (shared with TC-058): an unrecognized `pub` declaration in source_obligation.rs now panics, and a non-`pub use` `pub` line in selected_population.rs now fails.

## Verdict

CHANGES REQUESTED: two medium findings and one low finding. The test follows the merged TC-456 procedure for the cases the author probed. Schema properties, definitions, titles and enum entries are enumerated over all three `schemas/*.schema.json` files. Model root exports are enumerated, and so are the items of both public modules. A new `pub mod x;` fails (mutation P3). A new forbidden item in source_obligation fails (P1). A forbidden schema title fails (P2). An unrecognized declaration kind such as `pub union` fails closed (P4).

The adverse controls are real. They find exactly 3 violations: the runtime title, the `pointer_width` property and the `GumboNode` enum. Replacing `"runtime"` in the list makes the count 2 and the test fails (C1). The benign `jsonpointer` control is real too: broadening `"pointerwidth"` to `"pointer"` fails it (C2).

The vacuity check still found three escapes that the TC-456 procedure says must fail. In FND-001 TC-058 does not catch it either. FND-003 is a matching gap.

Scoped execution on the frozen head: `cargo test --locked --test it tc_456` gives 1 passed. `quire coverage --scope . --strict` gives 538/607 backed, 40 unbacked and 0 contradicted, with NFR-002-AC-1 and TC-456 backed. The base 8ac25a0a gives 536/607 backed and 42 unbacked. These figures match the claims made in the PR body, which was not trusted. The diff has no `/// Trace:` lines, no vendored file or schema, no compatibility layer and no new dependency.

Rust lane: the panics and expects are confined to the test. There is no unsafe code, no integer conversion and no async code. The shared `exported_items` change tightens TC-058 without changing what it accepts on today's sources.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The selected_population guard accepts any `pub use` line but inventories only the first `pub use crate::checked_package::{..}` block. An extra `pub use crate::checked_package::CheckedSelectedPopulation as RuntimeSelectedPopulation;` passes TC-456 and TC-058 (mutation M1: all 5 public_interface tests pass). That breaks the procedure's rule that an unrecognized public declaration must fail. | tests/it/public_interface.rs:264-283 |
| FND-002 | low | Two escapes pass TC-456 itself and are caught only by TC-058's FR-019 table and root-line count, although TC-456 is meant to be independent. In source_obligation.rs, `pub const fn runtime_pointer_width()` is inventoried as `fn` (mutation M2), and `pub static mut` would be inventoried as `mut`. At the model root, an inline `pub mod runtime_bridge {}` or a `pub mod x; // ...` line passes the `pub mod` guard but is never added to `public_modules` (mutation M4). | tests/it/public_interface.rs:226-243, 325-349 |
| FND-003 | medium | Matching is a lowercase substring search over snake_case-only spellings `file_path` and `os_path`, so camelCase and PascalCase names escape: `filePath`, `FilePath` (`filepath`), `osPath`, and `file_system`. Mutation M3 added `filePath`, `osPath` and `file_system` schema properties and the test still passed. These are natural spellings of operating-system-path vocabulary for a Rust type or JSON property. | tests/it/public_interface.rs:17-35, 37-53 |

## Coverage

Examined: the full diff; `exported_items`, `expected_items` and `tc_058_model_root_exports_exactly_the_documented_public_items` (context, shared helper); crates/quire-contract-model/src/lib.rs public lines; selected_population.rs; the column-0 `pub` declarations of source_obligation.rs; schemas/*.json and schemas/README.md; TC-456 and NFR-002.

Mutations ran on a scratch clone of the frozen head, and each was reverted after its run. Escapes (the test stayed green): M1, M2, M3, M4, and M5 (schema `const` discriminator `rust_layout`; see SR-12960). Caught (the test went red): P1, P2, P3, P4, C1 and C2.

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
  excerpt: Enumerate every repository-owned public JSON schema's name-bearing property, definition, title and enum entries. Enumerate the model crate's public root exports and every public item reachable through each public module, including re-exports and public declarations. The current public modules are selected_population and source_obligation; a newly declared public module or an unrecognized public declaration must fail the inventory until its names are checked.
- id: TC-058
  path: tests/it/public_interface.rs
  role: context_only
  excerpt: tc_058_model_root_exports_exactly_the_documented_public_items shares exported_items with TC-456.
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The fix added `strip_comments`, which does not lex string or char literals, so a `/*` or `//` inside a string is taken as the start of a comment. In mutation M6, `const _GLOB_DOC: &str = "/*";` followed by `pub struct RuntimeHandle;` in source_obligation.rs hides the forbidden struct. All 5 public_interface tests pass, TC-058 included, because FR-019 never listed the hidden item. M6b shows the same for `"http://x"`, which hides a later declaration on that line. Head 6891f296 had no such escape: it caught a forbidden pub struct (P1). | tests/it/public_interface.rs:65-84 |
| FND-005 | low | The separator-free matching fixed FND-003 but now flags names that join two words: `profile_path` folds to `filepath`, `repos_path` to `ospath` and `profile_system` to `filesystem`. Mutations F1-F3 failed TC-456 on these benign names. This fails safe, but the test becomes brittle against legitimate names, and `profile` is core vocabulary here (`CanonicalProfile`, `BOUND_IDENTITY_PROFILE`). Matching per word after splitting on case and separator boundaries would avoid it. | tests/it/public_interface.rs:19-49 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head ec344121 (commit 139770f) parses every `pub` statement of selected_population.rs. Aliases export their alias; globs, nested groups and paths outside `crate::checked_package` panic. Re-measured: M1 and M1b (alias, second braced block) fail TC-456 with `forbidden public names`; M1c (glob) and M1e (other path) fail closed. Nested groups were checked by reading only, since a nested path into checked_package does not compile. |
| FND-002 | fixed | `public_item_name` skips the `const`/`async`/`unsafe`/`extern`/ABI qualifiers and `static mut`. Re-measured: `pub const fn runtime_pointer_width` (M2), `pub static mut RUNTIME_X` (M2b) and `pub unsafe extern "C" fn runtime_c` (M2c) each fail TC-456 under their real names. Inline `pub mod x {}` (M4) and `pub mod x; // note` (M4b) now fail TC-456 itself. |
| FND-003 | fixed | Names are folded to lowercase ASCII alphanumerics before matching against `filepath`, `ospath`, `filesystem` and the other terms. Re-measured: `filePath`, `FilePath`, `osPath` and `file_system` (M3a-d) each fail TC-456. The three adverse controls and the `jsonpointer` control still behave (C1 and C2 go red), and P1-P4 stay red. |
