---
id: SR-2026110414560009
title: IR-1003 coverage backing code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1081 and IR-1003; PR 898 changed Rust tests,
  private reader implementation and coverage status
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-038
  type: references
---

## Summary

Rust production and test review found one duplicate numeric-ordering implementation. No production correctness defect was established. The canonical sink uses the same encoder and native Vec reservation; selected-document comparison preserves existing intake number semantics and flat lifecycle. No applicable AssuranceProfile is declared under spec/.

## Verdict

PASS after disposition round 1. The original finding is preserved below; its fixed outcome is recorded separately.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | decimal_order introduces a second sign/length/digits integer ordering implementation beside temporal::IntegerString::cmp and magnitude_cmp. Reuse one borrowed comparison implementation from both paths, preserving the narrow accessor's exact logical charges and avoiding an unnecessary owned decimal copy. Search of the existing v2 numeric helpers located temporal.rs:146-164 as the existing owner of this ordering rule. | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:1095-1115 |

## Coverage

Plan completion: not assessed. All changed files were inspected. The independent review used source, computed matrix and supplied focused-run receipts; no Cargo build or aggregate gate was run by the reviewer. The lead owns the final reviewed-head local gate.

Semantic binding review was explicitly requested in the dispatch. Quire matrix reports the 15 selected criteria tagged; selected-document tests include identity mismatch, metadata/version distinction, object-order/number-spelling equivalence and deep bounded-stack lifecycle. Genuine producer controls exercise all ten derived forms and the private API negative probes. QSpec vectors remain by-reference. AC-235 whole Inspection, AC-135 independent refusal oracle, FR-352-AC-6 and NFR-001 cross-platform work remain uncredited. The ambient stale-provider difference is not a PR delta.

### Examined acceptance units

- FR-035-AC-32 (examined), spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md: Implemented (IR-1003). Lowering an absent, derived requested key with an operand emits a `source_keys` entry equal to the source id by rebuilt structural key and the operand closure in that table; table-local index numbers may be rewritten from the source graph, and extra entries with unique rebuilt keys are allowed and validated. Every source-frame reference is a `{"source_key": i}` reference into that table, and an empty output graph with `invalid_input` and empty requested occurrences admits. An output-frame id resolves only in `semantic_graph`. Neither serialized frame resolves a
- FR-035-AC-34 (examined), spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md: Implemented (IR-1003). The lowered source id and output root id, every accounting/request pair, and a refusal's requested id compare by rebuilt structural key across their distinct frames. Permuting `source_keys` or the output graph while rewriting references and retaining keys leaves each comparison and refusal pointer unchanged; unequal keys refuse at the owning FR-330 join.
- FR-038-AC-88 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: A two-member recursion group with an application referencing one member and an outside node retains its exact body, shared non-empty label and structural `{package,path,ordinal}` IDs. In-group and outside references each resolve by their exact structural target ID. A malformed structural ordinal, duplicate ID or unresolved target refuses at its own graph or wire locus. The former group-position/group-reference-ordinal branch is withdrawn in place because no such checked-wire carrier exists.
- FR-038-AC-113 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: `make conformance-qspec` reads the current authoritative dependency-selection vectors from `QUIRE_SPECIFICATION_DIR`, never copying them into IR. It checks each closed `{identity}` selection, exact structural supply and diamond result and fails if the checkout, vector or expected refusal is missing. No package identity is derived or compared to a recorded value.
- FR-038-AC-129 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Implemented (IR-1003). The reader admits against the selected document actually supplied under an identity, after checking that document names the identity and its declarations join the graph. It does not authenticate a historical document snapshot. Two valid documents supplied in separate reads under the same identity are checked independently; the package name is unchanged, while their differing complete JSON values remain distinguishable structurally.
- FR-038-AC-130 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Implemented (IR-1003). The ten derived shapes use one QSpec structural `{package,path,ordinal}` rule at the producer and one reader validation of closed body and graph joins. The model-member type derived from a selected declaration equals the node body and semantic type returned by the reader. No `MemberType::node_key`, second preimage encoder or candidate hash path is exported.
- FR-038-AC-132 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: No copy (IR-627 checkout-reference harness implemented; IR-1003 extended owner, declaration and labelled-group conformance implemented). Authoritative vectors for owner-bearing, declared and labelled-group forms remain at their owning QSL or QSpec checkout and are read there, not copied into this repository. A conformance read fails closed when its source is absent or malformed, as FR-038-AC-112 and AC-176 require. Test-side derivation controls do not replace published goldens.
- FR-038-AC-232 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Implemented (IR-1003). A fresh QSL FR-093 checked and emitted `record Point { x: Int[0,9]; }` value `Point { x: 3 }` is admitted by the public V2 reader; its `record_value` binding references a `numeric.narrow` conversion whose sole argument references the canonical Integer literal `3`, and whose member declaration, result type and semantic type name the same `Int[0,9]` range. In an admitted structural.eq or structural.ne over that value, `composite_application_operands` returns the original record GraphChild and semantic type with Literal domain and no introduced positions. At the p
- FR-038-AC-233 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Implemented (IR-1003). In crate-internal admitted-then-mutated controls, altering only the narrow node's operator, laws, leaves, mode, member kind/declaration, result type, semantic type or argument count/term; the range's presence, Integer base, endpoint binding/literal shape, endpoint literal type, endpoint grammar/order; or the operand literal's value kind, declared/semantic type or canonical spelling returns `MalformedDomain` with the enclosing ordinal and first valid type locus specified above. For both positive and negative ranges beyond i128, a canonical literal one below min 
- FR-038-AC-234 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Implemented (IR-1003). A narrow whose argument reaches a parameter, another application including a second narrow, or a value cycle returns `UnsupportedOperand`/`NonliteralGraphValue` at the first offending node with the enclosing ordinal; an unrelated application remains refused. A direct parameter operand retains its Parameter projection. Existing closed tuple, Option, collection and enum-value descendants remain accepted by their value-form paths, and a narrow integer field inside a supported composite receives only the stated narrow exception. Reader admission and first-refusal o
- FR-038-AC-319 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph shape checks exact dependencies and reference resolution before timed-model interpretation. A meaning-bearing body or owner edit is validated on its own terms; a stale dependency list refuses at graph shape. Reordering source occurrences does not alter structural ids.
- FR-038-AC-320 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: The timed-model step reconstructs the selected model from exactly one root and its referenced nodes. A missing root, dangling or wrong-form target, noncanonical path, and invalid or non-time quantity each refuse with FR-322-AC-120's paired code and cause at the failing member, visiting timed nodes in ascending structural node-id order. For TM-12, a distinct second place of one path in valid aggregate order refuses `invalid_package`/`malformed-clock-place` at its forged `resolved_path` row; TM-30 lists one place twice and refuses `invalid_semantic_graph` with no cause at the repeated 
- FR-069-AC-3 (examined), spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md: A `quire.op.tuple.project` over a selected model tuple resolves each valid position to the published type, including a tuple or union type reference; an out-of-range position or wrong declaring tuple refuses `ill_typed`/`operator-ineligible` at the application node (`/semantic_graph/nodes/{i}`), as QSpec FR-322-AC-152 requires; this is not the member, position or declaration pointer. An anonymous structural tuple is not accepted as that model declaration.
- FR-069-AC-5 (examined), spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md: The published `union_value`, `enum_value`, equality, case-arm and structural-leaf cases pass through the production reader: a nullary union case uses an empty-payload `union_value`; union-family `enum_value` and enum-family `union_value` refuse with the specified typed mismatch; recursive tuple/union declarations admit without a graph recursion group and their leaf walk terminates with QSpec's reachable-text rule.
- FR-016-AC-3 (examined), spec/model/functional/FR-016-canonicalization-digests.md: A deterministic reservation-failure harness forces canonical byte allocation failure and verifies `canonicalization_resource_exhausted` with no partial public bytes.
- FR-038-AC-235 (context_only), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Test controls implemented (IR-1003); qualified Inspection remains planned until IR-1006 lands. Independently count the exact logical graph, term/member, examined-endpoint-byte and examined-literal-byte charges for a small admitted narrow-record fixture. Its measured successful work limit succeeds; one less and zero return `WorkLimit` with the first attempted cumulative charge and no partial operands. At the private seam, a range with long canonical endpoints charges every endpoint byte once per logical visit before comparison even when its first sign or digit decides the order: the i
- FR-038-AC-135 (context_only), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: PLANNED/UNRUN (IR-1141). Each current QSpec `adverse.json` structural and body-grammar mutation refuses with its recorded code, cause and pointer against the current structural-id positive fixture. The harness fails on a missing or unexpectedly admitted mutation, a stale expected-failure entry or a mutation whose actual refusal differs; it reads QSpec directly and does not refresh a hash after mutation.
- FR-352-AC-6 (context_only), spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md: Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, members and identity, returns `UnsupportedExactType` when that complete shape cannot be faithfully represented, or returns `UnavailableIdentityWitness` when a required authoritative opaque identity witness is absent. Each refusal names the requested node, first failing type node and form, and charged work, before exposing any partial view. An admitted union without its authoritative declaration and payload identity witness takes the missing-witness outcome; a declaration key alone does n
- NFR-001-AC-2 (context_only), spec/core/non-functional/NFR-001-determinism.md: For each fixed admitted corpus input and supported profile that yields canonical output, Linux, macOS, and Windows produce byte-identical canonical bytes when cross-platform CI is enabled; compare those bytes directly.
- NFR-001-AC-3 (context_only), spec/core/non-functional/NFR-001-determinism.md: For each fixed rejected corpus input and profile, repeated runs and supported operating systems produce exactly the same complete ordered sequence of diagnostic `(code, path)` tuples.

### Files examined

- crates/quire-contract-model/src/canonical.rs
- crates/quire-contract-model/src/checked_package/v2/composite_operands.rs
- crates/quire-contract-model/src/checked_package/v2/mod.rs
- crates/quire-contract-model/src/checked_package/v2/model_fields.rs
- crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs
- crates/quire-contract-model/src/checked_package/v2/model_members.rs
- crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs
- crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs
- crates/quire-contract-model/src/checked_package/v2/selected_document.rs
- spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
- spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
- spec/checked_package/matrix/tests.md
- spec/core/matrix/tests.md
- spec/model/matrix/tests.md
- tests/conformance_qsl_exact_producer/derived_shapes.rs
- tests/conformance_qsl_exact_producer/main.rs
- tests/conformance_qsl_exact_producer/narrow_record.rs
- tests/conformance_qspec/grouped_ordinal.rs
- tests/conformance_qspec/model_tuple_union.rs
- tests/conformance_qspec/node_index.rs
- tests/conformance_qspec/selection_vectors.rs
- tests/conformance_qspec/timed_model.rs
- tests/it/checked_package_v2_composite_operands.rs
- tests/it/checked_package_v2_reader.rs

### Analysis selection

Code review includes the Rust lane. Gap analysis includes the explicitly requested intent-to-test check. Base and integrity cover the changed criterion/status text. EARS is not applicable because no requirement Description/Behavior/Constraints statement changes; dependency and object lenses are not applicable because no new relationship or public domain object is introduced. Credential and cross-platform workflow changes are outside this PR.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The narrow wrapper and IntegerString::cmp now call the same borrowed compare_integer_parts implementation. The shared helper performs only sign, digit-length and digit comparisons; no allocation, numeric conversion or meter change was introduced. |

### Disposition verification

Inspected the frozen fix diff and unchanged assertions directly. No additional findings. The final local aggregate gate remains the lead’s responsibility.
