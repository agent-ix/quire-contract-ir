---
id: SR-2686
title: "IR-680 code-review (rust-review lane) of the retained expected node key"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #318; crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_structural_keys.rs, tests/it/support/checked_package.rs, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_reader.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_canonical_encoding.rs, tests/it/checked_package_v2_enum_order.rs, tests/it/checked_package_v2_parameters.rs; ticket IR-680"
review_set: subset
---

## Summary

Ticket: IR-680, CODE stage, PR #318 (12 files). Code review with the rust-review lane folded
in. The reviewed commit identity is recorded only in the Linear marker, per this repository's
rule against commit identities in the tree. Model `claude-opus-5-5`; run
`324c4e6a-ee78-4f27-9020-194cd2cd917e`. Static review only: no cargo build or test was run by
the reviewer (the author's gates are reported as Rust 434/434 and model 177/177). The reviewer
ran `quire coverage --scope . --strict` (quire 0.36.1, engine 0.50.1) on the PR head: 25
unbacked rows, 0 contradicted, and FR-038-AC-183/184/185 are not among the unbacked rows.

Checked against FR-038-AC-183, FR-038-AC-184, FR-038-AC-185, the TC-048 "Retained expected node
key" procedure and the TC-226 "Retained derived-key refusal" section on the merged spec.

## Verdict

**Merge-ready after FND-001 (medium, test-only).** The production change matches the spec:

- `CheckedPackageRefusal` gains a module-private `expected_node_id: Option<CheckedNodeId>` and
  the public read-only `expected_node_id(&self) -> Option<&CheckedNodeId>`. No `pub` field, no
  Display or Serialize impl exists for the type (derives are `Clone, Debug, Eq, PartialEq`
  only), so the field participates in `Eq`/`PartialEq`/`Clone` and may show in derived `Debug`,
  as AC-185 allows. Construction is `pub(crate)` only (`new`, `stale_node_key`); no public
  synthetic constructor and no derive/rekey API is added.
- The six former `common.rs` struct literals now go through `CheckedPackageRefusal::new`, which
  sets `None`; each positional argument was checked against the old named field and maps
  correctly (including `noncanonical_number`'s `path` and `document_pointer`).
- The key is retained only at the two stale sites that already computed a value:
  `validate_derived_keys` (closed-shape `derived_key` and `UngroupedPreimage` both flow through
  the single `derived` variable, and `Some(digest)` is reused) and `validate_application_keys`
  (the `computed` digest is stringified once and reused). There is no second FR-092
  derivation. `derived == None` (self-typed with stale `semantic_type`, malformed or
  wrong-arity closed body, `collection_bounds` over a noncollection, non-self-typed shape with
  inconsistent semantic type) keeps the old `refused_at` with no key. The model-declaration
  stale site in `model_members.rs` is untouched and yields `None`.
- Precedence, first-refusal order, meter charges and byte limits are unchanged; no new
  `meter.charge` or hash call is added. The application site's locus moves from
  `self.node.node_id` to the index key `node_id`, which are the same value.
- Because the comparison is digest-only and the new id differs in digest, the expected id is
  always distinct from `locus`.
- Additivity: an organisation-wide code search plus reads of each consumer's main found struct
  literals only in the three spec-listed places (CG `src/kani/generate/outcome.rs`, CG
  `src/oracle/scalar/mod.rs`, quire-driver `tests/drive.rs`). QSL, quire-integration, RT and
  quire-driver `src/` only hold or match the type, never construct or exhaustively destructure
  it. No other breakage was found.
- Tests: the six IR literal sites migrate to a test-authored `ExpectedRefusal` compared field
  by field, with `ExpectedNodeId::{Absent, Present}`. The AC-183 closed-shape (`base.zero`) and
  `UngroupedPreimage` (`value/parameter`, `Shape::of` = `None`) branches, the AC-184
  application key, the dispatch forwarder, the eight self-typed forms, the `None` cases, the
  `Eq` distinction between two keys, and the clone are each exercised, with full-package
  admission after `rename_node` + `rebuild_source_map` + `refresh_identity` as the oracle.
  Mutants that are killed: key dropped (`expect`, and `Present` in the literals), digest
  corrupted, another node's key (positive-control admission fails), key kept on a `None`
  branch (the `None` asserts), and `PartialEq` ignoring the field (`assert_ne!(first, second)`
  with equal public fields).
- Rust idioms: no `unwrap`/`expect`/panic in non-test code; the new public accessor is
  documented; memory is bounded (one optional 64-hex id per refusal); no compatibility layer;
  no SHAs, digests, pins or local paths are added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Derived-key branch's typed `domain` is never asserted: a mutant that changes `domain: NODE_DOMAIN.into()` at the derived-key site survives every test, because the integration tests compare and rekey with `.digest` only and the dispatch check compares reader output with reader output | crates/quire-contract-model/src/checked_package/v2/derived_keys.rs:450; tests/it/checked_package_v2_structural_keys.rs:1706-1708 |
| FND-002 | low | `CheckedPackageRefusal::new` takes six positional arguments, two of them `Option<JsonPointer>` (`path`, `document_pointer`); swapping them compiles silently, where the replaced named-field literals could not be transposed | crates/quire-contract-model/src/checked_package/shared.rs:338-344 |

### FND-001 detail

AC-183 requires the reader to return "the distinct derived typed key". The application site has
a unit assertion on `expected.domain` (`operations.rs` test), but the derived-key site has
none. In `tc_048_derived_expected_keys_rekey_unreferenced_nodes_in_both_branches` the key is
checked only with `assert_ne!(expected.digest.as_ref(), key)` and used only through
`rename_node(&mut rekeyed, key, &expected.digest)`. The self-typed control asserts only
`is_some()`. Failure scenario: derived_keys.rs:450 is edited to any other domain string, for
example the stored `node_id.domain` of a foreign-domain node or a typo constant. The full suite
stays green, and a consumer (CG, QSL) that compares the whole `CheckedNodeId`, or rekeys with
the returned value as a typed id, gets a key that never matches. Fix: assert
`expected == &typed_node_id(&expected.digest)` (or `expected.domain == "<node domain>"`) in the
AC-183 test for both branches and in `assert_self_type_control`.

### FND-002 detail

`noncanonical_number` passes `Some(path)` in slot 2 and `Some(document_pointer)` in slot 6;
both are `Option<JsonPointer>`, so a transposition type-checks. Today all six call sites are
correct. A small named builder or a struct-update form inside `shared` would remove the hazard.
This is quality only, with no behaviour change.

## Dispositions

Disposition pass 1. The PR was rebased onto main with IR-651 (#316) merged; the reviewed
commit identity is recorded in the Linear marker only. A rebase check found no code or test
change outside the fix commit. The downstream struct literals are still only the three
spec-listed sites.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit "IR-680: close review findings on typed key evidence" (short id in the Linear marker). `tc_048_derived_expected_keys_rekey_unreferenced_nodes_in_both_branches` now asserts `expected == &typed_node_id(&expected.digest)`, and `assert_self_type_control` does the same for all eight forms. `typed_node_id` builds its domain from the test-support literal `"quire.checked-semantic-node/v1"`, not from the crate constant, so the mutant `domain: "quire.checked-semantic-node/v0".into()` at the derived-key site now fails both tests. |
| FND-002 | fixed | Same fix commit. Positional `new(..)` is replaced by `new(RefusalFields { .. })`, a `pub(crate)` named-field struct. All six former literal sites use named fields that map correctly. `expected_node_id` is set only by `stale_node_key`, and nothing public is added. |
