---
id: SR-591
title: "PR #205 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@97bcea493dbc71b1785ba67d25ff632a2f8fd141; crates/quire-contract-model/src/checked_package/v2/ (frame.rs, state.rs, structural.rs, mod.rs, operations.rs, model_members.rs, identity.rs, vocabulary.rs, lower.rs), Cargo.toml, Cargo.lock, Makefile, tests/it/ (git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---
# SR-591: PR #205 code and Rust review

## Summary

Ticket: IR-89. PR agent-ix/quire-contract-ir#205, head `97bcea4`. The review
covers code-review with the rust-review lane folded in.

Examined units:

- `v2/frame.rs`: `read_frame_body`, `read_modifies_entry`, `reject_repeat`,
  `validate_frame_semantics`, `frame_defect`, `join`, `resolve_field`.
- `v2/state.rs`: `validate_state`, `validate_placement`, `check_anchor`,
  `resolve_operation`, `check_clause`.
- `v2/structural.rs`: `StructuralForm::role`, `anchor_body`, `clause_body`,
  `is_state_clause_application`, `reference_type_target`, and the
  `parameter_defect` change.
- `v2/operations.rs`: `check_state_clause_result` and the `ReferenceEdge` arm.
- `v2/mod.rs`: `locate_in_preimage` remap and reader order.
- `v2/identity.rs`: `ModelOwner.version`.
- `v2/vocabulary.rs`: the 15 model forms and the new closed vocabularies.
- Catalog bump `61f4a44` to `4c49706`.
- The `string-edge` markers.

Measured by the reviewer at `97bcea4`, with its own logs and exit codes:

| Gate | Exit |
| --- | --- |
| `cargo fmt --all -- --check` | 0 |
| clippy workspace, all targets, `-D warnings` | 0 |
| clippy on `-p quire-contract-model` | 0 |
| `cargo test --locked --workspace --all-targets -- --include-ignored` | 0 (291 passed) |
| model doctests | 0 |
| `make corpus` | 0 |
| `make audit-unsafe` | 0 |
| `make qspec-vectors` against quire-specification `e56756f` | 0 (13 of 13 conformance lines) |

## Verdict

**CHANGES REQUESTED** (one high finding).

These checks passed:

- The reader admits exactly QSpec's 15 `ModelNode` forms.
- `FrameModifiesEntry`, `OperationAnchorBody`, `StateClauseBody` and
  `ParameterBody` match QSpec `schema.json`.
- Occurrence roles match `BodyBindingRules`: frame `generated`, anchor
  `anchor`, clause `claim` and parameter `expression`.
- Reader order is frame step, then state step, then operation step.
- `reference_edge` refuses with no panic.
- The PR adds no `unwrap` or `expect` on untrusted input. It removes the four
  IR-360 `expect` calls in the old frame code.
- The catalog bump is exactly the rev and SHA-256 that QSpec's
  `proposals/checked-package-v2/README.md` names. It adds two operations and
  nothing else.
- There is no compatibility reader for the old bare-key `modifies`.

One defect: the reader refuses a `record_value_type` declaring node that
carries no `declaration`, although FR-040 says it admits. One regression: a
quadratic duplicate check.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A field entry whose declaring node is a `model`/`record_value_type` node without a `declaration` refuses `missing_declaration`/`missing-selection`. FR-040 Behavior, FR-040-AC-3 and the matrix row say such an entry admits with its name unresolved. The cause: `is_model_declaration_node` treats any undeclared `model` node as a model declaration node, and `recover` finds no owner form for `record_value_type`. Reproduced with a probe; only a source-declared `record_value_type` admits. | crates/quire-contract-model/src/checked_package/v2/frame.rs:469-494; crates/quire-contract-model/src/checked_package/v2/model_members.rs:707-727 |
| FND-002 | medium | `reject_repeat` is O(n²) over one frame member. The work meter does not charge it (one unit per entry). It runs twice per frame, in the per-node loop and again in the frame step's re-parse. It replaces the old `BTreeSet` O(n log n) check. Measured in release: 2000, 4000 and 8000 `deletes` entries took 33, 96 and 344 ms. At `bounded()` 1 MiB this is about 0.1 s; with a caller-raised `bytes` limit it grows quadratically. | crates/quire-contract-model/src/checked_package/v2/frame.rs:150-162; crates/quire-contract-model/src/checked_package/v2/frame.rs:346-351 |
| FND-003 | low | The new test `operation_defect_refuses_a_reference_edge_operation` was inserted under the doc comment of the admit control test. The reference-edge test now carries the control test's description, and `operation_defect_admits_catalogued_identity_used_correctly` has lost its doc. | crates/quire-contract-model/src/checked_package/v2/operations.rs:2055-2065 |
| FND-004 | low | The frame step re-parses each frame body with `read_frame_body` instead of carrying the typed entries from the per-node loop. `join` is duplicated almost verbatim in `frame.rs` and `state.rs`. The code is correct but costs a second parse (see FND-002) and keeps two copies of one rule. | crates/quire-contract-model/src/checked_package/v2/frame.rs:346-351; crates/quire-contract-model/src/checked_package/v2/frame.rs:451-466; crates/quire-contract-model/src/checked_package/v2/state.rs:188-203 |

## Dispositions

Round 1, reviewed at `1d89455fc04ddfb60cd2ac932886f1b223cd3688` (rebased on origin/main `a38f3db`).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 1d89455 |
| FND-002 | fixed | 1d89455 |
| FND-003 | fixed | 1d89455 |
| FND-004 | fixed | 1d89455 |

## New findings (disposition pass 2)

Round 2, reviewed at `995bd4bec7fe526c7891f728044fec4c2e7d469e`. No reader (non-test) code changed since `1d89455`, so FND-001 to FND-004 stay fixed.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | A comment still says "QSpec's published `frame_mutations` are replayed by TC-056", but that replay was deleted in `4ee698d`. | tests/it/checked_package_v2_frame_bodies.rs:68-69 |
