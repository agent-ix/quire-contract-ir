---
id: SR-1780
title: "Independent code and Rust review — IR-627 structural keys"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@c3f83a41c4c00bca1a7b2ee1c5a81001a882be66; IR-627; crates/quire-contract-model/src/checked_package/v2/, tests/conformance_qspec/main.rs, tests/it/"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Independent code and Rust review of the IR-627 diff against 1540b3b6c0e4d167fe1ed9116c296e45e7dff258. The owner-free stage admits rekeyed bounded-domain nodes whose semantic type is Boolean: an integer range contrary to FR-038-AC-125 and collection bounds contrary to QSL FR-092’s type-node table.

## Verdict

**FAIL** — a reproducible high-severity admission error remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A rekeyed `integer_range` typed at Boolean admits even though its semantic type must be Integer | crates/quire-contract-model/src/checked_package/v2/derived_keys.rs:301 |
| FND-002 | high | A rekeyed `collection_bounds` typed at Boolean admits although QSL defines its type as the collection node | crates/quire-contract-model/src/checked_package/v2/derived_keys.rs:301 |

## Finding detail

`bounded_key` computes the Integer key and checks only the two bound literals against it. It passes `node.semantic_type.digest` directly into `anonymous`. An attacker can point an `integer_range` at the genuinely keyed Boolean node, rederive its node id from that altered type, update the package identity, and receive `Admitted`. QSL FR-092 fixes an integer range's semantic type to Integer; FR-038-AC-125 explicitly says a range typed otherwise has no derivable key even after rekeying. Require `node.semantic_type` to equal the Integer id before deriving an integer-range key, and add a rekeyed-type regression alongside the existing stale-key mutation.

The same function accepts `collection_bounds` with an arbitrary `semantic_type` digest. QSL FR-092’s “Type nodes” table defines its semantic type as the `K<T>` collection node. The changed reader is supposed to validate QSL’s owner-free ungrouped structural forms; a collection-bounds node over Boolean is not such a form. Check that the semantic-type target is a `composite_type` collection of `set`, `bag`, `sequence`, or `ordered_set` before deriving the key, and add a rekeyed non-collection regression. This check belongs in the owner-free stage, not IR-630’s owner-bearing or grouped work.

## Focused evidence

At c3f83a41c4c00bca1a7b2ee1c5a81001a882be66, `cargo test --workspace tc_226_a_node_is_its_own_type_and_the_key_covers_semantic_type_and_literal_type` passed. In disposable worktree `/tmp/ir627-review-c3f83a4/repro`, a test using `base()` and `rekeyed(&base.package, &base.zero, |node| node["semantic_type"] = node_id(BOOLEAN_KEY))` failed its expected-refusal assertion because `read` returned `Admitted`. Command: `CARGO_TARGET_DIR=/home/peter/dev/worktrees/ir-627-refresh/target cargo test --workspace ir627_repro_rekeyed_integer_range_typed_at_boolean_must_refuse` (exit 101). A second disposable test rekeyed `base.sequence_bounds` after pointing its `semantic_type` to Boolean; `cargo test --workspace ir627_repro_rekeyed_collection_bounds_typed_at_boolean_must_refuse` failed for the same reason: `Admitted`. `git diff --check` passed. Full gates are owned by the lead's pre-PR and pre-merge runs.

## Reviewed scope

derived_keys.rs, mod.rs, model_members.rs, structural.rs; FR-038; conformance_qspec/main.rs; changed tests/it/*.rs and tests/it/support/checked_package.rs; CI workflow diff (empty).

- `FR-038-AC-123` (examined): Tamper regression (IR-627; implemented for the member-read and scalar-operand rows and for a node nothing names; the state-field body target row has no test of its own, only AC-127's stage order, which puts the stage ahead of the state step). Over a package whose model-owned field read names a `bounded_domain`/`integer_range` node keyed as `Int[0, 1000]` (the field declared `Int[0, 1000]` in the selected domain document), the unmutated package admits; the same package with that node's `max` bind
- `FR-038-AC-124` (examined): Closed body of each derived shape (IR-627; implemented; every refused body is keyed again by its own derivation in the test, so only the closed-body rule can refuse it. A derived-shape node in a `recursion_group` also has no derivable key, the IR reading of IR-627-Q4). An `integer_range` or `collection_bounds` node whose body is an `aggregate` of exactly the bindings `min` then `max`, each an `integer` literal typed at the `Integer`-keyed node with a string of its form's grammar (`^(0
- `FR-038-AC-125` (examined): Self-typing and `semantic_type` (IR-627; implemented; an `integer_range` node whose `semantic_type` is not the `Integer`-keyed node also has no derivable key, even when its key is derived over that `semantic_type`). A `scalar_type` or `composite_type` node of a derived shape whose `semantic_type` is another node refuses `invalid_package`/`stale-node-key` at its `node_id`, although its key is unchanged. A `collection_bounds` node whose `semantic_type` is re-pointed at a collection of another elem
- `FR-038-AC-126` (examined): Indirect redirection (IR-627; implemented over nodes of the in-repo fixture: the stage reads no document, so the declared fields and operations of the prose are not built). Over fields and operations declared `Option<Int[0, 1000]>`, `Set<Int[0, 1000]>`, `Sequence<Int[0, 1000]>`, `Reference<O>` and an operation parameter typed `Int[0, 1000]`, a package in which the `option`, a collection or `reference` node keeps its stored key while its body `reference` is re-pointed at a genuinely keyed `Int[0,
- `FR-038-AC-127` (examined): Stage and order (IR-627; implemented). The derived-shape re-derivation runs after the graph-shape stage and the application key stage and before the nominal key stage and every declaration, frame, state, temporal, abstraction and operation step. A package holding a graph-shape defect and a tampered node reports the graph-shape defect; one holding a stale application key and a tampered node reports the stale application key; one holding a tampered node and an `ill_typed` defect in an application
- `FR-038-AC-128` (examined): Exact bounds at the extremes (IR-627; implemented over nodes of the in-repo fixture, not over declared fields). Over fields declared `Int[0, 0]` and `Int[-170141183460469231731687303715884105728, 170141183460469231731687303715884105727]` (the `i128` extremes), a node holding the declared bounds, the `i128` minimum as `min` included, admits and a node whose `max` differs from the declared bound by one in either direction refuses as AC-123 does. A comparison through a lossy float conversion would
- `FR-038-AC-129` (examined): Trust root (IR-627; implemented). A package whose `model_selections` row is re-pointed at a document declaring `Int[0, 10]`, whose node is re-keyed to `Int[0, 10]` and whose `package_id` is recomputed refuses `missing_import`/`missing-selection` at the row's `digest` when the evidence holds only the original document (FR-038-AC-27). The same package admits when the caller's evidence also holds the re-pointed document, because the reader admits a package against the evidence it is given, and the
- `FR-038-AC-130` (examined): One derivation (IR-627; implemented by the unit test of `v2/derived_keys.rs`). For each of the ten derived shapes the key the admission stage derives from a node's own body equals the key `MemberType::node_key` derives from the matching member type, and a node whose body is built from the derived key's own preimage admits. A test derives both for each shape over the bounds of AC-128 and compares them.
- `FR-038-AC-131` (examined): Structural forms (IR-627 owner-free ungrouped stage implemented; owner-bearing, declared and grouped re-derivation remains gated on IR-630). For an owner-free ungrouped node, a stored `node_id` that differs from the QSL FR-092 derived key refuses `invalid_package`/`stale-node-key` at that node's `node_id` for forms including `rational_range`, `decimal_range`, `float_rounding`, `text_bounds`, `model_population`, `compound_unit`, `text`, `parameter`, `union`, `record`, `tuple`, the value forms. De
- `FR-038-AC-132` (examined): No copy (IR-627; checkout-reference harness implemented, positive conformance pending QSpec #191 owner fixtures; IR-630 owner-bearing and grouped conformance remains gated). The vectors or types the reader is checked against for the gated forms come from the source IR-627-Q2 names, are not copied into this repository, and a test that reads them fails closed when the source is absent, as FR-038-AC-112 does.
- `FR-038-AC-133` (examined): Positive fixtures carry derived keys (IR-627; planned, ungated; supersedes FR-038-AC-107 for derived-shape nodes, which stands as built until then). The three fixtures AC-107 reads (`positive-all-families.json`, `positive-clause-operations.json`, `positive-union-nodes.json`) admit end to end with the derived key on every derived-shape node. The same fixture with the key of any one undeclared derived-shape node replaced by a placeholder, and every reference to it left in place, refuses; the nodes
- `FR-038-AC-134` (examined): In-repo fixtures carry derived keys (IR-627; implemented). Every in-repo fixture package (`tests/it/support/checked_package.rs`) carries on each undeclared node of the ten shapes the closed body of its form, its own key as `semantic_type` for a `scalar_type` or `composite_type`, and the derived key, and admits. The migration changes more than keys: the `aaaa` boolean's `literal` body becomes `aggregate{[]}`, and the `bbbb` option, typed at `aaaa` with an empty aggregate body, becomes self-typed

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4d667e9f70e9d87be18bf5eedb1c9bee067179cc |
| FND-002 | fixed | 4d667e9f70e9d87be18bf5eedb1c9bee067179cc |

## Disposition evidence

**Current disposition: PASS** — every original finding is fixed at `4d667e9f70e9d87be18bf5eedb1c9bee067179cc`.

Rechecked at `4d667e9f70e9d87be18bf5eedb1c9bee067179cc` against original reviewed head `c3f83a41c4c00bca1a7b2ee1c5a81001a882be66`. The added `bounded_key` guard ties `integer_range` to the Integer key. `validate_derived_keys` now requires a `collection_bounds` semantic type to resolve to a set, bag, sequence or ordered-set node. The AC-125 test rekeys both forms at Boolean and checks `stale-node-key` at the fresh id. `cargo test --workspace tc_226_` passed (19 integration and 2 unit tests; one named performance test ignored). Targeted FR-038 Quire validation, changed-file rustfmt check, and `git diff --check` passed. No new defect was found in this focused fix diff.
