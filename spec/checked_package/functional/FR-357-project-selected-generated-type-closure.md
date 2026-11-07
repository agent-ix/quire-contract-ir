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

IR-1354 implements the public atomic closure over one admitted selected monitor
using one caller-owned native work and storage budget. QSL-1630 retains the complete
atom, guard and helper bodies in its plan identity; QSL-1631 supplies the
final-emission source enum declaration/member rows. QSL-1723 and QSL-1727
now supply final-emission source union and source record/member rows. Merged
QSL-1463 retains model declaration/field, source-unit and equality
producer/fact rows. Collections are structural Exact
`ValueType::Collection` values and require no declaration identity row.
The IR-side union and source-record joins admit strict IR-owned fixture rows;
they do not authenticate caller-created opaque Exact values. A genuine
same-final-emission QSL-to-IR producer acceptance test in `quire-integration`
remains required by AC-11 for real producer acceptance. The implemented first IR-721 slice
reads an explicitly selected, bounded set of witnessed model declaration/field
and equality facts, as specified below. It does not traverse a monitor root.
IR-1353 adds the package-private bounded read of reached model and unit facts
from an admitted monitor root, reused by the public closure.
When a consumer prepares one independently selected, admitted monitor root,
Contract IR SHALL expose the complete ordinary
type and declaration closure needed to form the canonical semantic-value
`TypeEnvironment` for that root's four entry bodies. The read SHALL preserve
admitted facts and original producer identities; it SHALL make no new semantic
selection, type judgment, conversion choice or executable plan.

## Inputs

The `CheckedMonitorPlanRef::project_selected_type_closure` read SHALL
take one borrowed plan from an admitted `CheckedPackageV2`, one finite caller
owned `NativeJoinBudget` for work and storage and one IR-owned
`SelectedMonitorClosureRows` input containing the selected producer rows from
the same final QSL emission. The plan itself supplies the State, Inputs,
Scope and Results type nodes and the four ordered `init`, `on_event`,
`on_watermark` and `on_gap` entry signatures (parameter types in signature
order and result type) and bodies under
[FR-038](./FR-038-consume-checked-package-v2.md); a caller-provided list of
root or entry IDs SHALL NOT replace those admitted handles. The row input
SHALL retain the distinct source record/member, source union/member and
payload, tuple, source enum declaration/member, structural source collection,
model declaration/field and enum, source-unit, dimension/conversion and
equality fact channels required by reachable types, and the required
same-emission authority row of
[FR-361](./FR-361-bind-monitor-authority-and-runtime-layout-context.md).
A structural source collection row SHALL carry `key_node: Option<CheckedNodeId>`:
the emitted key type node of a map, which the base references before the value,
and `None` for every other collection. A missing, foreign or misplaced key
node SHALL refuse `SelectedMonitorTypeClosureErrorCode::CollectionKey` at the
collection node. Each row SHALL preserve
its final emitted structural node, selected owner or source and original
declaration/member identity where those exist, and QSL-issued opaque Exact
identity and value where that channel defines one. An anonymous structural
collection row has no invented owner or declaration identity.
IR SHALL check admitted structural correspondence but SHALL NOT authenticate
arbitrary caller-created opaque Exact values. A caller-created name,
structural key or Exact key alone SHALL NOT establish producer authority.
Where a reachable channel lacks a required row, the read SHALL return its
native unavailable-witness or projection refusal. The QSL producer SHALL
supply every row from one final emission; QSL-to-IR same-emission adapter
acceptance is owned by `quire-integration`, with QSL producer types remaining
outside IR's dependency graph.

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

A successful `SelectedMonitorTypeClosure` SHALL retain the selected monitor
root; State, Inputs, Scope and Results; the four entry signatures and complete
ordinary bodies in admitted order; every reachable type/declaration fact in
producer order within its declaration; original declaration/member identities;
QSL-issued opaque Exact keys, member bindings and full value types; and the
exact charged work. The selected facts SHALL be sufficient for a downstream
consumer to construct the canonical semantic-value `TypeEnvironment`, without
requiring a package-wide scan, source recheck or invented identity. The
result SHALL deduplicate shared declarations by their admitted structural
identity while preserving member and payload order. On failure, a
`SelectedMonitorTypeClosureError` with a closed
`SelectedMonitorTypeClosureErrorCode` cause, first native
node/edge/row/occurrence and charged work SHALL replace the entire view; no
partial material from which a driver may publish an owner is exposed. The
output SHALL NOT contain an IR-owned environment or a second executable plan.
Runtime `ProgramParts` and its `TypeEnvironment` remain owned by RT and
semantic-value; IR does not depend on RT.

The first selected-fact read SHALL return a complete view of only the
explicitly selected, representable witnessed model and equality facts, or a
typed refusal. That view SHALL NOT certify that any monitor root, entry body
or generated-source declaration closure is complete.

## Behavior

### Complete selected monitor closure read

The one bounded read on `CheckedMonitorPlanRef` SHALL seed a fixed traversal
from State, Inputs, Scope, Results and the four entry signatures and bodies in
`init`, `on_event`, `on_watermark`, `on_gap` order. It SHALL use an iterative
worklist and visited sets for full admitted structural node IDs and directed
edge roles. For each reached node it SHALL inspect only selected graph
dependencies needed by its ordinary body, the node's `semantic_type`, every
typed-term node ID including value/type IDs omitted by
`CheckedTerm::reference_targets`, and the admitted declaration, member and
payload type edges needed for that node's canonical environment facts. The
typed-term visitor SHALL itself be iterative; a nested term SHALL NOT grow
the host stack or bypass the caller allowance. The read SHALL NOT scan or
sort all graph nodes to discover the closure. It SHALL resolve each shared
node and edge once, and unrelated unsupported declarations SHALL NOT affect
the result.

For every reached type, the closure SHALL retain complete producer-ordered
source record members with original field identity, name, presence, final
type node and opaque Exact value type; tuple elements; structural collection
kind, element and bounds, and for a map its key and value type nodes and its
complete Exact `ValueType::Map`; sealed union members with settled-key ownership,
ordered final payload node IDs and opaque payload value types; source and
model enum declarations and cases; selected model records, objects, fields,
operations and ancestry; and source units, dimensions and admitted conversion
facts. It SHALL retain open or closed shape and every required/optional
member without dropping an unread field. It SHALL follow reachable ordinary
source functions and model declarations by their admitted dependencies, not
by name or graph position. IR SHALL apply the existing
[FR-354](./FR-354-project-admitted-runtime-context.md) model joins and native
nominal/Exact projection rules only to reached declarations. The complete
result SHALL preserve original declaration/member identities alongside
QSL-issued opaque Exact identities; a structural ID or display name SHALL
NOT replace a missing opaque identity.

The read SHALL use one shared finite native work and storage budget from seed inspection through
graph and term traversal, selected row/index joins, native Exact and nominal
projection, admitted equality decision reads, and output identity/value material. It SHALL charge every
inspected node, directed edge, row/index entry, member, payload position and
identity/value byte before using or copying it. A nested bounded read SHALL
receive the same budget and charge against its remaining allowance; a separate
reset meter or unreported successful charge SHALL NOT enlarge the allowance.
The external driver SHALL debit its selection and adapter work and logical
storage before performing that work or allocation through the public
`NativeJoinBudget::accounted_charge` API, then pass that same budget into the
native read. A new phase MAY use `resume` without restoring spent allowance.
Exhaustion SHALL identify the first uncharged native locus, attempted charge and
the work already charged. Equal admitted inputs, same-emission rows and limit
SHALL yield equal complete results or equal first stops, including after a
package clone. The read SHALL expose no partial view on refusal.

The read SHALL join each reached producer row to the final admitted
structural node, selected source or model document, owner, original qualified
declaration/member identity, form, payload/type references and occurrence
where applicable. Missing, foreign, duplicate, conflicting or stale rows;
unrepresentable reached types; and native nominal or Exact refusal SHALL
retain their typed cause and first native locus. A reached cross-package
reference with no admitted resolution rule SHALL return a typed unresolved
dependency refusal at that reference; it SHALL NOT search another package,
borrow a same-named declaration or omit the edge. Rows outside the reached
closure SHALL NOT be validated or refuse the read; a selected lookup SHALL
charge their inspected row/index positions and exhaust at the first such
uncharged position. The read SHALL NOT
reparse QSL source, create a QSL or RT dependency, construct a
`TypeEnvironment`, or lower an executable plan.

For a QSL-selected source record, IR SHALL accept one complete row carrying
the final admitted record node, selected source authority and identity,
original qualified declaration, QSL-issued opaque record key, and every
declared member in producer order. Each member row SHALL retain its original
declared field identity, final admitted binding name, declared presence,
final emitted type node and QSL-issued opaque field value type. IR SHALL
check the row against the selected source, admitted owner, declaration,
record form, complete ordered member census, binding names and type-node
references. It SHALL not infer a source `MemberId` or compare the opaque
record key and value types against invented IR values. An absent selected row
or a mismatched, duplicate or foreign row SHALL refuse without a partial
source record view. This row join and single-type projection alone SHALL not
certify monitor-entry reachability or the complete selected closure.

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

### Reached model and unit prerequisite

Before the complete public closure is available, an internal bounded read MAY
consume only the node IDs reached by the admitted monitor plan's selected graph
walk. It SHALL derive its model selection from those admitted nodes, not a
caller-supplied declaration list, a display name or a package-wide discovery
scan. It MAY inspect the complete admitted members of a reached declaration
and charged selected row or index entries. For each reached model record or
object it SHALL
retain the selected document and original declaration identity, QSL-issued
Exact identity, complete effective fields in admitted order with original
field identities, presence and full types, object ancestry, and the object's
own operations with ordered parameters, result, admitted effect and
provenance. It SHALL follow selected field, parameter, result and ancestry
type dependencies through an iterative visited-node worklist so a nested
declaration cannot be silently omitted. Shared declarations SHALL appear
once. A reached unsupported type or missing identity SHALL refuse at its
native locus; an unrelated selected declaration SHALL not refuse.

For a reached source unit, the prerequisite SHALL retain its QSL-issued
declared `UnitId`, original qualified declaration, admitted dimension and
conversion-target closure, exact scale and offset. For a reached model unit,
it SHALL join the selected document's admitted original unit, dimension,
target, scale and offset to its final model-owned unit node and the QSL-issued
`SelectedExactDeclarationRow` whose form is
`SelectedExactDeclarationForm::DeclaredUnit` and whose identity is
`SelectedExactIdentity::DeclaredUnit`; it SHALL not assign a source
declaration or invent a dimension key. Unit targets and dimension terms SHALL
be followed only from reached units, with a visited set and a typed unresolved
dependency refusal. A reached model variant SHALL return an unavailable
shape-witness refusal until QSL supplies its model `EnumShape` and
`VariantId` facts; source enum rows SHALL not stand in for model rows.

The prerequisite SHALL charge the selected graph walk, every inspected
selected row or index entry, followed member, operation, unit and dimension
edge, nested Exact or nominal read, and copied identity or value through one
finite allowance. It SHALL return its charged work with only complete reached
facts or a typed first refusal and no partial result. Equal admitted input,
rows and allowance, including a package clone, SHALL agree. It SHALL remain
package-private, create no `TypeEnvironment`, and SHALL not claim the complete
source/type closure or same-emission producer acceptance.

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
declaration closure. Merged IR PR #906 exposes selective bounded callable
reads for admitted monitor entries. PR #907 also permits direct admitted Value
bodies for ordinary source functions; monitor entries still require expression
bodies. These reads preserve admitted handles and charged work, without
certifying complete monitor type closure. QSL's generated PlanRoot reaches
source-owned records and an `Outcome` union. Their QSL producer rows are now
available, but this selective read does not traverse their four-entry closure;
the real same-final-emission closure adapter test remains pending in
`quire-integration`. Therefore this selected-fact read SHALL NOT report that
root's closure as complete or construct a `TypeEnvironment`.

For each selected model declaration and field, IR SHALL join the supplied row
against the selected admitted document, full final structural node, admitted
form and original declaration or field identity. For each selected equality,
IR SHALL join its application row by full final application node and exact
occurrence, then each required fact row by selected source-map correspondence,
structural node and key, owner where present, fact role and the admitted
decision's referenced type, unit or enum fact. It SHALL use the FR-353
admitted equality read and preserve its opaque decision or native typed
refusal, passing the caller's same mutable `NativeJoinBudget` to that read.
The equality read SHALL charge its own inspected decisions, owned identities,
row material and typed refusal payloads against cumulative work and logical
storage before inspection or allocation. IR SHALL NOT derive an Exact identity or equality schedule from a
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
| FR-357-AC-1 | Implemented (IR-1354; TC-470). One bounded read of an admitted `CheckedMonitorPlanRef` seeds State, Inputs, Scope, Results and all four ordered entry signatures and bodies, then returns exactly the reachable complete type/declaration and admitted equality closure. Source records, tuples, structural collections, sealed unions, source/model enum cases, model objects/fields and units/dimensions/conversions retain original identities, QSL-issued opaque Exact identities, producer order, presence, shape and member/payload types sufficient for the canonical `TypeEnvironment`; shared references yield one declaration. An IR-owned strict fixture proves the traversal separately from AC-11's real QSL producer handoff. | Test |
| FR-357-AC-2 | Implemented (IR-1354; TC-470). A missing, foreign, duplicate, conflicting or stale producer row; changed member/payload type node; unresolved reached declaration; cross-package reference without an admitted resolution rule; unrepresentable reached type; or native nominal, Exact or equality refusal returns its typed cause and first node/edge/row/occurrence locus, with charged work, no partial view and no invented identity. An unrelated unsupported graph node or absent row outside the reached closure does not refuse; a selected row search can still exhaust at an inspected unrelated row position. | Test |
| FR-357-AC-3 | Implemented (IR-1354; TC-470). A large selected closure amid unrelated graph nodes visits only reached graph, typed-term, semantic-type and declaration edges using iterative worklists and visited node/edge sets. A deep term does not grow the host stack. One shared native work and storage budget charges every inspected node/edge, row/index, member/payload position and identity/value byte, including nested Exact, nominal and equality reads and output; the exact allowance succeeds and one less names the first uncharged locus. Equal inputs and allowances, including a package clone, give equal complete results or first stops. | Test |
| FR-357-AC-4 | Implemented (IR-1354; TC-470 and FR-019 exact inventory). Static API and dependency analysis finds one bounded `CheckedMonitorPlanRef` projection over admitted package facts and IR-owned selected producer rows, with no IR-owned `TypeEnvironment`, second plan/evaluator, QSL or RT dependency, source reparse or witness fabrication. The root exports appear in FR-019's exact implemented inventory. | Analysis |
| FR-357-AC-5 | Implemented (IR-721 selected facts; IR-owned fixture). Given an admitted package, explicit IDs for one selected model declaration, its selected record field and one equality application/occurrence, and matching IR-owned same-emission rows, the public bounded read returns exactly those complete facts. It preserves selected document, final structural node, original identities, field order/presence/type, exact equality occurrence and opaque FR-353 decision. No monitor-entry traversal or generated-source closure is claimed. | Test |
| FR-357-AC-6 | Implemented (IR-721 selected facts). A missing, foreign, duplicate, conflicting or pre-rewrite selected declaration, field, equality application or fact row; wrong document, form, owner, role, occurrence or source-map correspondence; selected unsupported form or unrepresentable type; and native FR-353 refusal each produce a named typed outcome with first native locus and no partial view or invented identity. An unrelated unsupported graph form or absent witness outside the explicit selection leaves the same successful result. Duplicate selected IDs produce one item. | Test |
| FR-357-AC-7 | Implemented (IR-721 selected facts). The bounded read inspects the explicitly selected admitted facts without traversing monitor entries or scanning unrelated graph nodes. Locating selected witnesses in full-emission row slices charges every inspected row and index entry, including unrelated rows; adding unrelated rows either raises required work or causes a charged first-row-locus stop at the former allowance. The exact allowance succeeds and one less refuses with attempted work and first uncharged locus. Equal inputs and allowance, including a package clone, yield equal results or first stops. | Test |
| FR-357-AC-8 | Implemented (IR-721 selected facts; API and dependency inspection). Public API and dependency inspection shows the read takes an admitted package, explicit selected model declaration/field IDs and equality application/occurrence pairs, IR-owned row slices, a work allowance and a caller-owned shared `NativeJoinBudget`, performs no QSL/RT dependency or source recheck, and returns neither an IR environment nor `ProgramParts`. The equality read uses that same budget for work and storage rather than a local equality limit. A caller adapts QSL-owned equality emission rows without changing their values; IR checks admitted joins but does not claim to authenticate arbitrary caller-created Exact payloads. | Inspection |
| FR-357-AC-9 | Implemented (`quire-integration` PR #47; explicit selected-fact scope only). A `quire-integration` test obtains a real final `PlanRoot.emission` through QSL, selects admitted IDs from its available model declaration/field rows and one equality application/occurrence pair, translates QSL equality rows without changing their values, and exercises the public IR selected-fact read on those explicit IDs. It verifies successful joins for representable selected model facts and an opaque FR-353 equality decision or its native typed missing-witness result for the selected equality. The executed PR #47 control observed opaque equality success; the missing-witness alternative remains source-reviewed rather than an observed result. It does not traverse four monitor entries or claim a complete generated-source closure, `TypeEnvironment` or RT `ProgramParts`, and it adds no QSL dependency to IR. | Test |
| FR-357-AC-10 | Implemented (IR-owned strict fixture; complete QSL handoff acceptance pending). A complete source record row joins one selected admitted source record with its original declaration identity, producer-claimed opaque record key and every member's original field identity, admitted name, presence, final type node and opaque value type in producer order. The selected record projects its opaque `ValueType::Composite` and exposes the complete joined member rows without creating a `MemberId`. Missing, foreign, duplicate, reordered, omitted, renamed or stale type-node rows refuse with a typed cause and no partial record view. Equal package clones and rows agree; an unrelated unsupported declaration is ignored. This explicit record projection does not claim four-entry closure or authenticate arbitrary caller rows as QSL output. | Test |
| FR-357-AC-11 | PLANNED/UNRUN (`quire-integration` same-emission acceptance). A genuine QSL final emission supplies the generated four-entry plan root; QSL's budgeted adapters return its source record/member, sealed union/member/payload, enum, model enum shape, collection, tuple, model, unit and equality rows as IR-owned row types without changing identities or order, and its `AdmittedPlanAuthority` supplies the FR-361 authority row. The one complete closure read over that authority's `admitted_package()` succeeds with those real rows, and a missing or pre-rewrite source row refuses at its native locus. The IR-owned fixture evidence for AC-1 through AC-3 alone does not satisfy this producer acceptance. | Test |
| FR-357-AC-12 | Implemented prerequisite (IR-owned strict QSpec monitor recipe in the explicit conformance-qspec lane; self-contained term bounds run in the default test gate; QSL same-emission handoff remains AC-11). Given one admitted selected monitor root, an internal borrowed read seeds its four selected type handles and four ordered entries, then visits only reached local graph nodes, dependency edges, semantic-type edges and ordinary typed-term reference, type and operation-member structural IDs and flat nominal IDs with iterative worklists and node/edge deduplication; specialized frame, abstraction, extension, operation-anchor and refinement terms refuse as unsupported in this prerequisite. Every inspected seed and node and each distinct inspected edge is charged to one finite allowance; repeated identical edges add no charge, and wide ordinary or flat nominal bodies stop at the first refused work item without materializing all child work; exact allowance succeeds, one below refuses at the first uncharged native locus without a partial result, and an equal package clone agrees. An unrelated graph node does not change the result or work, a deep ordinary term does not consume host stack, and a reached foreign dependency reference refuses with a typed cause; the foreign and specialized adverse tests inject terms after strict root admission to exercise the IR guard. This prerequisite does not join producer rows, project Exact or nominal facts, expose a driver-consumable result or certify complete type/declaration closure. | Test |
| FR-357-AC-13 | Implemented (IR-1353 reached-model prerequisite; TC-469). From an admitted selected monitor root whose walk reaches a selected model record and object, the package-private bounded read retains only those declarations and their followed model type dependencies. It retains complete ordered effective fields, original identities, presence, QSL-issued Exact identities, object ancestry and own operations with ordered parameters, result, effect and provenance; shared declarations appear once. A reached source unit and a reached model unit retain their distinct admitted owners, original identities, QSL-issued declared `UnitId`s, dimensions and exact conversion closures. An unrelated selected model declaration, variant or unit does not enter the facts, cause a refusal or change the exact reached-only allowance. This result does not certify a complete monitor closure. | Test |
| FR-357-AC-14 | Implemented (IR-1353 reached-model prerequisite; TC-469). A missing, foreign, duplicate or stale selected model/field/unit row, changed selected owner or original identity, unresolved field/operation/ancestry/unit dependency, unrepresentable member, or a reached model variant without its QSL-issued shape and variants refuses at the first native node, member or row with its typed cause and no partial facts. The read visits only reached nodes and followed dependencies without a package-wide discovery scan, while charging inspected reached members and selected row/index entries; it uses iterative visited sets, charges one shared allowance including nested reads and output, succeeds at the exact allowance and refuses one below at the first uncharged locus. Equal package clones agree. The internal helper creates no public closure API or environment. | Test |
| FR-357-AC-15 | Implemented (IR-1387 public budget API; TC-467). A driver can debit selection and adapter work and storage before allocation through one exported finite budget, then pass that same budget to a native read. The combined exact limit succeeds and one below refuses without resetting prior consumption or changing original limits. Driver charges retain their first unpaid kind, attempted amount and locus. | Test |
| FR-357-AC-16 | A reached admitted `composite_type`/`map` with a scalar key and a record value joins a collection row carrying its key node and value node in key-then-value order, and the closure retains the map's complete Exact `ValueType::Map`. A row whose `key_node` is absent, names another node, or is present on a non-map collection refuses `SelectedMonitorTypeClosureErrorCode::CollectionKey` at the collection node, with charged work and no partial closure; an unreached map row does not refuse. The closure with the map succeeds at its exact measured work and storage allowance, and one unit below either refuses at the first unpaid key-join charge with its kind, amount and locus. | Test |
| FR-357-AC-17 | The closure read requires the same-emission authority row of FR-361 and returns either a complete closure carrying the receipt and layout context or one typed first refusal; no read exists without that row, and no partial closure escapes an authority refusal. | Test, Inspection |

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
  enum declaration/member rows. The source enum shape row does not supply the
  QSL-issued `EnumShape` and `VariantId` facts of a model variant; existing
  model enum declaration/member rows contain only declaration/member keys and
  original identities. Reached model variants therefore retain a typed
  unavailable-shape boundary until a model-specific producer row exists.
  Collections need no declaration row. Source
  union native row joining and projection are implemented by IR PR #905
  (IR-1323), including complete ordered member/payload correspondence and
  distinct type-handle and settled-key roles. QSL-1723 now supplies
  final-emission union rows. QSL-1727 now supplies final-emission source
  record/member rows with original field names, declared presence and opaque
  Exact types; IR PR #910 joins source record rows structurally. The IR-owned
  fixture reads do not authenticate arbitrary caller-supplied opaque Exact
  values or prove one complete PlanRoot emission. AC-11 assigns that
  same-emission producer acceptance to `quire-integration` using normal QSL
  main. No source `MemberId` is retained by QSL's checking stage or invented
  by IR. IR PR #906 implements
  selective monitor-entry callable reads; PR #907 extends ordinary source
  function bodies to direct admitted Values. IR-1354 implements AC-1 through
  AC-4 using an IR-owned strict fixture and one native budget, including
  admitted equality. AC-11 retains the real same-emission producer handoff;
  the first selected-fact read alone does not satisfy AC-1 through AC-4.
  `quire-integration` owns
  AC-9's merged, executed real-emission QSL-to-IR integration test using the
  available IR selected-fact API; IR owns AC-5's explicitly selected fixture.
- Driver PR #117 delivers partial admission through opaque
  `AdmittedMonitorSelection` and the public `lower_scalar_functions` helper
  returning owned native RT function/expression arenas for an
  eleven-operator scalar component. This does not implement full `Layout`, the
  complete canonical `TypeEnvironment`, full `ProgramParts` or the complete
  monitor preparer. Those remain PLANNED/UNRUN with AC-1 through AC-4 and
  AC-11; partial admission does not authorize a partial
  executable owner.
- QSL-1728 owns the PlanRoot subject-occurrence proof needed for the driver's
  separate clock association. It is not a precondition for the IR type-closure
  traversal or its IR-owned fixture acceptance.
- RT FR-276 and its `ProgramParts` own the final immutable executable program.
  The driver consumes the future complete-closure IR projection to construct the canonical
  semantic-value `TypeEnvironment` passed to RT.
