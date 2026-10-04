---
id: SR-1206
title: "PR #281 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@98ed90df11a41f8c76aa1b4146b31916d56b4bf7; crates/quire-contract-model/src/checked_package/v2/ (abstraction.rs, rust_spelling.rs, mod.rs, encode.rs, model_members.rs, state.rs, temporal.rs, structural.rs, operations.rs, dependency_references.rs, identity.rs, lower.rs, vocabulary.rs), crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/Cargo.toml, Cargo.lock (git diff origin/main...HEAD, base 19d4e1f)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: references
---
# SR-1206: PR #281 code and Rust review

## Summary

Ticket: IR-509. PR agent-ix/quire-contract-ir#281. This review is the
code-review with the rust-review lane folded in. The repository's own
conventions take precedence.

Examined units:

- `v2/abstraction.rs`: `scan_body`, `scan_members`, `scan_member`, `node_key`,
  `read_body` and the entry readers, `validate_abstraction`, `check_node`,
  `check_identity`, `check_order`, `check_targets`, `check_members`,
  `check_object`, `check_frame` and `check_keys`.
- `v2/rust_spelling.rs`: the 2021-edition keyword list, raw identifiers, NFC,
  tuple indexes and receivers. The list was checked against QSpec FR-450
  "Rust spellings".
- `v2/encode.rs`: `AbstractionNodePreimage`.
- `v2/mod.rs`: `body_grammar`/`BodyGrammar`, `validate_body`, the
  `validate_graph` body loop and step order.
- `v2/model_members.rs`: `TypedSlot.name`.
- `v2/state.rs`: `resolve_operation` made `pub(super)`.
- Every exhaustive match that gains `CorrespondenceForm::AbstractionRelation`
  or `BodyTerm::AbstractionRelation`.
- `lower.rs` `requires_bound`.
- The `unicode-ident` and `unicode-normalization` dependencies and the
  `tinyvec` they bring in.

Measured by the reviewer in its own worktree and target dir:

| Gate | Result |
| --- | --- |
| `make fmt-check` | 0 |
| `make lint` (clippy workspace and `-p quire-contract-model`, `-D warnings`) | 0 |
| `make test` | 0 (329 + 138 + 7 passed; 24 `tc_225` tests, all ok) |
| `make deny` | 0 (advisories, bans, licenses, sources ok) |
| `make cargo-audit` | 0 (171 crates, no advisories) |
| `quire coverage --scope . --strict` | 23 unbacked rows, 0 contradicted. It was 35 at base 19d4e1f; FR-346 is 10/10 backed. |
| `make spec` grammar | 1 finding (FR-014), the same as the baseline |

## Verdict

**CHANGES REQUESTED** (two medium findings, one low).

These checks passed:

- The step order is frame, state, temporal, abstraction, operation
  (`validate_graph`).
- Within one node the order is shape, identity, order, targets, members.
  Key uniqueness runs over all admitted nodes after every node's own checks.
- `node_id` is recomputed once, through `quire_canonical::sha256`, over an
  `Encode` preimage of `{version: "quire.abstraction-relation-node/v1", body}`.
  There is no second hash path. A hash error maps to `invalid_semantic_graph`
  at the node, as the application-key path in `operations.rs` does.
- `is_frame` became `body_grammar`. This keeps the behaviour for frames and
  terms; only `correspondence`/`abstraction_relation` selects the new grammar.
- Every match is exhaustive with no catch-all arm. `lower.rs` only adds the
  form to the `false` arm of `requires_bound`, and lowering is the generic
  path, so no lowering outcome is invented.
- The PR adds no `unwrap`, `expect` or panic on a production path.
- Sort and uniqueness are linear or n log n. Operation and member resolution
  charge the work meter, and the body walk charges one unit per value.
- The new `CorrespondenceForm` variant has no consumer outside this
  repository.
- Dependencies are in scope. QSpec FR-450 requires the 2021-edition Rust
  `IDENTIFIER` and NFC. The exact `=` pins match every other dependency in
  `quire-contract-model/Cargo.toml`. `unicode-ident` 1.0.24 was already in
  the lock. Licenses pass deny. MSRVs are 1.71 and 1.36, against the
  workspace's 1.98.1.
- The nested-application scan does not depend on the unmerged IR-495 code. It
  runs in today's per-node body loop.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A `type`, `population` or `context` node key outside the node-identity domain, or with a non-hex digest, refuses `invalid_semantic_graph` at the entry. `node_key` treats it as a body of the wrong shape "whatever the wire said about the domain". FR-038 "Frame bodies" refuses a frame-body entry that names a node key outside the reader's node-identity domain as `digest_domain_mismatch`. `visit_reference` does the same for every term and frame-body reference, which is FR-038-AC-2's "cross-domain digest" class. FR-346 cites FR-040's mapping of schema cases, and neither FR-346 nor FR-451 states this divergence. The AC-2 test uses the string `"a name"`, so the wrong-domain case is untested. | crates/quire-contract-model/src/checked_package/v2/abstraction.rs:190-195; crates/quire-contract-model/src/checked_package/common.rs:905-925 |
| FND-002 | medium | `TypedSlot.name` is read from the domain document with `value.get("name").and_then(Value::as_str)` for every field, parameter and result slot, with no validation. FR-038 "Model-owned members" step 1 refuses malformed declarations at the selection row. A nameless parameter, a non-string name or two parameters with one name are not refused there. Instead, every frame entry over that operation refuses `invalid_model_binding`/`malformed-declaration` at the frame entry, which blames the relation body for a domain-document defect. No FR states this parameter-name read or its failure mode, and no test covers an operation whose parameters carry no name. Every other fixture in the repository builds unnamed `params`. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1856-1860; crates/quire-contract-model/src/checked_package/v2/abstraction.rs:657-672 |
| FND-003 | low | `scan_members`/`scan_member` is new native recursion that grows with the body's JSON depth. Today it is safe only under `on_stack_for`/`stacker` and the `depth` limit, which planned FR-038-AC-117 (IR-495) deletes. `tc_225_a_deeply_nested_member_refuses_as_the_wrong_shape` sets `CheckedPackageReadLimits.depth`, which AC-117 removes. It matches today's recursive `validate_term`, but it adds one more walker for IR-495 to rewrite. An explicit-stack walk, like `validate_placement` in `state.rs`, costs the same. | crates/quire-contract-model/src/checked_package/v2/abstraction.rs:150-188; tests/it/checked_package_v2_abstraction_relation.rs:885-934 |

## Dispositions

Round 1 was reviewed at 0768afb609da09d3a82043b1fd63a1780b0a6afa. The reviewer re-ran the gates: fmt-check, lint and test all exit 0 (332 + 138 + 7 passed). Strict coverage gives 23 unbacked rows and 0 contradicted; grammar has 1 finding (FR-014).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 0768afb. `node_key` refuses a wrong-domain, non-hex, uppercase or short key `digest_domain_mismatch` at the key's own path (`.../objects/{i}/type`, `.../populations/{i}/population`, `.../frames/{i}/context`). This matches `visit_reference`, which refuses at the key member `at.key(key)`, and FR-038 "Frame bodies". A value that is not a node key stays `invalid_semantic_graph` at the entry, as merged AC-2 states. FR-346 Body shape and AC-2 now state this. `tc_225_a_node_key_outside_the_node_domain_refuses_as_a_domain_mismatch` covers 4 keys x 3 members. |
| FND-002 | fixed | 0768afb. FR-346 step 4 and AC-7 now state the IR reading: a declared name is the slot's `name`; names that are missing, non-string or duplicated refuse `malformed-declaration` at the frame entry. `check_frame` now requires distinct declared names. This is consistent with FR-450-AC-6, under which an entry naming a parameter twice is refused, so declared `[from, from]` can never be bound exactly. `tc_225_an_operation_without_distinct_parameter_names_cannot_be_bound` covers the four cases. Refusal on the domain-document side is tracked in IR-571. |
| FND-003 | fixed | 0768afb. `scan_members` is an explicit-stack pre-order walk (children pushed in reverse, outermost first) with an arena of path steps. It counts one work unit per value, as before. `Trail::steps` is iterative. No native recursion on body depth remains in this walker. The one exception is the shared `validate_term` call on the first application found, which is IR-495's to remove. |
