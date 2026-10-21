---
id: SR-14431
title: gap-analysis review of IR-785 repeated protocol references
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
review_set: subset
---

## Summary

Ticket: IR-785. Independent review of PR #489. Plan completion: not assessed

Full computed matrix: 589 criteria; 510 tagged, 67 untagged, 11 method-without-symbol, 1 tagged-by-ignored-test. Strict matrix exits nonzero. New AC-294/295 are untagged; existing AC-239/241 are tagged, but the existing AC-241 single-occurrence test asserts the behavior that the coming coding stage must replace. No passing native receipt or repository-complete coverage is claimed. Source inspection confirms occurrences are excluded from structural node identity; generic empty/duplicate occurrence checks precede structural validation, actual duplicate nodes refuse at later node_id, and source-map validation follows graph validation. No source or test code is added by this PR.

## Verdict

**CONDITIONAL** — Explicit implementation and inherited coverage gaps require owner dispositions; specification readiness is separate from runtime acceptance.

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
    id are independently recomp
- id: FR-038-AC-240
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-782). An extra dependency or a dependency unequal to
    the carrier''s body target refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{i}/dependencies`
    in graph-shape validation; a stale carrier key with matching dependency refuses
    `invalid_package`/`stale-node-key` before the local target join. With both body
    target and sole dependency changed together and identities independently recomputed,
    an absent local target refuses `missing_declaration`/`missing-name` at `/semantic_graph/nodes/{i}/body/target`,
    even if an admitted selected dependency contains that key. '
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
  excerpt: PLANNED/UNRUN (IR-785). Starting with an otherwise admitted public-reader
    package containing one local protocol target and one carrier with `expression`
    ordinal 0, add a second source reference to that target as `expression` ordinal
    1 and its own exact source-map row. Both packages admit through `CheckedPackageV2::read`,
    retain exactly one carrier, and have equal independently recomputed carrier `node_id`
    and `package_id`; the admitted carrier retains both distinct ordinals and both
    source correspondences. A third occurrence with a further distinct ordinal and
    source-map row also adm
- id: FR-038-AC-295
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-785). From the admitting repeated-reference package,
    independently duplicate an occurrence ordinal or a source-map key while retaining
    all otherwise valid identities. The public reader refuses `invalid_source_map`
    respectively at the later `/semantic_graph/nodes/{n}/occurrences/{j}` or `/source_map/{i}`
    pointer. An empty carrier occurrence array refuses `invalid_source_map` at `/semantic_graph/nodes/{n}/occurrences`;
    changing either occurrence role away from `expression` refuses `invalid_semantic_graph`
    at that occurrence''s `/role` pointer before target resolution. '
- id: FR-038-protocol-reference-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'with

    `semantic_form: "protocol_reference"` as `ProtocolForm::ProtocolReference`.

    This carrier has one or more occurrences, all of role `expression`, no

    declaration, owner, nominal identity preimage or recursion group, and a closed
    body

    `{term: "reference", target: NodeRef}`. The body target is an exact typed

    `quire.checked-semantic-node/v1` key in this package''s own

    `semantic_graph.nodes`. The carrier''s `dependencies` SHALL contain exactly

    that target, and its `semantic_type` SHALL equal the target protocol node''s

    `semantic_type`. The body and target participate in the existing struc'
- id: FR-038-protocol-reference-2
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Every source reference to the same target SHALL share the one carrier
    node,

    with a distinct occurrence ordinal and its own `source_map` row keyed by

    (`node_id`, `expression`, `ordinal`) naming that reference''s exact source

    regions. Occurrences and source-map rows enter neither the structural key

    preimage nor the package identity projection. Adding a second source reference

    and its source correspondence SHALL preserve the carrier `node_id` and

    `package_id`; it SHALL NOT introduce a discriminator, a different identity

    preimage or another carrier node with the same key.'
- id: FR-038-protocol-reference-3
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The reader SHALL refuse an empty occurrence array or an occurrence of
    a role

    other than `expression` before target resolution. An empty occurrence array

    SHALL refuse `invalid_source_map` at the carrier''s `/occurrences`, preserving

    the general graph occurrence check. A repeated occurrence ordinal SHALL

    refuse `invalid_source_map` at the later

    `/semantic_graph/nodes/{n}/occurrences/{j}`; a repeated source-map key SHALL

    refuse `invalid_source_map` at the later `/source_map/{i}`. Two actual graph

    nodes with the same `node_id` SHALL still refuse `invalid_semantic_graph`.

    These rules imple'
- id: TC-048-repeated-reference-1
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'For FR-038-AC-294 and FR-038-AC-295 (IR-785, PLANNED/UNRUN), take that

    otherwise admitted public-reader package with one carrier occurrence of

    `expression` ordinal 0. Add a second reference to the same local target by

    adding ordinal 1 to that carrier and a distinct source-map row naming the

    second reference''s own source regions. Independently derive the canonical

    structural key and package identity before and after; assert both packages

    admit, both identities are equal, there is still exactly one carrier, and

    the admitted occurrences and source correspondence retain both references.

    '
- id: TC-048-repeated-reference-2
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'From the admitting two-reference package, separately repeat an occurrence

    ordinal, repeat a source-map key, empty the occurrence array, replace the

    first and then the second occurrence role with a non-`expression` role, and

    insert a second actual carrier node with the same `node_id`. Check each

    exact code and concrete later-entry or member pointer stated by AC-295.

    For the repeated ordinal, retain unique source-map rows so a duplicate row

    does not mask the occurrence failure; for the repeated row, retain distinct

    occurrence ordinals. Keep canonical encoding and semantic identities va'
- id: FR-322-AC-66
  path: /home/peter/dev/quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md
  role: context_only
  excerpt: A package with a `protocol`/`protocol_reference` carrier to its local,
    conforming `protocol_clause` target is admitted with one graph collection; its
    body target, sole dependency and semantic type bind that exact protocol node and
    its recomputed key changes when the target changes. With the body target and sole
    dependency changed together and identities recomputed, a missing local target
    refuses `missing_declaration`/`missing-name` even when a selected dependency contains
    that key, and a present non-protocol target or unequal semantic type refuses `ill_typed`/`operator-ineligible`.
    A
- id: FR-035-AC-9
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'The typed profile exposes `required_source_selections: Vec<CheckedSelection>`
    and the IR adapter conforms to QSpec FR-330-AC-10 and TC-935: input refusals use
    `InvalidInput { node_id }`/`invalid_input`, work stops use existing `Failed`/`Work`
    with unchanged key/limit/consumed counters, and successful checks continue the
    existing FR-038 meter without reset. The adapter preserves exact source provenance
    and emits only `profile.reference`, with no duplicated requirement wire field
    or new refusal payload.'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Repeated-reference admission and new AC-294/295 native evidence remain unimplemented; the current reader and existing single-occurrence test still reject a second valid occurrence. | crates/quire-contract-model/src/checked_package/v2/structural.rs:370-377 |
| FND-002 | medium | Inherited repository matrix gaps remain: strict coverage has 65 other untagged criteria and one ignored-test criterion, so this specification PR cannot claim complete repository verification. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:criterion row |

## Applicability

No applicable AssuranceProfile requiring enforced selection was found. Dependency analysis is inapplicable: no relationship or production dependency changes. Object review and architecture/risk analyses are inapplicable: no domain-object shape or architecture changes. Criterion-strength calibrated analysis was not run: no Jev client is available. Rust execution gates are inapplicable to the two Markdown files and are reserved to the later coding stage.

## Dispositions

Round 1: original findings and CONDITIONAL gap verdict remain unchanged. Both gaps have explicit root owner handoffs; neither is fixed by this Markdown-only PR. Independently re-read the actual Linear comments and unchanged source candidate.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Deferred to the same open IR-785 coding stage under independently read-back owner handoff d3f881c4-eae9-4c1d-99dd-f49d5e0e95b4: remove the second-occurrence rejection, replace the obsolete AC-241 test, implement and natively execute/tag AC-239/241/294/295 controls. This PR changes specification only; all new reader behavior remains PLANNED/UNRUN and IR-785 stays open. |
| FND-002 | deferred | Deferred to existing IR-878 inherited coverage triage under independently read-back owner handoff 37978ea8-8f13-45af-80a2-e2c8f5814bb3. It explicitly retains the 65 other untagged and one ignored-only criterion inventory (representative FR-035-AC-9); specific IR-621 and IR-785 coding ownership remains separate. No repository-complete coverage or implementation acceptance is claimed. |
