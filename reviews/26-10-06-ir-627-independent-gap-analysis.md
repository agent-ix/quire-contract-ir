---
id: SR-1781
title: "Independent gap analysis — IR-627 structural keys"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@c3f83a41c4c00bca1a7b2ee1c5a81001a882be66; IR-627; FR-038-AC-123 through FR-038-AC-134, changed tests and source"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

The computed Test Matrix tags most IR-627 criteria, but several tagged tests do not exercise the full stated acceptance row. The rekeyed semantic-type case is a concrete missing test, and form-specific mutations and declared-context rows remain absent.

## Verdict

**CONDITIONAL** — these evidence gaps need tests or an explicit scope change; the code defect is recorded in SR-1780.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-125 test changes `semantic_type` while keeping the stale key; it never tests a rekeyed wrong-type integer range | tests/it/checked_package_v2_structural_keys.rs:693 |
| FND-002 | medium | AC-131 requests one mutation per included form, but only `parameter` and `text` cover the generic stage | tests/it/checked_package_v2_structural_keys.rs:318 |
| FND-003 | medium | AC-126 states declared fields and operations; the test exercises in-repo anonymous nodes without those declarations | tests/it/checked_package_v2_structural_keys.rs:747 |
| FND-004 | medium | AC-128 states extreme bounds on declared fields; the test exercises anonymous fixture nodes only | tests/it/checked_package_v2_structural_keys.rs:897 |
| FND-005 | medium | AC-123's state-field body-target mutation row has no direct test | tests/it/checked_package_v2_structural_keys.rs:484 |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format tsv` computed tags for AC-123 through AC-131 and AC-133 through AC-134; AC-132 is untagged. AC-132 and the QSpec positive conformance portion of AC-133 await the separately planned QSpec #191 source fixtures, so this review does not assign their implementation to this PR. The five findings concern obligations this PR marks implemented. The matrix is a binder index: a `tagged` status does not prove each row of a criterion executes.

The changed production stage has an owning FR-038 requirement. No added stub or copied external fixture was found in the diff.

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
| FND-003 | fixed | 4d667e9f70e9d87be18bf5eedb1c9bee067179cc |
| FND-004 | fixed | 4d667e9f70e9d87be18bf5eedb1c9bee067179cc |
| FND-005 | fixed | 4d667e9f70e9d87be18bf5eedb1c9bee067179cc |

## Disposition evidence

**Current disposition: PASS** — every original finding is fixed at `4d667e9f70e9d87be18bf5eedb1c9bee067179cc`.

Rechecked at `4d667e9f70e9d87be18bf5eedb1c9bee067179cc` against original reviewed head `c3f83a41c4c00bca1a7b2ee1c5a81001a882be66`. AC-125 now tests both rekeyed wrong-type forms. AC-131 has one body mutation for the included generic forms, with separate compound-unit and union rows. AC-126 reaches selected-document wrapped fields and a dispatch operation parameter. AC-128 covers declared fields at zero and i128 extremes. AC-123 now uses a state-field binding target and all four tamper rows. `cargo test --workspace tc_226_` passed (19 integration and 2 unit tests; one named performance test ignored). Targeted FR-038 Quire validation, changed-file rustfmt check, and `git diff --check` passed. No new defect was found in this focused fix diff.
