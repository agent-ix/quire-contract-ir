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

Planned until IR-721 implements this read and QSL retains the required
same-acquisition producer witnesses. When a consumer prepares one independently
selected, admitted monitor root, Contract IR SHALL expose the complete ordinary
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

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-357-AC-1 | PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A strictly admitted generated monitor root with four entries yields one complete selected type/declaration closure; records, tuples, collections, unions, enum cases, object fields and source units retain original identities, producer order, presence, shape and member/payload types sufficient to build the canonical `TypeEnvironment`. Shared references do not duplicate declarations. | Test |
| FR-357-AC-2 | PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A missing, foreign, duplicate, conflicting or stale-after-rewrite producer row, unresolved reachable declaration, unrepresentable type or native nominal/Exact refusal returns its typed cause and first locus, with no partial view or invented identity. | Test |
| FR-357-AC-3 | PLANNED/UNRUN (IR-721). A large selected closure amid unrelated graph nodes and unsupported declarations visits only its reachable nodes/edges and charged identity/value material using an iterative worklist. Equal inputs and allowances give equal complete results or first work stops; one below the required work refuses without a partial view. | Test |
| FR-357-AC-4 | PLANNED/UNRUN (IR-721). Public API and dependency inspection finds a bounded IR projection over admitted package facts and original QSL witnesses; it finds no IR-owned `TypeEnvironment`, second plan/evaluator, RT dependency, source reparse or witness fabrication. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict package and
  monitor-root admission. [FR-353](./FR-353-read-admitted-equality-decisions.md)
  owns the separate opaque equality decision read; this requirement does not
  reconstruct or recheck those decisions.
- [FR-354](./FR-354-project-admitted-runtime-context.md) owns the existing
  selected model declaration and nominal joins. Its whole-graph
  `project_selected_type_declarations` output is insufficient for generated
  source closure and remains an independent read.
- QSL FR-251 owns generation of the ordinary entry bodies. Its current
  `emit_plan_root` result does not retain all final same-acquisition equality
  and source-unit witness rows. QSL must retain those rows for IR's joins;
  their shape and carrier remain QSL-owned.
- RT FR-276 and its `ProgramParts` own the final immutable executable program.
  The driver consumes this IR projection to construct the canonical
  semantic-value `TypeEnvironment` passed to RT.
