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

The complete closure remains planned under IR-721. QSL-1630 retains the complete
atom, guard and helper bodies in its plan identity; QSL-1631 supplies the
final-emission source enum declaration/member rows. Collections are structural
Exact `ValueType::Collection` values and require no declaration identity row.
Source union admission/projection remains the existing IR-1323 prerequisite,
followed by the matching final-emission producer handoff. Generated source
record/member identity handoff also remains unavailable; model-selected rows
are not evidence for those source declarations. Merged QSL-1463 retains
final-emission model declaration/field, source-unit and equality producer/fact
rows. The implemented first IR-721 slice SHALL
read an explicitly selected, bounded set of witnessed model declaration/field
and equality facts, as specified below. It does not traverse a monitor root.
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
carrier. Existing final-emission tuple, source enum declaration/member,
model enum declaration/member, source-unit, model declaration/field and equality
rows retain their distinct admitted ownership channels. They SHALL NOT be
presented as a complete witness set when a reachable source record/member or
union has no authentic handoff. In that case the complete-closure read SHALL
return the native typed unavailable-witness or projection refusal; it SHALL
NOT infer source identity from the node's structural key.

For the first selected-fact read, the caller SHALL supply an admitted package;
an explicit finite selection of full admitted model declaration node IDs,
record-node/original-field identities and equality application node/occurrence
pairs; `SelectedExactDeclarationRow`, `SelectedExactFieldRow` and
`SelectedSourceUnitRow` slices from one final QSL emission; same-emission
equality application and fact witnesses; and a finite work allowance. The
caller SHALL translate QSL's `EqualityProducerEmissionRow` and
`EqualityFactWitnessRow` into IR's `SelectedEqualityApplicationRow` and
`SelectedEqualityFactRow` without altering identity, occurrence, source map,
owner, fact role or order. QSL's public `Emission::equality_witness` adapter
provides this typed handoff; no access to private producer witness enums is
required. IR SHALL accept these IR-owned row types and SHALL
NOT depend on QSL or assume caller construction authenticates Exact payloads.
The caller SHALL keep all rows tied to the same final emission; IR SHALL check
their admitted structural correspondences. The selected IDs SHALL NOT assert
entry reachability or producer authority by themselves.

## Outputs

A read-only, complete selected closure of admitted type/declaration facts and
witnessed Exact identities sufficient for the consumer to construct the
canonical `TypeEnvironment`, or a typed refusal with the first failing native
node/occurrence and cause. The output SHALL NOT contain a partially authoritative
environment, an IR-owned environment or a second executable plan. Runtime
`ProgramParts` and its `TypeEnvironment` remain owned by RT and semantic-value;
IR does not depend on RT. A refused projection supplies no material from which
the driver may publish a partial owner.

The first selected-fact read SHALL return a complete view of only the
explicitly selected, representable witnessed model and equality facts, or a
typed refusal. That view SHALL NOT certify that any monitor root, entry body
or generated-source declaration closure is complete.

## Behavior

Starting from the four selected entry bodies, IR SHALL follow admitted type
references and declaration dependencies, including those of reachable ordinary
functions, expressions, members and payloads. It SHALL visit each relevant
node and edge once with an iterative worklist and charge visited identity and
value material under the caller allowance. It SHALL NOT scan or sort the whole
package to discover the selected closure. Repeated references SHALL resolve to
one admitted declaration identity. Unrelated declarations, including unsupported
forms outside the selected closure, SHALL NOT refuse this read.

The closure SHALL cover every reachable source-owned generated record, tuple
and union declaration, structural collection shape, enum declaration and case,
object declaration, field/member, and declared unit needed by the canonical environment. It SHALL
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

### First bounded selected-fact read

IR SHALL expose a separate public, read-only bounded projection for the
explicitly selected model declaration nodes, record fields and equality
application/occurrence pairs in one admitted package. It SHALL verify each
selected ID against that package and preserve the selected document, final
structural node, admitted form and original declaration or field identity.
For a selected record field it SHALL preserve the declaring record,
QSL-issued `MemberId`, producer order, presence and admitted field type.
Repeated selected IDs SHALL produce one item. The read SHALL inspect only the selected facts and the
admitted facts needed to validate them; it SHALL NOT traverse the selected
monitor root, its four entry signatures or bodies, or infer a reachable
declaration closure. QSL's current generated PlanRoot reaches source-owned
records and an `Outcome` union for which the authentic final declaration/member
handoff is not yet available. Collections remain structural, and delivered
source enum rows do not fill the source record or union gap. Therefore this
selected-fact read SHALL NOT report that root's closure as complete or
construct a `TypeEnvironment`.

For each selected model declaration and field, IR SHALL join the supplied row
against the selected admitted document, full final structural node, admitted
form and original declaration or field identity. For each selected equality,
IR SHALL join its application row by full final application node and exact
occurrence, then each required fact row by selected source-map correspondence,
structural node and key, owner where present, fact role and the admitted
decision's referenced type, unit or enum fact. It SHALL use the FR-353
admitted equality read and preserve its opaque decision or native typed
refusal, passing the remaining caller allowance to that read and charging its
reported work within the same limit. IR SHALL NOT derive an Exact identity or equality schedule from a
name, node index, structural key, operand type or missing row. IR's joins
cannot authenticate a caller-supplied opaque Exact value; the caller remains
responsible for supplying the final QSL emission's rows.

The selected-fact result SHALL contain only complete selected, representable
witnessed items. An unsupported graph form or absent witness outside the
explicit selection SHALL NOT refuse it. A selected unsupported form, missing,
foreign, duplicate, conflicting or stale-after-rewrite row, unresolved
selected dependency, unrepresentable selected type or underlying FR-353
refusal SHALL return a named typed outcome at the first failing native node,
occurrence or member, with the native cause and no partial authoritative
view. IR SHALL charge each selected ID, inspected admitted fact, identity and
mapped value before use. It MAY locate selected witnesses by indexing or
searching the supplied full-emission row slices, but SHALL charge every
inspected row and index entry, including unrelated rows, under the same
caller allowance before use. It SHALL NOT scan unrelated graph nodes to
discover selected facts. Exhaustion SHALL name attempted work and the first
uncharged row or graph locus. Equal admitted inputs, rows and
allowance SHALL produce equal results or equal first stops. The public API
SHALL remain in IR's cycle-free model, take no QSL or RT type, and return no
RT `ProgramParts` or IR-owned environment. The driver owns later canonical
`TypeEnvironment` assembly and `ProgramParts` construction; the selected-fact
result alone SHALL NOT authorize either construction.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-357-AC-1 | PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A strictly admitted generated monitor root with four entries yields one complete selected type/declaration closure; records, tuples, collections, unions, enum cases, object fields and source units retain original identities, producer order, presence, shape and member/payload types sufficient to build the canonical `TypeEnvironment`. Shared references do not duplicate declarations. | Test |
| FR-357-AC-2 | PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A missing, foreign, duplicate, conflicting or stale-after-rewrite producer row, unresolved reachable declaration, unrepresentable type or native nominal/Exact refusal returns its typed cause and first locus, with no partial view or invented identity. | Test |
| FR-357-AC-3 | PLANNED/UNRUN (IR-721). A large selected closure amid unrelated graph nodes and unsupported declarations visits only its reachable nodes/edges and charged identity/value material using an iterative worklist. Equal inputs and allowances give equal complete results or first work stops; one below the required work refuses without a partial view. | Test |
| FR-357-AC-4 | PLANNED/UNRUN (IR-721). Public API and dependency inspection finds a bounded IR projection over admitted package facts and original QSL witnesses; it finds no IR-owned `TypeEnvironment`, second plan/evaluator, RT dependency, source reparse or witness fabrication. | Inspection |
| FR-357-AC-5 | Implemented (IR-721 selected facts; IR-owned fixture). Given an admitted package, explicit IDs for one selected model declaration, its selected record field and one equality application/occurrence, and matching IR-owned same-emission rows, the public bounded read returns exactly those complete facts. It preserves selected document, final structural node, original identities, field order/presence/type, exact equality occurrence and opaque FR-353 decision. No monitor-entry traversal or generated-source closure is claimed. | Test |
| FR-357-AC-6 | Implemented (IR-721 selected facts). A missing, foreign, duplicate, conflicting or pre-rewrite selected declaration, field, equality application or fact row; wrong document, form, owner, role, occurrence or source-map correspondence; selected unsupported form or unrepresentable type; and native FR-353 refusal each produce a named typed outcome with first native locus and no partial view or invented identity. An unrelated unsupported graph form or absent witness outside the explicit selection leaves the same successful result. Duplicate selected IDs produce one item. | Test |
| FR-357-AC-7 | Implemented (IR-721 selected facts). The bounded read inspects the explicitly selected admitted facts without traversing monitor entries or scanning unrelated graph nodes. Locating selected witnesses in full-emission row slices charges every inspected row and index entry, including unrelated rows; adding unrelated rows either raises required work or causes a charged first-row-locus stop at the former allowance. The exact allowance succeeds and one less refuses with attempted work and first uncharged locus. Equal inputs and allowance, including a package clone, yield equal results or first stops. | Test |
| FR-357-AC-8 | Implemented (IR-721 selected facts; API and dependency inspection). Public API and dependency inspection shows the read takes an admitted package, explicit selected model declaration/field IDs and equality application/occurrence pairs, IR-owned row slices and a work allowance, performs no QSL/RT dependency or source recheck, and returns neither an IR environment nor `ProgramParts`. A caller adapts QSL-owned equality emission rows without changing their values; IR checks admitted joins but does not claim to authenticate arbitrary caller-created Exact payloads. | Inspection |
| FR-357-AC-9 | Implemented (`quire-integration` PR #47; explicit selected-fact scope only). A `quire-integration` test obtains a real final `PlanRoot.emission` through QSL, selects admitted IDs from its available model declaration/field rows and one equality application/occurrence pair, translates QSL equality rows without changing their values, and exercises the public IR selected-fact read on those explicit IDs. It verifies successful joins for representable selected model facts and an opaque FR-353 equality decision or its native typed missing-witness result for the selected equality. The executed PR #47 control observed opaque equality success; the missing-witness alternative remains source-reviewed rather than an observed result. It does not traverse four monitor entries or claim a complete generated-source closure, `TypeEnvironment` or RT `ProgramParts`, and it adds no QSL dependency to IR. | Test |

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
  equality producer/fact rows. QSL-1630 and QSL-1631 are delivered: the former
  binds the complete checked bodies to plan identity; the latter retains source
  enum declaration/member rows. Collections need no declaration row. Source
  union identity admission/projection is owned by existing IR-1323 and must
  precede its matching final producer rows; authentic generated source
  record/member identity handoff is also required. No delivered enum or model
  row substitutes for either missing channel. AC-1 through AC-4 remain planned
  until complete closure and its real producer handoff are implemented and
  exercised. The first selected-fact read does not satisfy those criteria.
  `quire-integration` owns
  AC-9's merged, executed real-emission QSL-to-IR integration test using the
  available IR selected-fact API; IR owns AC-5's explicitly selected fixture.
- RT FR-276 and its `ProgramParts` own the final immutable executable program.
  The driver consumes the future complete-closure IR projection to construct the canonical
  semantic-value `TypeEnvironment` passed to RT.
