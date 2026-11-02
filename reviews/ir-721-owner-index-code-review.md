---
id: SR-2026110101298
title: IR-721 anonymous owner target-index code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/owner.rs
  and existing monitor/owner controls
review_set: subset
---

## Summary

No introduced defect found in the scoped owner target-index optimization.

## Verdict

**PASS** — scoped code review clean; final performance/aggregate qualification remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Ticket IR-721, separate performance PR #827. Frozen candidate `03bd5330cbf24cec026d5ee28e940f00a4ca34a6` is verified against local and GitHub PR heads; base is `5197119438115abbd05e4aecff7a905b60d88dce`. Only owner.rs changes. Historical PR #824 review files are untouched; this invocation creates independent review artifacts. No applicable AssuranceProfile exists. Repository Rust conventions, code-review/Rust lane and gap-analysis rules apply; there is no normative/spec delta, so no new spec analyses run.

K denotes distinct raw target IDs of eligible ordinary anonymous functions, C their total candidate references, N all raw nodes. The helper first filters owner-bearing, declared, selected-monitor-entry and non-pure-function nodes, and only considers the two/three-member candidate body shape. It collects parameter/body/measure reference targets; the existing anonymous_body_shape remains the authority for exact order, syntax and target kinds. Only actual candidate references invoke reference_target decoding. Node IDs are already parsed raw IDs: there is no canonicalization, serialization, structural-key expansion, resolution or global key sort in this helper.

For K=0 the selected-map scan is skipped entirely by the explicit targets.is_empty guard. Empty monitor-set membership uses the borrowed raw node ID and does not manufacture a key. The candidate and required owner-admission passes perform bounded raw-label/kind checks; no whole-node index is constructed. For small nonzero K, one scan performs raw target membership against K IDs and inserts only matches into a map of at most K keys. In ordinary key-operation terms the new work is O(N + C log(K+1) + N log(K+1)), rather than inserting every node into an N-entry map. This is a structural analysis, not a claim of constant-time comparison for arbitrarily long raw ID strings.

The candidate collector over-approximates permissible syntax rather than deciding it, so malformed/reordered/clause/extra bodies cannot gain owner exemption. Every target used by a potentially accepted ordinary anonymous body is included, while declared functions already select SourceOwner before the anonymous flag; adding declaration.is_none to that flag changes no refusal. Selected monitor entries keep their existing bypass and plan-stage diagnosis. The retained target scan traverses original wire order and overwrites an equal key, preserving the original map's last-duplicate lookup behavior before ordinary duplicate identity validation. The duplicate-target unit demonstrates both old outcomes: last expression permits the owner stage, while last scalar selects the same MalformedWire owner-stage pointer at node 0. The helper does not change monitor root ordering, general owner joins, identity validation or schedulability.

Focused recorded evidence at this head: two new owner-stage units; 15 monitor groups over 63 by-reference reader recipes; five entry-key filter groups (one existing identity overlap); three default reader controls; 28 owner controls; one real public consumer/read/roundtrip; nine compile-fail probes with the intended E0515/E0451/E0277 diagnostics; fmt/string-edge/workspace/default-model Clippy all pass. The owner-only debug observation is 0.804 ms for 1k irrelevant nodes and 124.818 ms for 100k; it has no timing assertion and does not prove end-to-end reader scaling. Two historical QSpec population/NFD cases were excluded from focused owner controls and are not represented as current-head passing results.

The user-required final-head long_index_chain_admits_on_a_small_stack_with_shared_keys test has NOT run for this candidate. It awaits the separate schedulability global-sort repair owned by the other lane, then a rebase and parent-owned final long-chain/full-target/aggregate/hosted qualification. This review neither modifies that code nor repeats the known long failing baseline run. PR #827 stays DRAFT HOLD until those gates and Luna's merge decision. No Cargo, source edit, commit or merge was performed by this reviewer. Real producer closure, RT/QSL lowering/execution and overall IR-721 completion remain separate open work.

New helper/fixture search found no existing owner-specific partial raw-ID lookup to reuse. No copied schema/vector, persistent tracking machinery, public surface expansion or gate weakening is introduced. Source has no placeholder or unsafe path. The two tests exercise the actual owner stage; the scaling observation deliberately supplies evidence without a fragile timing threshold.

```yaml
scope:
- id: FR-038-AC-153
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: A declared record, tuple and function each carries its source owner in
    the node and retains the complete package value; QSpec's two-owner packages for
    `Point` and recursive `List` admit with distinct `Point` ids, distinct `List`
    group labels and member ids, and equal builtin `Integer` and application-keyed
    `three` ids under the two owners. An undeclared model declaration and clause function
    each carries its model owner, while an anonymous type, `source_locus` node and
    application-keyed node carry no owner.
- id: FR-038-AC-154
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Omitting a required node owner, inserting null, using the wrong owner kind,
    adding version to ModelOwner or placing an owner on an owner-free node refuses
    malformed_wire at the node that lacks a required owner or at the present owner,
    before identity validation. No missing owner is reconstructed from occurrences,
    source map or lock. A shipped complete package value is refused by AC-310 before
    any obsolete projection-owner comparison.
- id: FR-038-AC-155
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: PLANNED/UNRUN (IR-1141). After strict wire and graph-shape validation,
    a `SourceOwner` absent from `lock.sources` refuses `missing_declaration`/`missing-selection`
    at its node's `node_id`; a source-map region of its `declaration` occurrence naming
    another source pair refuses `invalid_package`/`invalid-value` at that source-map
    entry even when both pairs are lock-selected. A `ModelOwner` whose `identity`
    is unselected, whose `node` names no declaration, or whose `node` names the wrong
    kind refuses `missing_declaration`/`missing-selection` at its node's `node_id`,
    except a `model`/`tuple_value_t
- id: FR-038-AC-421
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. The strict reader admits QSpec FR-487 MP-01
    and MP-02 as distinct temporal monitor-plan roots with their exact seventeen ordered
    bindings, State, Inputs, Scope, Results, four ordinary pure-function signatures,
    six-case outcome and exact integer/rational/text/unit premises. The anonymous
    ownerless root has generated source occurrences and semantic_type equal to State;
    generic owner/source-map checks retain precedence. A capacity edit remains admitted;
    the entries' expression bodies are checked without execution and flatten/sequence
    extension MP-32 adds no operation
- id: FR-038-AC-425
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Paired defects select shape before reference
    kind, signatures/outcome before premise range, and range before buffers/state;
    a wrong dependencies set selects the earlier graph-shape invalid_semantic_graph
    at /dependencies. Two defective roots stored opposite structural order choose
    the structurally first root at the post-temporal plan step. The exact code, optional
    cause, primary pointer and root node locus are asserted for each control.
- id: FR-038-AC-433
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. The exact monitor State is now, history_start,
    the ordered buffer_<n> fields, activations, then integer event_position. By-reference
    QSpec MP-74..MP-76 refuse a missing event_position, text-typed event_position
    or placement before activations as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/1,
    retaining the root structural locus. Updated positive State contexts admit with
    no added root binding or public getter. MP-77, MP-57 and MP-69 remain by-reference
    QSL/runtime controls for init zero, one increment per admitted event including
    ba
- id: FR-038-AC-434
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-721 lands. TC-048 reads QSpec entry-key-vectors.json
    EK-01..EK-08 by reference: the published declaration-less function path has parameter
    count, ordered parameter indices, body-root index and optional measure index;
    equal content gives equal paths and a changed body, reordered parameter or measure
    gives a different path. Complete package controls admit genuine declaration-less
    pure_function bodies with exactly ordered parameters/body and an optional final
    measure, including EK-06 with typed parameter references, body-root reference
    and measure reference. Negative controls rej'
bindings:
- test_id: anonymous_lookup_tests::anonymous_owner_lookup_preserves_last_duplicate_target
  ac_id: FR-038-AC-425
  trace: correct
- test_id: anonymous_lookup_tests::owner_schema_irrelevant_nodes_scaling
  ac_id: FR-038-AC-425
  trace: correct
```
