---
id: SR-5700
title: IR-7 PR337 code-review operation redefinition
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir@c7a29419a31028d456ad5d1d2ad189ae2e943a85; crates/quire-contract-model/src/checked_package/v2/model_members.rs;
  crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs; tests/it/checked_package_v2_temporal.rs
review_set: subset
relationships:
- type: references
  target: ix://agent-ix/quire-contract-ir/FR-038
---

## Summary

Ticket: IR-7; PR #337. Frozen checkout clean. Three changed paths, 131 additions/20 deletions. No spec, schema, public API, dependency, CI or gate change. Review limited to the operation-redefinition delta; AC excerpts identify the portions examined, not full-criterion acceptance. Current published main 7fd9917e4b7a176f58baf6ce354f53f9b86e344b adds IR-484 AC-210..213 only as normative context and leaves AC-28/103 unchanged.

## Verdict

**PASS (PR-diff scope only)** — no defects found.

The actual unit resolver returns Child/act when Gadget and Child both redefine Widget/act, testing the most-derived-same-target rule rather than simple hiding. The public reader uses real digest-bound model evidence and admits the fairness member; removing both links must instead refuse AmbiguousDeclaration/AmbiguousName with the exact member-name path and declaration locus. The malformed-null operation link paired with a dangling parameter must return malformed; the dangling link paired with inverted multiplicity must return missing-name. Both test assertions run unconditionally or fail their alternate result branch. No mocks, stubs, skips or vacuous new assertions. Shared helper extraction preserves field behavior; operation links feed the existing bounded resolver and deterministic defect-row ordering.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Rust lane folded into code-review. Checked repo conventions and default rust-style; no new panic/unsafe/async/shared-state/numeric boundary/resource-growth surfaces, duplication/vendoring, compatibility layer or gate weakening.

No Cargo/build/Kani/lock or full gate run by this reviewer, per dispatch. Independently inspected candidate aggregate receipt and CI logs: 444 integration +193 model tests, 17+7 doctests, zero ignored; fmt/lint/corpus and 578 document grammar checks pass. Actual make ci exit 2; exact baseline 43 unbacked-row triples and zero status lies; retained deny/one-copy/audit/unsafe continuation exits 0. Static matrix run by reviewer: 367 criteria, 314 tagged, 45 untagged, 2 ignored-only, 6 method-without-symbol; gap ID/status set matches baseline. AC-28/103 are tagged with the added tests. Current computed matrix equals retained candidate matrix. Retained coverage has no untracked symbols; ten declaration/method/property-shape diagnostics are context limitations, not delta defects. This is no repository-wide PASS or whole IR-7 acceptance. Remaining temporal/producer acceptance and existing repository gaps are unresolved outside this PR.

No applicable AssuranceProfile found in spec/. Authoritative remote FCD schema independently fetched at 531b007085d5c1977cb903340d8a230d5501e6c5, blob 2f3431b964385ed9bcb2f5ed55567e8cb4c1d271: operation.redefines is optional semanticIdentity reference. No upstream content copied into the reviewed repository. Exact backend model ID is not exposed; configured Codex recorded as model=unknown. Quire0.36.2(engine0.50.2), Quoin0.28.3. CLI matrix emits DuplicateArchetype/InverseEdge diagnostics retained in matrix.stderr; schema with explicit module also emitted UnknownEdgeType diagnostics, exits0. No fallback to old tools.

```yaml
scope:
- id: FR-038-AC-103
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: a `name` that matches two exposed operation members of the declaration
    node refuses `ambiguous_declaration`/`ambiguous-name`, one that the node only
    inherits admits and resolves to its most-derived redefinition
- id: FR-038-AC-28
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: a node failing two rows reports the earlier (a dangling `typeRef` before
    a multiplicity with `lower > upper`, a malformed member before both)
- id: References::redefinition
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: '    /// Reads a field or operation''s optional redefinition reference.'
- id: semantic_ir_object_type::operation-intake
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: '            redefines: references.redefinition(operation, &base, defects),'
- id: DomainModel::resolve
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: "        // The most-derived-redefiner rule: a redefiner of a target that\
    \ a\n        // redefiner in a more derived owner also redefines is hidden."
- id: operation_redefinitions_select_the_most_derived_member
  path: crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs
  role: examined
  excerpt: '    assert_eq!(resolved.identity(), "ix://acme/orders/Child/act");'
- id: operation_redefinitions_refuse_malformed_and_missing_targets
  path: crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs
  role: examined
  excerpt: '        assert_eq!(read(&document(vec![declared])).map(|_| ()), Err(expected));'
- id: fairness_resolves_the_most_derived_operation_redefinition
  path: tests/it/checked_package_v2_temporal.rs
  role: examined
  excerpt: '                "most-derived redefinition must admit: {result:?}"'
- id: read_with_model
  path: tests/it/checked_package_v2_temporal.rs
  role: examined
  excerpt: '    CheckedPackageV2::read(&canonical(package), limits, &evidence)'
- id: FR-038-AC-210
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'PLANNED/UNRUN (IR-484). For a valid self-typed declared `unit` node, references
    to distinct values typed by that node resolve to `quantity`: all twelve catalogued
    `quire.op.quantity.*` entries resolve each required quantity position and reject
    a Boolean at the first one, independent of their later result, member and dimension
    obligations; `quantity.add` and `quantity.eq` with valid same-unit operands also
    admit end to end. Replacing a required quantity argument by Boolean refuses `ill_typed`/`operator-ineligible`
    at that argument; placing a declared-unit quantity in the first `intege'
- id: FR-038-AC-211
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: PLANNED/UNRUN (IR-484). `quire.op.control.if` over a Boolean condition
    and two values of the same declared-unit quantity type admits, while replacing
    the second branch with an Integer value refuses `ill_typed`/`operator-ineligible`
    at `arguments/2` under its `same_family` constraint. A valid `aggregate` in each
    of the three aggregate positions of `quire.op.temporal.clause` and in the first
    position of `quire.op.state.clause` fits `aggregate` at the operation step after
    earlier shape checks. Placing an aggregate in the first `boolean.and` argument
    refuses `ill_typed`/`operator-ineligi
- id: FR-038-AC-212
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: PLANNED/UNRUN (IR-484). A body-root or argument literal whose `value_kind`
    is `integer` and whose `type` names an Integer node admits when the enclosing
    operation is otherwise valid; retaining that value kind while changing only its
    declared type to a Boolean node and re-deriving identities refuses `ill_typed`/`operator-ineligible`
    at the literal's `type`, even if the enclosing operand position accepts either
    family. An enum literal accepts either `enum` or `ordered_enum` type according
    to its nominal declaration, and a `none` literal accepts an option type; each
    mismatched kind/type
- id: FR-038-AC-213
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'PLANNED/UNRUN (IR-484). `quire.op.quantity.convert` over a declared-unit
    quantity and a selected quantity `type_argument` admits mode `{kind: rounding,
    value: exact}`; `toward-zero` on otherwise identical operands and result refuses
    `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`, even
    though the unit type has no explicit rounding binding. The result type alone pins
    this conversion to the declared-unit default `exact`. The same mode check applies
    to `quire.op.collection.sum.quantity` when its quantity binder and member are
    valid.'
- id: SemanticIR2.operation.redefines
  path: agent-ix/filament-core-data/schema/semantic/v1/semantic-ir.schema.json
  role: context_only
  excerpt: '"redefines": { "$ref": "common.schema.json#/$defs/semanticIdentity" }'
bindings:
- test_id: operation_redefinitions_select_the_most_derived_member
  ac_id: FR-038-AC-103
  trace: correct
- test_id: operation_redefinitions_refuse_malformed_and_missing_targets
  ac_id: FR-038-AC-28
  trace: correct
- test_id: fairness_resolves_the_most_derived_operation_redefinition
  ac_id: FR-038-AC-103
  trace: correct
```
