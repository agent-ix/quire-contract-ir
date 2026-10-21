---
id: SR-14632
title: spec-review/base review of IR-785 repeated protocol references
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; IR-785; checked-package reader, protocol-reference
  tests and status prose
review_set: subset
---

## Summary

Ticket: IR-785. PR: agent-ix/quire-contract-ir#492. Frozen head 95c9664464811b3f2101c116f647ad1a8e33faf6 against main 06130a47657c13def85c7f01b97864c6a09138bb. No applicable AssuranceProfile was found. Reviewer performed no Cargo, fmt, metadata, compiler, native or Kani execution. Existing raw execution receipts and exact old/healthy inputs were inspected. Model deployment SKU is unavailable; marker uses the advertised model family gpt-6. Excerpts beyond 600 characters are verbatim prefixes; complete criteria were read.

## Verdict

PASS — PASS for status-only spec changes. FR-038 and TC-048 change acceptance/evidence status only for the reviewed repeated-reference implementation, preserve all normative clauses, and expressly distinguish focused native controls from aggregate qualification. Existing AC-239/241 IR-782 labels are unchanged; AC-294/295 implementation status is supported by inspected receipts. IR-539 required-nullable operation fields and IR-621 public outcome constructors are unchanged. No new requirement statement, relationship, domain object, boundary, security or architecture change: EARS/integrity/dependency/failure/risk/evidence/object analyses are inapplicable to this status-only diff and are skipped. The prior spec PR489 supplies the reviewed contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Original focused log records all 12 public-reader tests passing, locked workspace/all-target/all-feature Clippy and independent default model-consumer Clippy passing. Actual original production structural.rs restores the removed restriction and compiles; the new positive fails specifically at two-reference admission with InvalidSemanticGraph at the second occurrence (exit 101). Byte-exact healthy restoration returns all 12 to passing. All scopes record memory.max=17179869184 and memory.swap.max=0; peak focused memory 3498594304 bytes, red 1223639040, restored 1220120576, all memory events zero. These are native reader controls, not a Kani proof or final aggregate gate.

## Examined Scope

```yaml
scope:
- id: FR-038-AC-239
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-782). A package with a `protocol`/`protocol_reference`
    carrier whose closed reference body targets an exact local `protocol` node, with
    one `expression` occurrence, the same semantic type and exactly that target as
    its dependency, admits through the public V2 reader as `ProtocolForm::ProtocolReference`.
    Both a `protocol_clause` target and a different protocol form (`role` or `control`)
    admit; a reader that accepts only `protocol_clause` fails the second control.
    For each admitted target, the structural key, identity projection and package
    id are independently recomputed; chan
- id: FR-038-AC-240
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-782). An extra dependency or a dependency unequal to
    the carrier''s body target refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}/dependencies`
    in graph-shape validation; a stale carrier key with matching dependency refuses
    `invalid_package`/`stale-node-key` before the local target join. With both body
    target and sole dependency changed together and identities independently recomputed,
    an absent local target refuses `missing_declaration`/`missing-name` at `/semantic_graph/nodes/{i}/body/target`,
    even if an admitted selected dependency contains that key. A present '
- id: FR-038-AC-241
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-782). A carrier with a non-reference or `dependency_reference`
    body or an extra body member fails the closed V2 schema. A carrier with a declaration,
    owner, nominal preimage or recursion group refuses at its offending member before
    target resolution. Multiple valid `expression` occurrences are admitted as specified
    by FR-038-AC-294; they are not a forbidden carrier member. An ordinary node holding
    a generic reference remains an ordinary node; no second protocol-reference form
    or separate collection is admitted.
- id: FR-038-AC-294
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented; tagged native public-reader controls passed (IR-785). Starting
    with an otherwise admitted public-reader package containing one local protocol
    target and one carrier with `expression` ordinal 0, add a second source reference
    to that target as `expression` ordinal 1 and its own exact source-map row. Both
    packages admit through `CheckedPackageV2::read`, retain exactly one carrier, and
    have equal independently recomputed carrier `node_id` and `package_id`; the admitted
    carrier retains both distinct ordinals and both source correspondences. A third
    occurrence with a further distinct or
- id: FR-038-AC-295
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented; tagged native public-reader controls passed (IR-785). From
    the admitting repeated-reference package, independently duplicate an occurrence
    ordinal or a source-map key while retaining all otherwise valid identities. The
    public reader refuses `invalid_source_map` respectively at the later `/semantic_graph/nodes/{n}/occurrences/{j}`
    or `/source_map/{i}` pointer. An empty carrier occurrence array refuses `invalid_source_map`
    at `/semantic_graph/nodes/{n}/occurrences`; changing either occurrence role away
    from `expression` refuses `invalid_semantic_graph` at that occurrence's `/role`
    p
bindings:
- test_id: protocol_reference_reads_both_protocol_target_forms_and_rekeys_on_retarget
  ac_id: FR-038-AC-239
  trace: correct
- test_id: protocol_reference_joins_only_the_local_typed_protocol_target
  ac_id: FR-038-AC-240
  trace: correct
- test_id: protocol_reference_target_refusal_precedes_an_operation_refusal
  ac_id: FR-038-AC-240
  trace: correct
- test_id: protocol_reference_dependency_shape_precedes_key_and_local_join
  ac_id: FR-038-AC-240
  trace: correct
- test_id: protocol_reference_requires_closed_body
  ac_id: FR-038-AC-241
  trace: correct
- test_id: protocol_reference_refuses_forbidden_carrier_members_before_target_join
  ac_id: FR-038-AC-241
  trace: correct
- test_id: repeated_protocol_references_retain_one_carrier_and_stable_canonical_identities
  ac_id: FR-038-AC-294
  trace: correct
- test_id: repeated_protocol_reference_ordinals_remain_unique
  ac_id: FR-038-AC-295
  trace: correct
- test_id: repeated_protocol_reference_source_map_keys_remain_unique
  ac_id: FR-038-AC-295
  trace: correct
- test_id: protocol_reference_occurrences_must_be_nonempty
  ac_id: FR-038-AC-295
  trace: correct
- test_id: each_repeated_protocol_reference_occurrence_must_have_expression_role
  ac_id: FR-038-AC-295
  trace: correct
- test_id: repeated_protocol_references_do_not_allow_duplicate_carrier_nodes
  ac_id: FR-038-AC-295
  trace: correct
```
