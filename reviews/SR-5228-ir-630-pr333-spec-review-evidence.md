---
id: SR-5228
title: evidence review of IR-630 structural key specification
type: SpecReview
analysis: evidence
scope: agent-ix/quire-contract-ir@47861328129cc60638217f0fd858402b3815a30b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md
review_set: subset
---

## Summary

Ticket: IR-630. Ran deterministic quoin advise and examined its selected changed criteria: authored Test classes match recommendations. Manual test-plan judgment found one stop-after-exhaustion control that does not necessarily reach group-stage exhaustion.

## Verdict

**CONDITIONAL** — One medium evidence-plan defect needs a same-PR correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The total-read M/M-1 boundary can exhaust after group validation, so its counter does not falsify continued group work after an in-group charge fails. Force exhaustion within the group stage while group work remains, and assert the failed-charge pointer, no admitted package and no subsequent group visits/digests. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3883 |

## Examined Scope

```yaml
scope:
- id: FR-038-AC-124
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Closed body of each derived shape (IR-627; implemented; every refused body
    is keyed again by its own derivation in the test, so only the closed-body rule
    can refuse it. The earlier assertion that membership in a recursion group itself
    leaves no derivable key is superseded by the planned AC-145 through AC-150 group
    derivation). An `integer_range` or `collection_bounds` node whose body is an `aggregate`
    of exactly the bindings `min` then `max`, each an `integer` literal typed at the
    `Integer`-keyed node with a string of its form's grammar (`^(0|-?[1-9][0-9]*)$`
    for `integer_range`, `^(0|[1-9][0-
- id: FR-038-AC-127
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Stage and order (IR-627 baseline implemented; extended IR-630 two-pass
    controls PLANNED and UNRUN). The derived-shape re-derivation, both IR-630 passes
    included, runs after the graph-shape stage and the application key stage and before
    the nominal key stage and every declaration, frame, state, temporal, abstraction
    and operation step. Owner joins precede both passes: an owner defect wins over
    a stale structural key even on a lower-key node. Within IR-630 the complete first
    pass precedes group-label comparisons. A package holding a graph-shape defect
    and a tampered node reports the graph-shape '
- id: FR-038-AC-131
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED; new IR-630 controls UNRUN. Every structural form, declared or
    not and owner-bearing or not, is re-derived from its actual node members under
    the one QSL FR-092/FR-094 production authority. Nominal and application nodes
    retain their own key authorities; only correspondence/abstraction_relation is
    excluded from structural derivation. One-body-value mutations retaining old keys
    cover rational_range, decimal_range, float_rounding, text_bounds, model_population,
    compound_unit, text, parameter, union, record, tuple, value forms and owner-bearing
    clause functions. Each reaches invalid_packag
- id: FR-038-AC-132
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: No copy (IR-627 checkout-reference harness implemented; IR-630 extended
    conformance PLANNED and UNRUN). Authoritative vectors for owner-bearing, declared
    and labelled-group forms remain at their owning QSL or QSpec checkout and are
    read there, not copied into this repository. A conformance read fails closed when
    its source is absent or malformed, as FR-038-AC-112 and AC-176 require. Test-side
    derivation controls do not replace published goldens.
- id: FR-038-AC-145
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED (IR-630); UNRUN. Correctly keyed and labelled record List { next?:
    List; }, record Tree { kids: Sequence<Tree>[0, 3]; } and the owner-free QSL G1
    self-option group admit through both structural-key and group-digest passes. Every
    structural member is authenticated, including the declared record, option, sequence
    and collection_bounds nodes. The source-owned positives use actual joined wire
    owners. An application member retains its separate application key check and contributes
    its local preimage to the group digest. A reader that still skips any member fails
    the tamper controls of AC-14'
- id: FR-038-AC-146
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; retires ordinary-ungrouped-key-plus-label admission.
    Boolean, Integer, integer_range and reference retain their existing closed-shape
    checks and are never skipped because of a label or cycle. Adding a lone label
    to a correctly keyed ungrouped Int[0, 1000] node while retaining its key, mirroring
    projection and recomputing package identity now refuses invalid_package/stale-node-key
    at its node_id, with or without changing max to 10. The same stale-key mutations
    over Boolean, Integer and reference refuse. Correctly rekeyed and labelled nodes
    are judged by both published p
- id: FR-038-AC-147
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; retires off-cycle-same-label-ordinary-key admission.
    All nodes sharing a retained label participate in its wire-order group preimages,
    even off the names cycle. Adding an off-cycle Set<Int[0, 1000]> node carrying
    the Tree group label and its unchanged ordinary ungrouped key, then mirroring
    projection and refreshing package identity, refuses invalid_package/stale-node-key
    at the least first-pass stale node_id; it cannot admit by verifying the extra
    node as ungrouped. A lone labelled acyclic collection_bounds node with its old
    key also refuses. Dependencies do not supply
- id: FR-038-AC-148
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; replaces former residual admissions with refusal
    regressions. On correctly keyed Tree, keep node keys while separately changing
    collection_bounds max to 5, min to 1, its semantic_type to another group member,
    or Sequence<Tree> body reference to another member; mirror projection and refresh
    package identity. Each refuses invalid_package/stale-node-key under the two-pass
    order. Forge a self-option by starting from a correctly keyed ungrouped Option<Integer>,
    changing its body reference to its own retained node_id and adding a label, retaining
    the old key, mirroring proje
- id: FR-038-AC-149
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN. Group membership, every structural-key visit,
    local-preimage term visits and member/group digest computations are charged before
    work as specified above, including owner-bearing and off-cycle labelled members.
    For a fully valid Tree read, independently account for these visits and find the
    exact smallest total work M that admits; work M-1 returns work-incomplete at the
    first unpaid value with limit, consumed count and pointer, and exposes no package.
    A crate-local counter proves that no later group work occurs after exhaustion
    and that every labelled member contributes
- id: FR-038-AC-150
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; the prior owner gate and SCC skip are retired.
    First authenticate each structural node in ascending retained node-id digest order
    using actual joined owner/declaration, null recursion outside a labelled group,
    or its wire-order group-local preimage with recursion.group equal to the retained
    label. Then recompute every label from all member-local preimage digests in wire
    ordinal order. A stale node key reports invalid_package/stale-node-key at that
    node_id; a stale label reports that refusal at the least retained member, with
    failing groups ordered by least member. A pa
- id: FR-038-AC-176
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'IMPLEMENTED (IR-654 code; external conformance target passes). With `QUIRE_SPECIFICATION_DIR`
    naming the authoritative QSpec checkout, `make conformance-qspec` requires all
    nine `positive-*.json` packages under `proposals/checked-package-v2/fixtures/`:
    `positive-all-families.json`, `positive-clause-operations.json`, `positive-control-operations.json`,
    `positive-nominal-identities.json`, `positive-operation-identities.json`, `positive-recursive-records.json`,
    `positive-two-owners-a.json`, `positive-two-owners-b.json` and `positive-union-nodes.json`.
    Each admits through the production reader wit'
- id: FR-038-AC-123
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Tamper regression (IR-627; implemented for the member-read, scalar-operand,
    state-field body-target and unreferenced rows). Over a package whose model-owned
    field read names a `bounded_domain`/`integer_range` node keyed as `Int[0, 1000]`
    (the field declared `Int[0, 1000]` in the selected domain document), the unmutated
    package admits; the same package with that node's `max` binding changed to `10`,
    and separately to `5000`, the `node_id` kept, `identity_projection` patched and
    `package_id` recomputed through `quire-canonical` in the test, each refuses `invalid_package`/`stale-node-key`
    at that
- id: FR-038-AC-125
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Self-typing and `semantic_type` (IR-627; implemented; an `integer_range`
    node whose `semantic_type` is not the `Integer`-keyed node also has no derivable
    key, even when its key is derived over that `semantic_type`). A `scalar_type`
    or `composite_type` node of a derived shape whose `semantic_type` is another node
    refuses `invalid_package`/`stale-node-key` at its `node_id`, although its key
    is unchanged. A `collection_bounds` node whose `semantic_type` is re-pointed at
    a collection of another element range, an `integer_range` node whose `semantic_type`
    is re-pointed at a node other than the `Int
- id: FR-038-AC-130
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: One derivation (IR-627; implemented by the unit test of `v2/derived_keys.rs`).
    For each of the ten derived shapes the key the admission stage derives from a
    node's own body equals the key `MemberType::node_key` derives from the matching
    member type, and a node whose body is built from the derived key's own preimage
    admits. A test derives both for each shape over the bounds of AC-128 and compares
    them.
- id: FR-038-AC-153
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: A declared record, tuple and function each carries its source owner in
    the node and equal identity projection; QSpec's two-owner packages for `Point`
    and recursive `List` admit with distinct `Point` ids, distinct `List` group labels
    and member ids, and equal builtin `Integer` and application-keyed `three` ids
    under the two owners. An undeclared model declaration and clause function each
    carries its model owner, while an anonymous type, `source_locus` node and application-keyed
    node carry no owner.
- id: FR-038-AC-154
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Omitting a required node or projection owner, inserting `null`, using the
    wrong owner kind, adding `version` to `ModelOwner`, or placing an owner on an
    owner-free node refuses `malformed_wire` at the node or projection object that
    lacks a required owner and at the present `owner` otherwise, before identity validation.
    A well-shaped projection owner differing from its node's owner refuses `invalid_package`/`invalid-value`
    at `/identity_preimage/identity_projection/{i}/owner`, before the owner join and
    even with the package id recomputed over that projection. QSpec's `projection-source-owner-dif
- id: FR-038-AC-155
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'Before structural key re-derivation, a `SourceOwner` absent from `lock.sources`
    refuses `missing_declaration`/`missing-selection` at its node''s `node_id`; a
    source-map region of its `declaration` occurrence naming another source pair refuses
    `invalid_package`/`invalid-value` at that source-map entry even when both pairs
    are lock-selected. A `ModelOwner` whose `identity` is unselected, whose `node`
    names no declaration, or whose `node` names the wrong kind refuses `missing_declaration`/`missing-selection`
    at its node''s `node_id`, even if nothing reaches that node: a relationship cannot
    back `mo'
- id: FR-038-AC-183
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Implemented and verified by TC-048 and TC-226 (IR-680 code). For an unreferenced
    owner-free, ungrouped node in each of the `derived_key` closed-shape and `UngroupedPreimage`
    branches, a retained node id that differs from the production-derived key refuses
    `invalid_package`/`stale-node-key` at the original `node_id`, keeps that original
    typed id in `locus`, and returns the distinct derived typed key through `expected_node_id()`.
    A fresh package whose unreferenced node is rekeyed to that value and whose package
    identity is refreshed admits; one changed preimage member changes the expected
    digest
- id: TC-226
  path: spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md
  role: examined
  excerpt: 'IR-630 owner-bearing, declared and labelled-group controls are PLANNED
    and

    UNRUN. No IR-630 code, runtime tests, Cargo/Kani gates or external conformance

    runs are claimed by this specification amendment.'
```

## Evidence and Limits

Frozen base ee116d87774572bb45633a77f2e267bc3bf77396; exactly two documents, 232 additions and 260 deletions. Targeted Quire validation exited 0; 2/2 grammar-clean, zero grammar findings. All new controls remain PLANNED and UNRUN. No Cargo, Kani, builds, runtime probes, full CI or branch edits were performed. Rust-review checklist loaded before prospective algorithm/API examination; formal Rust source lane and production gap-analysis are inapplicable to this spec-only diff. No applicable AssuranceProfile was found. Jev and Filament executables are unavailable; semantic judgments are manual, not solver verdicts. Actual model/native identity is unavailable in the harness. Canonical private upstream content was read locally and not copied or excerpted here.

The existing reader charges recursion edges after the key stage (crates/quire-contract-model/src/checked_package/v2/mod.rs:1937 and :2000). A Tree read necessarily has such edges; therefore the entire-read boundary can fail after the group work has already completed. Keep the total boundary and add a charge failure inside the group stage with pending work.

Advisor limit: quoin 0.28.3 / quire 0.36.2 (engine 0.50.2), installed executables, quoin advise --repo <frozen-worktree> --json exited 0. Existing AC-124 regex pipes cause its authored method to be extracted as a regex fragment rather than Test. This baseline extraction issue is disclosed to the lead; its advice is not trusted as a method verdict. The ten other selected rows are authored Test, inconclusive false, mismatch false. No method is changed by this reviewer.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 6742c1100de2c5f316db9b672447447e3b27b5fc; round 1, AC-149 and TC-226 add a denied charge within production group work while a later visit or digest remains pending, a sufficient-budget witness of that pending work, a continuation mutant that fails the event assertion and restored immediate propagation that passes. These controls remain PLANNED and UNRUN. |

## Disposition Pass 1

Reviewed 6742c1100de2c5f316db9b672447447e3b27b5fc against its parent 47861328129cc60638217f0fd858402b3815a30b. The original finding text and original scope are unchanged. Only AC-149, TC-226 and custody of the nine original review files were checked; no broad review sweep or runtime/build/gates ran.

**PASS** — FND-001 is fixed in the test specification; no new finding. This is not runtime qualification.

After excerpt (FR-038-AC-149, verbatim):

```text
Separately, select a charge inside group-local preimage or group-digest work on the valid Tree, with at least one later group visit or digest still pending, and set the existing work ceiling so that this selected charge is denied. The result is work-incomplete at that failed charge's pointer with no admitted package; observations of actual group visits and digest computations show none after the denied charge. A genuine mutant that continues the production group walk after this unpaid charge makes the pending visit or digest observable and fails this control; restoring immediate prop
```
