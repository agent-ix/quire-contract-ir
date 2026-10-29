---
id: SR-12959
title: IR878 PR471 code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; PR471 vs merge-; tests/it/public_interface.rs
review_set: subset
---

## Summary

Ticket: IR-878. This is a non-author code review of quire-contract-ir PR 471, with the rust-review lane folded in. The diff is test-only: one file, tests/it/public_interface.rs, +158/-2. It adds `tc_456_public_schema_and_model_names_are_portable`, tagged `#[trace("TC-456", "NFR-002-AC-1")]` through `ix_trace_rs::trace`. It also adds `FORBIDDEN_PUBLIC_NAME_PARTS`, `portable_name_violations`, `collect_schema_names` and `public_schema_names`. It hardens `exported_items` (shared with TC-058): an unrecognized `pub` declaration in source_obligation.rs now panics, and a non-`pub use` `pub` line in selected_population.rs now fails.

## Verdict

CHANGES REQUESTED: two medium findings and one low finding. The test follows the merged TC-456 procedure for the cases the author probed. Schema properties, definitions, titles and enum entries are enumerated over all three `schemas/*.schema.json` files. Model root exports are enumerated, and so are the items of both public modules. A new `pub mod x;` fails (mutation P3). A new forbidden item in source_obligation fails (P1). A forbidden schema title fails (P2). An unrecognized declaration kind such as `pub union` fails closed (P4).

The adverse controls are real. They find exactly 3 violations: the runtime title, the `pointer_width` property and the `GumboNode` enum. Replacing `"runtime"` in the list makes the count 2 and the test fails (C1). The benign `jsonpointer` control is real too: broadening `"pointerwidth"` to `"pointer"` fails it (C2).

The vacuity check still found three escapes that the TC-456 procedure says must fail. In FND-001 TC-058 does not catch it either. FND-003 is a matching gap.

Scoped execution on the frozen head: `cargo test --locked --test it tc_456` gives 1 passed. `quire coverage --scope. --strict` gives 538/607 backed, 40 unbacked and 0 contradicted, with NFR-002-AC-1 and TC-456 backed. The gives 536/607 backed and 42 unbacked. These figures match the claims made in the PR body, which was not trusted. The diff has no `/// Trace:` lines, no vendored file or schema, no compatibility layer and no new dependency.

Rust lane: the panics and expects are confined to the test. There is no unsafe code, no integer conversion and no async code. The shared `exported_items` change tightens TC-058 without changing what it accepts on today's sources.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The selected_population guard accepts any `pub use` line but inventories only the first `pub use crate::checked_package::{..}` block. An extra `pub use crate::checked_package::CheckedSelectedPopulation as RuntimeSelectedPopulation;` passes TC-456 and TC-058 (mutation M1: all 5 public_interface tests pass). That breaks the procedure's rule that an unrecognized public declaration must fail. | tests/it/public_interface.rs:264-283 |
| FND-002 | low | Two escapes pass TC-456 itself and are caught only by TC-058's FR-019 table and root-line count, although TC-456 is meant to be independent. In source_obligation.rs, `pub const fn runtime_pointer_width` is inventoried as `fn` (mutation M2), and `pub static mut` would be inventoried as `mut`. At the model root, an inline `pub mod runtime_bridge {}` or a `pub mod x; //...` line passes the `pub mod` guard but is never added to `public_modules` (mutation M4). | tests/it/public_interface.rs:226-243, 325-349 |
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
| FND-004 | medium | The fix added `strip_comments`, which does not lex string or char literals, so a `/*` or `//` inside a string is taken as the start of a comment. In mutation M6, `const _GLOB_DOC: &str = "/*";` followed by `pub struct RuntimeHandle;` in source_obligation.rs hides the forbidden struct. All 5 public_interface tests pass, TC-058 included, because FR-019 never listed the hidden item. M6b shows the same for `"http://x"`, which hides a later declaration on that line. had no such escape: it caught a forbidden pub struct (P1). | tests/it/public_interface.rs:65-84 |
| FND-005 | low | The separator-free matching fixed FND-003 but now flags names that join two words: `profile_path` folds to `filepath`, `repos_path` to `ospath` and `profile_system` to `filesystem`. Mutations F1-F3 failed TC-456 on these benign names. This fails safe, but the test becomes brittle against legitimate names, and `profile` is core vocabulary here (`CanonicalProfile`, `BOUND_IDENTITY_PROFILE`). Matching per word after splitting on case and separator boundaries would avoid it. | tests/it/public_interface.rs:19-49 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | The round-2 word splitting stops terms written across words from matching. `PathBuf` and `path_buf` split into `path`+`buf`, `MacOS`/`macOS`/`mac_os` into `mac`+`os`, `RunTime`/`run_time` into `run`+`time`, and `USize` into `u`+`size`, so none matches `pathbuf`, `macos`, `runtime` or `usize`. Each passed TC-456 as a schema property (mutations N-PathBuf, N-MacOS, N-RunTime). caught them all, and `PathBuf` is the canonical Rust OS-path type name. A fix is to also match terms against each run of adjacent words joined together. | tests/it/public_interface.rs:18-35, 39-44, 47-73, 75-99 |
| FND-007 | low | `strip_comments` treats `r` as a raw-string start only when the previous character is not an identifier character, so prefixed raw literals `br#"..."#` and `cr#"..."#` are lexed as plain strings. A quote inside one closes the plain string early, and a following `/*` starts a false block comment. In mutation M9, `const _B: &[u8] = br#"a" /* "#;` followed by `pub struct RuntimeHandle;` in source_obligation.rs leaves all 5 public_interface tests green. Plain, raw, `r##`, byte, char, escaped and doc-comment cases are handled correctly. | tests/it/public_interface.rs:112-139 |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | low | The run-of-words check requires a run's concatenation to equal the term exactly. So a term split across words escapes when a digit or suffix is attached to the last word: `PointerWidth64` (`pointer`+`width64`), `PathBufs`, `path_bufs`, `RunTimes`, `run_times`, `MacOSX`. The schema property `PointerWidth64` passes every public_interface test. The same names as model `pub struct`s pass TC-456 and are caught only by TC-058's FR-019 table. snake_case spellings with separate digits (`pointer_width_64`) are caught. Testing that the run's concatenation starts with the term, with the term reaching into the run's last word, would close it without readmitting `profile_path`. | tests/it/public_interface.rs:68-87 |
| FND-009 | low | `parse_model_module` reads `src/<module>.rs` and ignores a `#[path =...]` attribute on the public `mod` item. In mutation V5, `#[path = "so_alt.rs"] pub mod source_obligation;` points at a copy that adds `pub struct RuntimeHandle;`, while the unchanged source_obligation.rs remains. All 5 public_interface tests pass. This is contrived, since it needs a deliberate redirect and a stale file, but it is a way past the procedure's "every public item reachable through each public module". Refusing `path` attributes on public modules would close it. | tests/it/public_interface.rs:100-107, 154-163 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |  parses every `pub` statement of selected_population.rs. Aliases export their alias; globs, nested groups and paths outside `crate::checked_package` panic. Re-measured: M1 and M1b (alias, second braced block) fail TC-456 with `forbidden public names`; M1c (glob) and M1e (other path) fail closed. Nested groups were checked by reading only, since a nested path into checked_package does not compile. |
| FND-002 | fixed | `public_item_name` skips the `const`/`async`/`unsafe`/`extern`/ABI qualifiers and `static mut`. Re-measured: `pub const fn runtime_pointer_width` (M2), `pub static mut RUNTIME_X` (M2b) and `pub unsafe extern "C" fn runtime_c` (M2c) each fail TC-456 under their real names. Inline `pub mod x {}` (M4) and `pub mod x; // note` (M4b) now fail TC-456 itself. |
| FND-003 | fixed | Names are folded to lowercase ASCII alphanumerics before matching against `filepath`, `ospath`, `filesystem` and the other terms. Re-measured: `filePath`, `FilePath`, `osPath` and `file_system` (M3a-d) each fail TC-456. The three adverse controls and the `jsonpointer` control still behave (C1 and C2 go red), and P1-P4 stay red. |
| FND-004 | fixed | (code ) lexes string, raw-string, char and byte literals, lifetimes and nested block comments. Re-measured: the `"/*"` string (M6), `r##"a"# /* "##` (M7) and a `'"'` char literal (M8), each followed by `pub struct RuntimeHandle`, now fail TC-456. A probe of public_statements also inventoried the item after: escaped quotes, escaped backslashes, `b"/*"`, `b'"'`, `'\''`, `'\u{22}'`, lifetimes, labels, nested `/* /* */ */` and doc comments. Legitimate lifetimes are not mis-lexed, and the base head passes. The residual prefixed raw-string case is FND-007. The `"http://x"; pub struct` same-line case (M6b) is line-start parsing, not lexing, and it cannot occur because CI runs `cargo fmt --check`, which puts each item on its own line. |
| FND-005 | fixed | Words are matched one at a time, with forbidden adjacent pairs. Re-measured: `profile_path`, `repos_path`, `profile_system` and `jsonpointer` are admitted, and `filePath`, `FilePath`, `osPath`, `OsPath`, `OSPath`, `FILEPATH`, `filepath`, `file_path`, `FILE_PATH`, `os_path`, `file_system` and `FileSystem` are caught. The new false negatives are FND-006. |
| FND-006 | fixed | (code ) also matches a term as the exact concatenation of a run of adjacent words. Re-measured: `PathBuf`, `path_buf`, `mac_os` and `run_time` fail TC-456 as schema properties, and `SourcePathBuf`, `MacOS`, `RunTime` and `USize` fail it as model `pub struct`s. `profile_path`, `repos_path`, `profile_system` and `jsonpointer` stay admitted. `FILEPATH`, `OsPath`, `OSPath`, `filepath`, `FILE_PATH`, `OSPathBuf` and `Run_Time_Error` are caught. The residual digit and suffix case is FND-008. |
| FND-007 | fixed | The hand-written lexer is replaced by `syn::parse_file` (syn 2 was already a dev-dependency; Cargo.toml is unchanged). Re-measured: `br#"a" /* "#;` (M9), `"/*"` (M6), `r##` (M7), the `'"'` char (M8) and the same-line `const..; pub struct` (M10), each followed by `pub struct RuntimeHandle`, now fail TC-456. |
| FND-008 | fixed | (code ) matches a run of adjacent words whose concatenation starts with the term and reaches into the last word. Re-measured: the schema property `PointerWidth64` and the model `pub struct`s `PathBufs`, `RunTimes`, `MacOSX`, `path_bufs` and `run_times` each fail TC-456. `profile_path`, `repos_path`, `profile_system` and `jsonpointer` stay admitted, and the unmodified head passes 5/5 public_interface tests, so existing names raise no new false failures. |
| FND-009 | fixed | A public `mod` that carries a `#[path]` attribute now fails. Re-measured: the V5 redirect fails TC-456 (and TC-058) with "public module source_obligation redirected by #[path]", and the unmodified head passes. |
