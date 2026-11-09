---
id: FR-361
title: "Bind same-emission order and progress authority and the runtime layout context to the selected monitor closure"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-010
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-251
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-004
    type: references
  - target: ix://agent-ix/quire-specification/FR-252
    type: references
  - target: ix://agent-ix/quire-contract-runtime/FR-276
    type: references
  - target: ix://agent-ix/quire-driver/FR-019
    type: references
---
# FR-361: Bind same-emission order and progress authority and the runtime layout context to the selected monitor closure

## Description

When a consumer reads the complete selected closure of one admitted monitor
root under [FR-357](./FR-357-project-selected-generated-type-closure.md),
Contract IR SHALL also bind, in the same atomic read and under the same
caller-owned `NativeJoinBudget`, the order and progress authority that QSL
admitted for the same final emission, and SHALL expose the exact selection
context values the runtime `Layout` retains. The result is a receipt of IR's
structural correspondence checks over admitted facts. IR SHALL NOT decode,
re-encode, authenticate or derive the order value or the progress association
value; QSL and the observation owner own those values and their admission.
IR SHALL NOT depend on QSL, the observation owner or the runtime.

The architecture this requirement belongs to is stated
in [AD-010](../../assurance/AD-010-tick-monitor-admission-lowering-and-execution.md).

## Inputs

`SelectedMonitorClosureRows` SHALL carry one required field
`authority: &'a SelectedMonitorAuthorityRow<'a>`. The row is an IR-owned value
with private fields, constructed only through the fallible
`SelectedMonitorAuthorityRow::try_new` from these components, each taken from
QSL's `AdmittedPlanAuthority` without changing a value or byte:

| Component | Type | Source in the QSL handoff |
| --- | --- | --- |
| `admission` | `AdmissionBinding` | `AdmittedPlanAuthority::admission_binding()`: the capability of the package QSL strictly read from this plan root's own final emission (FR-038-AC-492). |
| `clause` | `CheckedNodeId` | The admitted clause of that package's single monitor plan, which QSL joined to the plan subject through the same emission's temporal binding rows. |
| `occurrence` | `CheckedOccurrence` | Role `Claim` and the plan subject's claim ordinal. |
| `time_source` | `SelectedMonitorTimeSource` | `Dense`, or `Tick { period: CheckedRational }` with the checked time source's exact period. |
| `order_value` | `&'a [u8]` | `AdmittedPlanAuthority::admitted_order()`. |
| `progress_association` | `&'a [u8]` | `AdmittedPlanAuthority::progress_context()`. |

The row borrows the two byte values from the handoff; neither IR nor the row
copies them. `try_new` SHALL return
`Err(SelectedMonitorAuthorityRowError { component })`, where `component` is the
closed enum `SelectedMonitorAuthorityComponent { Occurrence, OrderValue,
ProgressAssociation }`, for a non-`Claim` occurrence role, an empty order value
or an empty association value, checked in that order. It performs no other
check and allocates nothing. A missing authority row is not representable:
there is no `Option`, default row or read without authority.

The original structural model, unit, dimension and tick-operation identities
inside the order and association values are original checker identities, not
final emitted nodes. IR SHALL NOT compare them with admitted graph nodes;
QSL compares them against the checked time source.

### Cross-owner seam: QSL `AdmittedPlanAuthority`

QSL owns the single admission stage `PlanRoot::admit_authority` and its
signature. It consumes the `PlanRoot` and takes, by role: the authority
selection, the optional observation progress view, QSL's structural
validation caps, the caller's `&mut NativeJoinBudget` and the caller's
`NativeJoinLocus`; it returns `AdmittedPlanAuthority` or a typed
`PlanAuthorityRefusal`. This requirement depends on QSL providing, on
`AdmittedPlanAuthority`:

- `admitted_package() -> &CheckedPackageV2`, the package of its own one strict
  read of the plan root's final emission, and `admission_binding() ->
  AdmissionBinding` issued by that same package;
- `admitted_order()` and `progress_context()` borrowed for the authority's
  lifetime;
- a borrowed read of the checked time source giving its kind, and a retained
  borrowed IR `CheckedRational` period for a tick source, from which the
  row's `time_source` is built (QSL names these reads);
- charging of every QSL selection, strict-read and canonical-value allocation
  to the caller's `NativeJoinBudget` before it is made.

FR-361-AC-6 depends on this seam.

## Outputs

A successful `SelectedMonitorTypeClosure` SHALL expose
`authority() -> &SelectedMonitorAuthorityReceipt<'a>` and
`layout_context() -> &SelectedMonitorLayoutContext`.

The receipt SHALL expose `clause()`, `occurrence()`, `clock()` (the selected
`CheckedMonitorClockRead` for that occurrence), `order_value() -> &'a [u8]` and
`progress_association() -> &'a [u8]`, which return the row's borrowed slices.

The layout context SHALL own these values, each produced once during the
closure read:

| Read | Value |
| --- | --- |
| `root_key()`, `entry_keys()`, `clause_key()` | Borrowed resolved structural keys of the plan root, the four entries in `init`, `on_event`, `on_watermark`, `on_gap` order, and the admitted clause. |
| `package_lock() -> &[u8]` | The RFC 8785 encoding of the admitted `CheckedPackageLockV2` produced by IR's V2 envelope writer for the `lock` member (the fixed-shape member encoding of `encode_checked_package_v2`). |
| `occurrence() -> &[u8]` | The RFC 8785 encoding, by its `quire_canonical::Encode` implementation, of the admitted `CheckedSourceMapEntry` for the selected claim occurrence. |
| `profile() -> &str` | The RFC 8785 text of the object `{"authority", "identity", "role"}` holding the selected clause profile's `CheckedSelection` definition authority, definition identity and the role's wire spelling. |
| `subject() -> &str` | The identity text of the clause's admitted subject (`over`) model, read from its one `declarations` row of the closure rows: the `Display` text of a `Record` row's `NodeKey`, or for an `Object` row the RFC 8785 text of its `EffectiveId` in the FR-150 form IR's `CheckedEffectiveId` encodes (`{"original":{"node","package"},"owner_effective_type":null|{...}}`). |
| `clock_binding() -> &[u8]` | The canonical clock-binding value (below). |

The strict V2 reader refuses noncanonical input, so the admitted package's
`lock` member and its source-map entries are already in RFC 8785 form; the
writer's encoding of the admitted typed value therefore equals the admitted
input bytes of that member. IR SHALL NOT retain a raw input buffer for this.

The canonical clock-binding value SHALL be the RFC 8785 JSON object whose
members are exactly the [QSpec FR-252](ix://agent-ix/quire-specification/FR-252)
key components a `model-time` binding admits, read from the admitted
timed-model facts of the selected occurrence:

| Member | Value |
| --- | --- |
| `binding_kind` | `"model-time"` |
| `selected_profile_identity` | The same object as `profile()`, as a JSON object. |
| `time_source` | An object with `kind` (`"dense"` or `"tick"`), `model` (the model's qualified name: the admitted declaration path, segments joined by `.`, or, for a node with no `declaration`, the original identity of its owning model declaration (`owner.node`) verbatim, of the model named by one source per kind: for `tick`, the time-source node's own `model` binding; for `dense`, which has no such binding, the timed-model root's `model` binding), and, for `tick` only, `operation` (the admitted declaration path of the time source's `operation` binding, joined by `.`; an `operation_anchor` has no declaration and contributes its own `operation` binding text verbatim). |
| `unit` | The `Display` text of the `NodeKey` of the time source's admitted `unit`, read from its one Declared-domain `UnitId` among the `declarations` (`DeclaredUnit`) and `source_units` rows. |
| `period` | For `tick` only: `{"denominator", "numerator"}` canonical decimal strings of the time source's admitted `period` rational. Absent for `dense`. |
| `sequence_authority` | `"step-order"` |

No other member SHALL appear and no admitted member SHALL be empty or
defaulted. The kind, unit, operation and period SHALL be read from the
time-source node named by `CheckedMonitorClock::ModelTime { source, .. }`: its
`kind` and `unit` bindings, and for `tick` its `operation` and `period`
bindings. The model SHALL be read from the single source the table names for
the kind. The runtime compares this value
byte-for-byte as the clock-binding value of RT interface-012; IR is its only
producer.

A refusal SHALL return `SelectedMonitorTypeClosureError` with
`code: SelectedMonitorTypeClosureErrorCode::Authority(SelectedMonitorAuthorityErrorCode)`,
the first native locus and charged work, and no closure, receipt or context.

## Behavior

### Step order

The closure read SHALL run its steps in this order and report the first
refusal:

1. `read_reached_model_facts`: graph and term traversal and the model
   declaration, field, tuple, source-unit and model enum joins.
2. Clock correspondence of every claim occurrence of the clause.
3. Authority binding (below).
4. Admitted equality reads.
5. Source record, union, tuple, enum and collection joins and Exact
   projection.
6. Layout context and output materialization.

A model-row refusal therefore precedes an authority refusal, and an authority
refusal precedes every equality, source-row or projection refusal.

### Authority binding

The read SHALL check, stopping at the first failure:

1. `ForeignEmission`: the plan's package does not `retain` the row's
   `admission` capability. An independent read of equal bytes, or another
   package instance, SHALL refuse. When the capability is retained, the plan
   is a view of that admitted package, so its root, four entry and clause node
   IDs resolve to their structural keys in it; this is an invariant of the
   admitted package, not a further refusal.
2. `ClauseMismatch`: the row's `clause` is not equal, by full structural
   identity, to the plan's admitted `clause` binding.
3. `OccurrenceMismatch`: the row's `occurrence` is not one of the clause's
   admitted claim occurrences retained in `clocks()`.
4. `TimeSourceMismatch`: the selected occurrence's clock is not
   `CheckedMonitorClock::ModelTime`, or the time-source node's admitted `kind`
   binding differs from the row's kind, or a tick `period` binding differs
   from the row's exact period.

5. `LayoutIdentityUnavailable`, read after the closure's other steps with the
   layout context: the clause's subject model has not exactly one `declarations`
   row of identity `Record` or `Object`, or the time source's unit has not
   exactly one row among the `declarations` (`DeclaredUnit`) and `source_units`
   rows, whose `UnitId` is `Declared`. A compound unit is refused. The locus is
   the subject model node or the time-source unit node that failed. The layout
   context adds no `tick_unit` member: the driver takes the tick unit from
   `plan.tick_unit()` and the closure's units.

   The same code also refuses, at the node that failed, any identity the read
   cannot resolve (a missing claim source-map entry, profile law or selection,
   time source, model or operation name, or clause key) and a non-model-time
   clock. These are defense in depth: package admission and the authority
   binding already guarantee them, so they are not reachable through the
   closure read. Of the row checks, a missing or ambiguous subject or unit row,
   a `Union` subject and a compound unit are likewise refused earlier in the
   closure read by the reached-model steps, and are reachable only through a
   direct `layout_context` call with rows the closure read would not accept.
   No refusal of this step panics.

Locus: `ForeignEmission` and `ClauseMismatch` at the plan root node;
`OccurrenceMismatch` and `TimeSourceMismatch` at the clause node with the row
occurrence. Equal clause names, labels, numeric indices or equal package
contents SHALL NOT satisfy any check.

### Charging

Every inspected identity, component and binding SHALL be charged as work
before inspection under FR-358. The receipt borrows its byte values and SHALL
charge no storage for them. Each owned layout-context value SHALL charge one
container plus one storage unit per encoded byte before allocation. IR's own
charges SHALL use these loci:

| Charge | Locus source | Form |
| --- | --- | --- |
| `ForeignEmission`, `ClauseMismatch` and `OccurrenceMismatch` inspections, before the occurrence is resolved | `NativeJoinLocusSource::MonitorAuthority` | `"selected_authority"` |
| `TimeSourceMismatch` inspections, after the occurrence resolves to index `k` in `clocks()` | `MonitorSeed(k)` | `"selected_authority"` |
| Layout-context values, including the writer scratch of their canonical encodings | `MonitorSeed(k)` | `"selected_layout_context"` |
| Clock-binding value | `MonitorSeed(k)` | `"selected_clock_binding"` |
 The exact allowance SHALL succeed and one unit below it SHALL refuse
at the first unpaid charge with no receipt.

### Ownership boundary

The receipt and context SHALL borrow only the admitted package and the row.
They SHALL NOT contain a second plan identity, `PlanId`, digest or hash. QSL's
source `PlanId` remains metadata and SHALL NOT be read. The plan's identity is
its resolved root content key, which already covers the entries, premises and
helper call closure.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-361-AC-1 | An IR-owned strict admitted Timed monitor package with two claim occurrences of one clause and a row naming the second occurrence returns a closure whose receipt holds that occurrence, its own clock read and the row's exact byte slices; a row naming the first returns the first. Dense and tick sources with equal and different exact periods give the specified success or `TimeSourceMismatch`. | Test |
| FR-361-AC-2 | Each of an independent strict read of equal package bytes, another package, a different clause, an absent ordinal, a configured clock, a dense row against a tick source and a different exact period returns its named `SelectedMonitorAuthorityErrorCode` at the stated locus, with charged work and no closure. Paired defects show the stated authority check order; a model-row defect beside an authority defect reports the model row, and an authority defect beside a source-row or equality defect reports the authority. `try_new` returns `SelectedMonitorAuthorityRowError` with component `Occurrence`, `OrderValue` or `ProgressAssociation` for a non-Claim role, an empty order value and an empty association value. | Test |
| FR-361-AC-3 | (Layout context implemented.) The root, entry and clause keys equal the plan's resolved structural keys. `package_lock()` and `occurrence()` are byte-equal to the corresponding member bytes of the admitted input. `profile()` and `clock_binding()` for dense and tick sources equal independently written expected RFC 8785 values with exactly the specified members; changing the model, tick operation, unit, period, source kind or profile each changes `clock_binding()`. | Test |
| FR-361-AC-4 | The read succeeds at its exact measured work and storage allowance. One unit below the cumulative charge before each authority, layout-context and clock-binding charge refuses at that charge with its kind, amount and exact locus: `MonitorAuthority` for the admission, clause and occurrence inspections (including at an `OccurrenceMismatch`), and `MonitorSeed(k)` with the tabled form after resolution; no partial receipt escapes. The same read on a package clone yields equal values and charges. | Test |
| FR-361-AC-5 | API and dependency inspection shows `SelectedMonitorClosureRows::authority` is a required field, the row has private fields and one fallible constructor and borrows its byte values, the model crate gains no QSL, observation or runtime dependency, no byte value is decoded or re-encoded, no raw input buffer is retained, and no `PlanId`, digest or second identity appears. | Inspection |
| FR-361-AC-6 | In `quire-integration`, a genuine `AdmittedPlanAuthority` row binds through the closure read on its own `admitted_package()`; the receipt bytes equal QSL's order value and the owner's association value. The same row against the package of a second admission of the same plan root's bytes refuses `ForeignEmission`. Depends on the QSL seam above. | Test |

## Dependencies

- [FR-357](./FR-357-project-selected-generated-type-closure.md) owns the closure
  read this requirement extends; [FR-358](./FR-358-compose-native-identity-join-budget.md)
  owns the shared budget; FR-038 owns strict canonical admission, the admitted
  clock and model-time reads (FR-038-AC-470, FR-038-AC-484) and the admission
  capability (FR-038-AC-492).
- QSL [FR-251](ix://agent-ix/quire-spec-language/FR-251) (AC-45 through
  AC-48) and the seam above own the order value, the same-emission admission
  and the comparison of original checker identities; the observation owner's
  FR-004 owns the association value.
- The driver's [FR-019](ix://agent-ix/quire-driver/FR-019) consumes the receipt
  and the context to fill runtime `Layout`.
