---
id: SR-12155
title: "IR-851 code review (rust lane) of IR PR #438"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{relationship_end.rs,model_fields.rs,model_members.rs,operations.rs,mod.rs}, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_model_fields.rs, spec FR-038/TC-048/tests.md/FR-019 rows"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-851 code review (rust lane) of IR PR #438

## Summary

Ticket: IR-851. PR quire-contract-ir#438 is one commit on the merge, which is the merged spec PR #435. The `/rust-review` lane is folded into this file. The diff (`origin/main...HEAD`) touches 12 files, +928/-39. It adds:

- `v2/relationship_end.rs`, with the accessor, the two public types and two crate-internal unit tests;
- `RetainedRelationship` and `RetainedModels.relationships`, built in `RetainedModels::new` (`model_fields.rs`);
- `DomainModel::end_declaration_key` (`model_members.rs`), which now serves both the reaches/navigate branch of `check_relationship_member` (`operations.rs`) and retention;
- the root re-export, the FR-019 row, six tests/it tests and one helper, and the FR-038/TC-048/tests.md status flips.

origin/main has moved on to (IR-210 #434). `git merge-tree` merges it cleanly.

Gates ran at head in a throwaway worktree with its own target dir:

- `cargo fmt --check` is green.
- Both clippy lanes (`--workspace --all-targets` and `-p quire-contract-model`, `-D warnings`) are green.
- `cargo test --workspace --all-targets --no-fail-fast --include-ignored` passes with `QUIRE_SPEC_LANGUAGE_DIR` set: 640 `it` tests, 217 model lib tests and `source_obligation_public`. Without that variable, `selected_population::duplicate_max_on_reader_admitted_population_is_not_population` panics. That is environmental and does not touch this diff.
- tc_058 passes, so the FR-019 row equals the compiled root exports.

Mutation results are in the gap-analysis artifact SR-12156.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| crates/quire-contract-model/src/checked_package/v2/relationship_end.rs | examined | CheckedPackageV2::relationship_end(&self, application: &CheckedNodeId) -> Result<CheckedRelationshipEnd, CheckedRelationshipEndError>; nine-field CheckedRelationshipEnd; closed enum UnknownNode / NotReachesApplication / UnresolvedEnd |
| crates/quire-contract-model/src/checked_package/v2/model_fields.rs | examined | RetainedModels.relationships: BTreeMap<(usize, Box<str>), RetainedRelationship>; RetainedModels::new builds source_key/target_key via end_declaration_key(..).and_then(Result::ok); RetainedModels::relationship(node) |
| crates/quire-contract-model/src/checked_package/v2/model_members.rs | examined | DomainModel::end_declaration_key(type_ref) -> Option<Result<String, quire_canonical::Error>> |
| crates/quire-contract-model/src/checked_package/v2/operations.rs | examined | check_relationship_member: match owner.package.end_declaration_key(&destination.type_ref) { None => ineligible(member_at("name")), Some(Ok(key)) => key, Some(Err(_)) => key_refused() } |
| crates/quire-contract-model/src/checked_package/v2/mod.rs | examined | validate_package: ... validate_diagnostics(wire, limits, &mut meter)?; Ok((kinds, RetainedModels::new(index, models))); PartialEq compares wire and kinds only |
| crates/quire-contract-model/src/lib.rs | examined | root pub use adds CheckedRelationshipEnd, CheckedRelationshipEndError |
| spec/model/functional/FR-019-rust-library-interface.md | examined | `checked_package` (V2 relationship end) \| types `CheckedRelationshipEnd`, `CheckedRelationshipEndError` \| FR-038 |
| FR-038 "Typed relationship end of a reaches application" | examined | Selection, Retention, Result, refusal paragraphs; the Public type table of nine fields and three closed variants |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The accessor re-implements admission's end-resolution rule instead of sharing it. `relationship_end` has its own `match (source.role == name, target.role == name)`, and its own pick of destination and `forward`. That duplicates `check_relationship_member` (`operations.rs:2179-2192`), and `navigation_type` (`model_members.rs:257`) holds a third copy of the forward-to-destination choice. FR-038 says the accessor resolves "through the same rule as admission". The PR shares only the destination-key derivation (`end_declaration_key`), not the direction/destination selection. The three copies agree today, so nothing is wrong now. A change to one (for example a new ambiguity rule) would silently make the accessor disagree with what admission checked. Lift one `RelationshipDecl` method that returns `forward` with the receiver and destination ends, or a typed `missing`/`ambiguous` outcome, and call it from admission, `navigation_type` and the accessor. | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:101-108; crates/quire-contract-model/src/checked_package/v2/operations.rs:2179-2192; crates/quire-contract-model/src/checked_package/v2/model_members.rs:257 |
| FND-002 | low | Several docs went stale when relationship records were added. The `RetainedModels` doc still lists "the declaration index, each document's field tables, and the selected operation parameter types". The `CheckedPackageV2::models` field doc says "the declaration index and the field tables". The equality doc names only "the retained field tables and declaration index" as excluded. The `model_fields.rs` module doc does not mention relationship records. A reader cannot tell from the docs that relationship ends are retained, or that they are outside equality. | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:1-4,407-411; crates/quire-contract-model/src/checked_package/v2/mod.rs:551-560 |
| FND-003 | low | A ticket id appears in a code comment: `// --- IR-851: the typed relationship end of a reaches application ---`. Ticket ids belong in commits and the tracker. A section comment should name the FR-038 subsection. | tests/it/checked_package_v2_model_members.rs:3318 |
| FND-004 | low | The in-crate test helpers `id`/`node_id` are copies of the ones in `parameter_declaration.rs`'s test module. `quire coverage` now reports two new `[oracle-resembles-implementation]` diagnostics, one for each file pointing at the other, at token similarity 1.00. A shared `#[cfg(test)]` helper in `v2` would remove both the duplicate and the new diagnostic noise. | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:142-148 |

## Verdict

PASS with one medium and three low findings. None blocks correctness. What was checked and found clean:

1. **Public shape matches FR-038.** `CheckedRelationshipEnd` has exactly the nine fields of the table, in table order and with the table types (`application`, `relationship`, `role: Option<Box<str>>`, `destination_type`, `lower: u64`, `upper: Option<u64>`, `ordered`, `unique`, `forward`). It has no `serde_json::Value`, member type, population or charge. `CheckedRelationshipEndError` is closed: three variants, no `#[non_exhaustive]`, `Copy`, a `thiserror` Display. The root re-export adds exactly the two types. The FR-019 row matches, and tc_058 passes. `selected_population` gains no root `pub use`.
2. **Semantics.** Forward means the name equals the source role. The destination is then the target end with `target_key`; the inverse case is symmetric. `role` is copied from the destination end, so an omitted role gives `None`. `upper` passes through with no change, so unbounded gives `None`. `destination_type` is `NODE_DOMAIN` plus the retained key derived from the declared end type, so a subtype operand does not change it and no graph lookup happens. Refusal order: an unknown id gives `UnknownNode`. A body root that is not `term: application` with identity `quire.op.model.reaches` gives `NotReachesApplication`. Every member, declaration, name or role defect, and a missing retained key, gives `UnresolvedEnd`. Non-test code has no `unwrap`, `expect`, indexing or `panic!`. Every lookup goes through `Option`/`?`, `node_index.get` plus `nodes.get` plus a node-id filter.
3. **The refactor is byte-identical.** The old inline code was `object_types.get(type_ref)`, then None gave `ineligible(member_at("name"))`. The form was SystemsInterface if `interface`, else ObjectType. `declaration_key(&identity, form, type_ref, bytes)` with Err gave `key_refused()`. The new `end_declaration_key` reads the same `DomainModel` fields with the same arguments, and the caller maps `None`, `Some(Ok)` and `Some(Err)` to the same three outcomes, at the same point after the direction check. No meter is charged in either form. `declaration_key`/`DeclarationForm` remain in use elsewhere in `operations.rs`.
4. **Retention is outside equality, charge-free and order-neutral.** `RetainedModels::new` runs after every validator in `validate_package` (`mod.rs:954-971`), so it cannot change a refusal or its order. It takes no `WorkMeter`. `PartialEq for CheckedPackageV2` still compares `wire` and `kinds` only. Retention costs one clone of each `RelationshipDecl` plus two derived keys per selected relationship. That is linear in the selected documents, which the reader already bounds by bytes. FR-038 "Retention" requires a record for every selected relationship, so this is mandated.
5. **A key stored as `None` cannot reach an admitted reaches application.** For an admitted application, admission's destination is the same end, by the same rule, that the accessor picks. Admission refuses when `end_declaration_key` is `None` (ineligible) or `Err` (key_refused). Retention calls the same function on the same model, so the retained key of that end is `Some`. The `None` arm of the accessor is reachable only through a hand-built retained record.
6. **Visibility and docs.** New internals are `pub(super)`. `with_relationship` is `#[cfg(test)]`. Every pub item has a doc comment.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1. `RelationshipDecl::select_ends(name) -> Result<SelectedEnds, EndSelectionError>` and `RelationshipDecl::ends(forward)` now live in `model_members.rs`. `check_relationship_member` maps `MissingName` and `Ambiguous` to the same `missing_name()` and `ambiguous()` refusals at `member_at("name")`, at the same point as before. `navigation_type` reads `self.ends(forward).1`. The accessor calls `select_ends` and `RetainedRelationship::destination_key(forward)`. The only role-to-end `match` left in v2 is the one in `select_ends` (`model_members.rs:286-287`). Re-verified in a fresh throwaway worktree with its own target dir: the full workspace suite is green (654 `it`, 219 model lib), the pinned `REACHES_ADMISSION_WORK = 97` test passes, and the AC-252..257 reaches tests pass unchanged. Three mutants of the shared helper were killed by existing admission tests and by the new ones: swapping the missing/ambiguous refusals in admission, flipping `ends`, and resolving both roles as forward. |
| FND-002 | fixed | Round 1. The `CheckedPackageV2::models` doc, the equality doc, the `RetainedModels` doc and the `model_fields.rs` module doc now name the relationship records and say they are outside equality. |
| FND-003 | fixed | Round 1. The section comment now reads `// --- FR-038: the typed relationship end of a reaches application ---`. |
| FND-004 | fixed | Round 1. `id` and `node_id` moved to an inline `#[cfg(test)] mod test_support` in `v2/mod.rs`, and both `parameter_declaration` and `relationship_end` tests import them. The identity-digest source scan's `production_source` strips inline `#[cfg(test)]` items, so its `cfg(test)` ancestry oracle needs no new entry, and none was added. `quire coverage` no longer reports the two `oracle-resembles-implementation` diagnostics; the warning count is 12, down from 14. |
