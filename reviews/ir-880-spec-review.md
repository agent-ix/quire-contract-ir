---
id: SR-12980
title: spec review of the amended TC-456 public-name inventory scope
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir@7e53c33fc42965558a0f5f7c14a6ece61adfeb07; spec/core/matrix/TC-456-public-name-portability.md, spec/core/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-880. PR agent-ix/quire-contract-ir#473, head 7e53c33, original frozen candidate before any fix. Diff scoped to `git diff origin/main...HEAD` (two spec files, Markdown only). The amendment answers both IR-880 scope questions: string `const` values are in scope, and "every public item" is defined as every exported item including feature-gated exports and re-exports, plus the public fields, enum variants and public associated functions of each exported type. Re-measured against the tree: the four `schemas/*.schema.json` files carry 36 `const` keywords (34 string, 2 integer `schemaVersion` major/minor in the package-reference schema); `collect_schema_names` in `tests/it/public_interface.rs` does not read `const`; `MappingAllocationPoint` is exported only under `#[cfg(feature = "fault-injection")]` at `crates/quire-contract-model/src/lib.rs:223-224` and `exported_items()` skips it explicitly. The facts behind the ticket hold, and the TC and matrix row correctly stay planned.

## Examined scope

```yaml
scope:
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: "Enumerate every repository-owned public JSON schema's name-bearing property, definition, title, enum entry and string `const` value. A forbidden name used as a schema `const` must fail the check. Enumerate every exported item of the model crate, including feature-gated exports, re-exports and public items reachable through public modules. For each exported type, also enumerate its public fields, enum variants and public associated functions."
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: "Include a positive control showing that a benign `jsonpointer` name is admitted and adverse controls for a runtime title, pointer-width property, architecture-language enum name, forbidden schema `const`, feature-gated export, public field, enum variant and public associated function."
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: "The complete public schema and model-name inventory, including string `const` values and all exported item kinds under their applicable features, has zero forbidden names. Every adverse control is detected and the benign control is not."
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: "Planned until the NFR-002-AC-1 inventory test covers the amended scope. Its existing trace tag alone does not establish that coverage."
- id: TC-456
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| TC-456 | Public schema and model names exclude implementation-specific vocabulary | Unit | P0 | NFR-002-AC-1 | planned: the tagged source/schema inventory test exists, but does not yet cover schema `const` values, feature-gated exports, or public fields, enum variants and associated functions of exported types |"
- id: NFR-002-AC-1
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: context_only
  excerpt: "Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary."
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The procedure requires adverse controls for a forbidden schema `const`, feature-gated export, public field, enum variant and public associated function, but does not require each control to pass through the same extraction path as the real inventory (schema walker / Rust source enumerator). A control fed straight to the name classifier, as most of the current test's controls are (`portable_name_violations(["RunTime", ...])`), stays green when `const` or field extraction is missing, which is exactly the reviewer mutation M5 that motivated IR-880. State that each adverse control is injected as a schema or source fixture and detected through the inventory's extractor. | spec/core/matrix/TC-456-public-name-portability.md:20-34 |
| FND-002 | medium | The item-kind list for exported types names public fields, enum variants and public associated functions only. It leaves out public associated constants and trait items, which the model crate has: `Std001Code` declares macro-generated `pub const` codes and `REGISTERED`, `DiagnosticCode::ALL`, `SchemaVersion::V1_1`, and the exported trait `CanonicalBody`. Read literally, a forbidden name on an associated constant or trait method escapes the inventory. Either add associated constants and trait items to the list, or state that they are out of scope and why. | spec/core/matrix/TC-456-public-name-portability.md:24-25 |
| FND-003 | low | The procedure does not say whether enumeration is over macro-expanded API or source text. `DiagnosticCode` and its variants, and the `Std001Code` constants, exist only inside `macro_rules!` bodies (`identity.rs` `diagnostic_codes!`, `code.rs` `registered_codes!`), so a source-text enumerator like the current `public_declarations` misses them while a rustdoc/expanded enumerator does not. Name the basis, or require macro-generated public items explicitly. | spec/core/matrix/TC-456-public-name-portability.md:23-25 |

## Verdict

FAIL on two medium findings. The amendment answers the ticket's scope questions, its facts re-measure true, and its status and matrix row are honest: both stay planned until the test lands, and the TC states that the existing `#[trace("TC-456", "NFR-002-AC-1")]` tag alone does not establish coverage. `quire validate` is clean on both files, and `quire coverage --scope . --strict` gives byte-identical output on `origin/main` (6224262) and on the head: 40 unbacked rows existed before this PR, none were added, and there are 0 contradicted statuses. The computed matrix still counts NFR-002-AC-1 as backed because the tag exists. That comes from how tag-based coverage works, not from this PR, and the prose says so. Context only, not a finding: if associated constants come into scope, `Std001Code::KANI_*` names would make the reader ask whether "Kani" counts as solver vocabulary; `kani` is not in `FORBIDDEN_PUBLIC_NAME_TERMS`, so those names would pass either way. No code was built or run by this review beyond `quire validate`/`quire coverage`.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | TC-456 now requires each adverse schema or source fixture to pass through the inventory's own extractor; direct classifier inputs do not count. |
| FND-002 | fixed | TC-456 and the matrix now include public associated constants and exported traits and their items, with concrete public API examples. |
| FND-003 | fixed | TC-456 and the matrix now require enumeration of macro-generated public items in the expanded API. |
