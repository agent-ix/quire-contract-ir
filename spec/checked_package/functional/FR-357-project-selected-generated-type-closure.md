---
id: FR-357
title: "Project the selected generated type and declaration closure"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-353
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: depends_on
  - target: ix://agent-ix/quire-contract-runtime/FR-276
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-251
    type: depends_on
---
# FR-357: Project the selected generated type and declaration closure

## Description

The complete closure remains planned until IR-721 implements it and QSL-1631
supplies the remaining source-owned enum, collection and union witnesses.
Merged QSL-1463 retains final-emission model declaration/field, source-unit
and equality producer/fact rows. A separately testable first IR-721 slice SHALL
read the bounded reachable subset witnessed by those rows, as specified below.
When a consumer prepares one independently selected, admitted monitor root,
Contract IR SHALL expose the complete ordinary
type and declaration closure needed to form the canonical semantic-value
`TypeEnvironment` for that root's four entry bodies. The read SHALL preserve
admitted facts and original producer identities; it SHALL make no new semantic
selection, type judgment, conversion choice or executable plan.

## Inputs

An admitted `CheckedPackageV2`, its selected monitor root and four admitted
entries under [FR-038](./FR-038-consume-checked-package-v2.md), the types reached
through those entries' ordinary expression/function bodies, the corresponding
same-acquisition QSL selected identity rows required by
[FR-354](./FR-354-project-admitted-runtime-context.md), and a finite caller
work allowance. Rows for generated source declarations, members, enum cases,
tuples and units SHALL correspond to the final emitted package identities and
occurrences. A caller-created name, structural key or Exact key alone SHALL NOT
establish producer authority. This requirement defines no new QSL witness
carrier.

For the first subset read, the caller SHALL supply the admitted package, one
admitted selected monitor root, `SelectedExactDeclarationRow` and
`SelectedExactFieldRow` slices from that root's final QSL emission,
same-emission equality application and fact witnesses, and a finite work
allowance. The caller SHALL translate QSL's `EqualityProducerEmissionRow` and
`EqualityFactWitnessRow` into IR's `SelectedEqualityApplicationRow` and
`SelectedEqualityFactRow` without altering identity, occurrence, source map,
owner, fact role or order. IR SHALL accept these IR-owned row types and SHALL
NOT depend on QSL or assume caller construction authenticates Exact payloads.
The caller SHALL keep all rows tied to the same final emission; IR SHALL check
their admitted structural correspondences.

## Outputs

A read-only, complete selected closure of admitted type/declaration facts and
witnessed Exact identities sufficient for the consumer to construct the
canonical `TypeEnvironment`, or a typed refusal with the first failing native
node/occurrence and cause. The output SHALL NOT contain a partially authoritative
environment, an IR-owned environment or a second executable plan. Runtime
`ProgramParts` and its `TypeEnvironment` remain owned by RT and semantic-value;
IR does not depend on RT. A refused projection supplies no material from which
the driver may publish a partial owner.

## Behavior

Starting from the four selected entry bodies, IR SHALL follow admitted type
references and declaration dependencies, including those of reachable ordinary
functions, expressions, members and payloads. It SHALL visit each relevant
node and edge once with an iterative worklist and charge visited identity and
value material under the caller allowance. It SHALL NOT scan or sort the whole
package to discover the selected closure. Repeated references SHALL resolve to
one admitted declaration identity. Unrelated declarations, including unsupported
forms outside the selected closure, SHALL NOT refuse this read.

The closure SHALL cover every reachable source-owned generated record, tuple,
collection and union shape, enum declaration and case, object declaration,
field/member, and declared unit needed by the canonical environment. It SHALL
retain original declaration and member identities, producer order, required or
optional presence, open or closed shape, element and payload types, ancestry,
unit dimensions and admitted conversion facts. An admitted structural node or
display name SHALL NOT substitute for a missing selected Exact identity.
IR SHALL retain [FR-354](./FR-354-project-admitted-runtime-context.md)'s
existing joins and refusal rules for model-selected declarations. IR SHALL
include those declarations only when reachable from the selected closure.

IR SHALL join each required QSL row to the selected source or document, final
admitted structural node, original declaration/member identity, form and
occurrence. Missing, foreign, duplicate, conflicting or stale-after-rewrite
rows, unrepresentable admitted types, unresolved dependencies and exhausted
work SHALL return a named typed refusal before any complete view is exposed.
The read SHALL preserve the native cause and locus of an underlying nominal or
Exact projection failure. It SHALL NOT infer identities, drop a required member,
choose a new unit path, or use an unavailable witness as an empty declaration.

The existing `project_selected_type_declarations` read projects selected model
declarations and source units by scanning all graph nodes, and currently refuses
enum and model-unit selections. Its success SHALL NOT be treated as proof of
this selected generated-source closure. Implementations may reuse its admitted
joins, but SHALL expose the complete selected result through one bounded read
without a second plan, evaluator, type vocabulary or QSL source recheck.

### First bounded witnessed subset read

IR SHALL expose a separate public, read-only bounded projection for model
declarations, fields and equality decisions reachable from the selected
root's four admitted entry bodies. Starting at those entries, it SHALL follow
only admitted ordinary expression/function references, selected model
type/declaration dependencies and equality applications. It SHALL include a
model declaration only when a reached type or declaration depends on it and
preserve its selected document, final structural node, form and original
identity. For each reached record field it SHALL preserve the declaring
record, original field identity, QSL-issued `MemberId`, producer order,
presence and admitted field type. Repeated references SHALL share one
declaration and one equality application identity. This read SHALL NOT claim
complete generated-source closure or construct a `TypeEnvironment`.

For each reached model declaration and field, IR SHALL join the supplied row
against the selected admitted document, full final structural node, admitted
form and original declaration or field identity. For each reached equality,
IR SHALL join its application row by full final application node and exact
occurrence, then each required fact row by selected source-map correspondence,
structural node and key, owner where present, fact role and the admitted
decision's referenced type, unit or enum fact. It SHALL use the FR-353
admitted equality read and preserve its opaque decision or native typed
refusal. IR SHALL NOT derive an Exact identity or equality schedule from a
name, node index, structural key, operand type or missing row. IR's joins
cannot authenticate a caller-supplied opaque Exact value; the caller remains
responsible for supplying the final QSL emission's rows.

The subset result SHALL contain only complete reached, representable
witnessed items. An unsupported graph form or absent witness outside the
reached subset SHALL NOT refuse it. A reached unsupported form, missing,
foreign, duplicate, conflicting or stale-after-rewrite row, unresolved
dependency, unrepresentable type or underlying FR-353 refusal SHALL return a
named typed outcome at the first failing native node, occurrence or member,
with the native cause and no partial authoritative view. IR SHALL use an
iterative worklist and charge each visited node, edge, identity, row and
mapped value before using it under the single caller allowance; exhaustion
SHALL name attempted work and its locus. Equal admitted inputs, rows and
allowance SHALL produce equal results or equal first stops. The public API
SHALL remain in IR's cycle-free model, take no QSL or RT type, and return no
RT `ProgramParts` or IR-owned environment. The driver owns later canonical
`TypeEnvironment` assembly and `ProgramParts` construction.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-357-AC-1 | PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A strictly admitted generated monitor root with four entries yields one complete selected type/declaration closure; records, tuples, collections, unions, enum cases, object fields and source units retain original identities, producer order, presence, shape and member/payload types sufficient to build the canonical `TypeEnvironment`. Shared references do not duplicate declarations. | Test |
| FR-357-AC-2 | PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A missing, foreign, duplicate, conflicting or stale-after-rewrite producer row, unresolved reachable declaration, unrepresentable type or native nominal/Exact refusal returns its typed cause and first locus, with no partial view or invented identity. | Test |
| FR-357-AC-3 | PLANNED/UNRUN (IR-721). A large selected closure amid unrelated graph nodes and unsupported declarations visits only its reachable nodes/edges and charged identity/value material using an iterative worklist. Equal inputs and allowances give equal complete results or first work stops; one below the required work refuses without a partial view. | Test |
| FR-357-AC-4 | PLANNED/UNRUN (IR-721). Public API and dependency inspection finds a bounded IR projection over admitted package facts and original QSL witnesses; it finds no IR-owned `TypeEnvironment`, second plan/evaluator, RT dependency, source reparse or witness fabrication. | Inspection |
| FR-357-AC-5 | PLANNED/UNRUN (IR-721 subset). Given one admitted selected root and its final QSL emission's model declaration/field rows and translated equality rows, the public bounded IR read returns only reached model declarations/fields and opaque FR-353 equality decisions from the four entries, preserving selected document, final structural node, original identities, field order/presence/type, exact equality occurrence and producer facts. Shared references appear once. The result is a subset, not a complete generated-source closure or `TypeEnvironment`. | Test |
| FR-357-AC-6 | PLANNED/UNRUN (IR-721 subset). A missing, foreign, duplicate, conflicting or pre-rewrite declaration, field, equality application or fact row; wrong document, form, owner, role, occurrence or source-map correspondence; unresolved reached dependency; reached unsupported form; and native FR-353 refusal each produce a named typed outcome with first native locus and no partial view or invented identity. An unrelated unsupported declaration or missing witness outside the reached subset leaves the same successful result. | Test |
| FR-357-AC-7 | PLANNED/UNRUN (IR-721 subset). A selected root amid unrelated graph nodes visits only reached nodes, edges and required rows with an iterative worklist; the exact work allowance succeeds and one less refuses with attempted work and locus. Equal inputs and allowance, including a package clone, yield equal results or first stops. | Test |
| FR-357-AC-8 | PLANNED/UNRUN (IR-721 subset). Public API and dependency inspection shows the read takes an admitted package, selected root, IR-owned row slices and a work allowance, performs no QSL/RT dependency or source recheck, and returns neither an IR environment nor `ProgramParts`. A caller adapts QSL-owned equality emission rows without changing their values; IR checks admitted joins but does not claim to authenticate arbitrary caller-created Exact payloads. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict package and
  monitor-root admission. [FR-353](./FR-353-read-admitted-equality-decisions.md)
  owns the separate opaque equality decision read; this requirement does not
  reconstruct or recheck those decisions.
- [FR-354](./FR-354-project-admitted-runtime-context.md) owns the existing
  selected model declaration and nominal joins. Its whole-graph
  `project_selected_type_declarations` output is insufficient for generated
  source closure and remains an independent read.
- QSL FR-251 owns generation of the ordinary entry bodies. Merged QSL-1463
  retains final `emit_plan_root` model declaration/field, source-unit and
  equality producer/fact rows. QSL-1631 remains pending for source-owned enum,
  collection and union witnesses required by AC-1 through AC-4. The first
  subset read uses only currently retained reached rows and does not satisfy
  those complete-closure criteria.
- RT FR-276 and its `ProgramParts` own the final immutable executable program.
  The driver consumes this IR projection to construct the canonical
  semantic-value `TypeEnvironment` passed to RT.
