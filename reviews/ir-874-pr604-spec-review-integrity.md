---
id: SR-604104
title: "IR-874 PR 604 spec-review/integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; PR #604 "
review_set: subset
---

## Summary

Reviewed the two-file IR-874 PR #604 spec diff against QSpec FR-375-AC-26/27, FR-432-AC-16..19, FR-208-AC-18 and FR-154-AC-32. No AssuranceProfile applies.

## Verdict

**PASS** — no separate finding under this method. Full FR-038-AC-325 remains open for implementation and conformance dependencies.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FR-038-AC-325 (examined): The protocol step compares every present refinement `class_catalog` against the complete concrete and abstract classes reconstructed from selected model declarations and the admitted local or directly selected dependency protocol graph, including distinct `fork` and `join` classes on one parallel node, checked binder types and source-default memory classes, even with no step row. The concrete refinement `semantic_type` resolves to a selected package `model_root`, while an owned clause's separate `owner_model` resolves to an object-type node of a model owned by that selected declaration; classes from that clause count even without a step row. Foreign, unselected and wrong-kind owners cannot complete the selected inventory. A source subject with `sc`, `tso` and `ra` defaults admits three distinct `memory drain` entries with `drain: store-buffer-flush`, keyed by (`class`, `memory_model`) and ordered `ra`, `sc`, `tso`, followed by `memory flush` for `tso`; repeated selection of one default adds no entry. An `on each (x: T)` clause admits `activate <protocol>` associated with that clause node and binder types `[T]` for non-record `T`, or the effective fields of record `T` in source supertype order then own declaration order, retaining a diamond field once and placing a redefining field where the hidden field stood; reversing source supertype order changes that binder order. An `on origin` clause admits no activate entry. An ambiguous same-name inherited field without redefinition refuses `ambiguous_declaration`/`ambiguous-name` at the catalog binder record's `node` right side. A wrong subject, clause/node association, or wrong or reordered binder signature refuses `invalid_model_binding`/`malformed-declaration` at the catalog entry; a missing or extra class refuses `invalid_semantic_graph` at the catalog array or entry; duplicate or misordered catalog keys refuse at the earlier refinement canonical-order check, with memory keys compared by class then model. An absent required or forbidden present catalog refuses `invalid_semantic_graph` at its member. A protocol-body defect reports before this graph join; a refinement-step row/catalog defect reports before either. A later request memory override does not change checked-package catalog facts or package identity.
- FR-038 catalog ownership and ordering (examined): Where one subject selects multiple source-default models, QSpec TC-310
requires one `memory drain` entry per distinct selected model, keyed by
(`class`, `memory_model`), even when two `parallel` controls select the same
model. A declared class is keyed by its class spelling alone. Entries order
by the UTF-8 bytes of `class`, then by the UTF-8 bytes of `memory_model` for
equal memory-class spellings: drains for `ra`, `sc`, `tso`, followed by the
`tso` flush. A duplicate key or an out-of-order entry refuses
`invalid_semantic_graph` at the later duplicate or first out-of-order entry
in the earlier refinement canonical-order step. A missing or extra distinct
memory class refuses at the later catalog graph join. The reader SHALL
admit the complete ordered multi-default catalog, independently of step rows.

For a protocol clause activated `on each (x: T)`, the reader SHALL associate
the `activate <protocol>` class with the protocol clause node itself, not the
trigger node. Its `binder_types` SHALL contain `[T]` when `T` is not a record;
for a record `T`, it SHALL contain one key per effective field in order. The
effective order walks the source `supertypes` array in authored order, then
the type's own `fields` in authored order. A diamond field appears once at
its first position; a redefining field replaces the hidden field at that
position, with hiding applied once over the whole walk. Changing source
supertype order may therefore change `binder_types`, even when the same
fields and nodes remain. An ambiguous same-name field without a redefinition
refuses `ambiguous_declaration`/`ambiguous-name` at the catalog entry's
`node` right side when its binder record is requested. A clause activated
`on origin` contributes no `activate` entry. A wrong clause association or
wrong or reordered binder type refuses `invalid_model_binding`/
`malformed-declaration` at the catalog entry; an omitted required entry
refuses `invalid_semantic_graph` at the catalog array, and an extra entry
for `on origin` refuses that code at the extra entry.

- TC-048 catalog cases 12-14 (examined): Also retain QSpec TC-310 `CC-12`'s one-subject multi-default case with
    distinct `memory drain` entries for `sc`, `tso` and `ra` and a `tso`
    `memory flush` entry. Require admission with the three drains in
    `ra`, `sc`, `tso` order followed by the flush, including when two
    `parallel` controls select the same default; the repeated model contributes
    one entry. Rekey and refuse a duplicate (`memory drain`, `memory_model`)
    pair or a declared class spelling at the later duplicate catalog entry,
    and a reordered pair at the first out-of-order entry, in the earlier
    refinement canonical-order step. Omit a distinct default or add an
    unselected one and require the later catalog graph-join refusal.
13. Build public-writer packages for selected concrete and abstract clauses
    activated `on each (x: T)` with non-record `T` and with record `T`.
    Require `activate <protocol>` to name the clause node on the concrete
    side and its clause `RefinementKey` on the abstract side, with `[T]` for
    non-record `T` and one binder type per effective record field. A clause
    activated `on origin` contributes no activate entry. Rekey independently
    with the trigger node substituted for the clause, another clause node,
    a wrong binder type, reversed binder order, a missing activate entry
    and an extra origin entry. Require respectively
    `invalid_model_binding`/`malformed-declaration` at the entry for the
    wrong node or binder, `invalid_semantic_graph` at the catalog array for
    omission, and that code at the extra entry for origin (FR-038-AC-325;
    QSpec FR-375-AC-27, TC-310).
14. For record activation binders, build `Base { a: Boolean; b: Int }`,
    `Mid { c: Int }` and `Leaf { d: Boolean }` with the source inheritance
    chain `Base`, `Mid`, `Leaf`; require binder type keys corresponding to
    `a`, `b`, `c`, `d` in that order. Reverse `Base`'s field declaration order
    and require `b`, `a`, `c`, `d`. For `P { p: Boolean }`,
    `Q { q: Int }` and `Both { r: Boolean }`, compare source
    supertypes `P`, `Q` against `Q`, `P` and require `p`, `q`, `r` against
    `q`, `p`, `r`, irrespective of declaration node-key or IR `types` order.
    In a diamond `Top { t }`, `L { l }`, `R { r }`, `Bot { z }`, require
    `t`, `l`, `r`, `z` with `t` once. Redefine `Base::a` in `Leaf` and
    require the new type of `a` at its inherited position; in a diamond,
    redefine `Top::t` through `L` and require the replacement once at `t`'s
    position. A same-name field inherited without an explicit redefinition
refuses `ambiguous_declaration`/`ambiguous-name` at the catalog entry's
    `node` right side when its binder record is requested. A wrong or
    reordered binder type in an otherwise unambiguous case refuses
    `invalid_model_binding`/`malformed-declaration` at that entry
    (FR-038-AC-325; QSpec FR-208-AC-18, FR-154-AC-32,
    FR-432-AC-16 through FR-432-AC-19).



## Coverage

QSpec origin/main was read directly. This is spec-only: no Cargo, Make or build checks were run. The memory catalog key/order, activation node, on-origin exclusion, source supertype order, diamond deduplication and redefinition position match the named QSpec rules. This review does not claim FR-038-AC-325 implementation completion.
