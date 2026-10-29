---
id: SR-12015
title: "IR-832 code review (rust lane) of IR PR #427"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{parameter_declaration.rs,structural.rs,mod.rs}, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_parameters.rs, tests/it/checked_package_v2_model_fields.rs, spec FR-038/TC-048/tests.md/FR-019 rows"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-832 code review (rust lane) of IR PR #427

## Summary

Ticket: IR-832, PR quire-contract-ir#427 (one commit on origin/main). The `/rust-review` lane is folded into this file. Diff `origin/main...HEAD`: 10 files, +550/-54. It adds a new `parameter_declaration.rs` (accessor, two public types, two crate-internal unit tests). In `structural.rs`, `parameter_binding` is extracted and `parameter_defect` now calls it, and `literal` takes a `kind_of` closure instead of `&Graph`. The diff also adds the root re-export, the FR-019 row, three tests/it tests in parameters, one in model_fields, and the FR-038/TC-048/matrix status flips.

The gates ran at head in a throwaway worktree with one target dir. `cargo fmt --check` and both clippy lanes (`--workspace --all-targets` and `-p quire-contract-model`, `-D warnings`) are green. `cargo test --workspace --all-targets --no-fail-fast --include-ignored` passed: 629 `it` tests, 215 model lib tests, and the model/root doctests and `source_obligation_public`. One test was skipped. `selected_population::duplicate_max_on_reader_admitted_population_is_not_population` panics without `QUIRE_SPEC_LANGUAGE_DIR`, which is environmental and does not touch this diff. tc_058 (FR-019 inventory against the compiled root exports) passes. Mutation results are in the gap-analysis artifact SR-12016.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| crates/quire-contract-model/src/checked_package/v2/structural.rs | examined | parameter_defect -> parameter_binding(&node.body, &\|id\| graph.kind(id)); parameter_binding; literal(term, value_kind, type_form, kind_of); compound_unit_term exponent caller |
| crates/quire-contract-model/src/checked_package/v2/parameter_declaration.rs | examined | CheckedPackageV2::parameter_declaration; CheckedParameterDeclaration { node, name, level }; CheckedParameterDeclarationError { UnknownNode, NotParameter, MalformedBody } |
| crates/quire-contract-model/src/lib.rs | examined | root `pub use checked_package::{... CheckedParameterDeclaration, CheckedParameterDeclarationError, ...}` |
| spec/model/functional/FR-019-rust-library-interface.md | examined | `checked_package` (V2 parameter declaration) \| types `CheckedParameterDeclaration`, `CheckedParameterDeclarationError` \| FR-038 |
| FR-038 "Typed parameter declaration" | examined | A parameter node whose body fails any of these checks shall refuse `MalformedBody` ... the body is an aggregate of exactly the `"name"` and `"level"` bindings; the `"name"` binding is a `text` literal typed at a `scalar_type`/`text` node whose value is an identifier; the `"level"` binding is an `integer` literal typed at a `scalar_type`/`integer` node whose value is a canonical non-negative decimal string. The accessor shall not otherwise re-check the package. |
| QSL FR-092 "Parameter nodes" | context_only | body: `aggregate{[binding "name" = the name as a text literal typed at the text scalar node (T3), binding "level" = the binder's level as an integer literal typed at the `Integer` node (T2)]}`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `parameter_declaration` finds the node and both literal type nodes with linear `nodes.iter().find(..)` scans. The package already keeps `node_index` (`BTreeMap<CheckedNodeId, usize>`) and the admitted `kinds`, and `comparison_operands` and `composite_operands` use them. That makes each call O(n) over the whole graph, three times, and a driver that binds every parameter operand of every application does O(n·k) work. The `kind_of` closure also decodes tag and form again from the wire instead of reading the kind admission already decoded. The answer is the same on an admitted package (both paths go through `CheckedNodeKind::decode`), so this is a cost and duplication issue, not a wrong result. `scalar_application_operands` has the same scan pattern, so this follows one of two precedents in the module. Use `self.node_index.get(id)` with `self.kinds.get(pos)` and `self.wire.semantic_graph.nodes.get(pos)`. The unit fixture's `kinds: Vec::new()` would then have to be filled, for example by mapping the wire through `decode`. | crates/quire-contract-model/src/checked_package/v2/parameter_declaration.rs:73-85 |

## Verdict

PASS with one low finding. What was checked and found clean:

1. **No admission behaviour change.** Old `parameter_defect`, line by line, read: `let Some([name, level]) = aggregate_members(&node.body) else Body`; `name_ok = binding(name,"name").and_then(literal(.., Text, Text, graph)).and_then(as_str).is_some_and(is_identifier)`; the same for level with `Integer` and `is_non_negative_integer`; `!(name_ok && level_ok) => Body`. New `parameter_binding` keeps the same slice pattern on the same `aggregate_members`, the same `binding`/`literal` calls, and the same predicates (as `.filter`). `parameter_defect` maps `None` to `Defect::Body` at the same point: after the semantic-type check, before the dependencies check. The only difference is that the level side is no longer evaluated after a bad name. These are pure functions with no meter charge, so the defect, its pointer and the refusal order are unchanged. `literal` swaps `graph.kind(&ty)?` for `kind_of(&ty)?`, and both callers in structural.rs pass `&|id| graph.kind(id)`, which is the identical lookup. `literal` has no other callers. Five semantic mutants of the shared check were applied to origin/main's inline code and to head's `parameter_binding`, and were caught by the same tests/it tests on both sides (see SR-12016). The full `it` suite, including the catalogue differential (`checked_package_v2_catalogue_differential`), passes at head.
2. **Accessor contract.** Absent id gives `UnknownNode`. Tag/form other than `value`/`parameter` gives `NotParameter`, checked before the body is read. Body failure gives `MalformedBody`. The post-admission re-check is exactly what the spec lists and nothing else: an aggregate of exactly two members; `"name"` binding, a `text` literal typed at `scalar_type`/`text`, identifier value; `"level"` binding, an `integer` literal typed at `scalar_type`/`integer`, canonical non-negative decimal. The node's semantic type, dependencies, occurrences and owner are not re-checked. The result has exactly `node`, `name` and `level` (`Box<str>`), with no `serde_json::Value`, region, type, function or owner. The error enum is closed: three variants, no `#[non_exhaustive]`, `Copy`, a `thiserror` Display. Non-test code has no `unwrap`, `expect`, indexing or `panic!`: `aggregate_members` uses a slice pattern, and every lookup is `Option`/`?`. `node.clone()` returns the supplied id.
3. **Public surface.** The root re-export adds exactly the two types. The FR-019 row matches the compiled exports (tc_058 passes) and the FR-038 Public type table. The doc comments state the FR-092 sharing rule.
4. **Rust idioms.** `parameter_binding` is `pub(super)` with lifetime-correct borrowing (`&'a str` out of `&'a Value`). The closure is taken as `&impl Fn`, so no allocation and no boxing. Clippy is clean.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 : the accessor now looks up `self.node_index` and `self.kinds`, and does no linear scan and no wire re-decode. `NotParameter` is decided by `kind_of(node) != Some(CheckedNodeKind::Value(ValueForm::Parameter))`. The result is identical on admitted input. `kinds[pos]` is the `decode(tag, form)` of the node at `pos`, which the old wire check compared. Admission refuses duplicate ids, so the index is one-to-one, and the defensive `.filter(node_id == node)` can only turn an inconsistent index into `UnknownNode`. The unit fixture now fills `kinds` by decoding each fixture node. The mutations change only the parameter body, so every check on the post-admission re-check list still runs: the type nodes `1`, `2` and `3` keep their decoded kinds, all 13 mutations still return `MalformedBody`, and the "name typed at a non-scalar_type" case reads node `3` as `Value(Parameter)`. Re-run mutants in a fresh worktree with its own target dir: name/level swap killed by 4 tests, shared-node return killed by 3, any-known-kind-is-a-parameter killed by 3 (including the AC-261 test), ordinal+1 killed by the 2 AC-159..164 scalar tests. fmt and `clippy --workspace --all-targets -D warnings` are green. |
