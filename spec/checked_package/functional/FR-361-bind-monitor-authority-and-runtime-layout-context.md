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
`authority: &'a SelectedMonitorAuthorityRow`. The row is an IR-owned value
with private fields, constructed only through the fallible
`SelectedMonitorAuthorityRow::try_new` from these components:

| Component | Type | Source |
| --- | --- | --- |
| `admission` | `AdmissionBinding` | The capability of the admitted package that the consumer strictly read from the same final emission as the QSL handoff (FR-038-AC-492). |
| `clause` | `CheckedNodeId` | The final emitted temporal clause declaration that QSL joined through the same emission's temporal binding rows. |
| `occurrence` | `CheckedOccurrence` | The selected claim occurrence: role `Claim` and its ordinal. |
| `time_source` | `SelectedMonitorTimeSource` | `Dense`, or `Tick { period: CheckedRational }` with the exact checked tick period. |
| `order_value` | `Box<[u8]>` | QSL's canonical model-step order value bytes, unchanged. |
| `progress_association` | `Box<[u8]>` | The observation owner's canonical association value bytes, unchanged. |

`try_new` SHALL refuse an empty `order_value` or `progress_association`, and
an `occurrence` whose role is not `Claim`, with
`SelectedMonitorAuthorityRowError` naming the component; it performs no other
check and allocates nothing beyond moving its arguments. The consumer SHALL
translate QSL's admitted handoff into this row without changing any value or
byte. A missing authority row is not representable: there is no `Option`,
default row or legacy read without authority.

The original structural model, unit, dimension and tick-operation identities
inside the order and association values are original checker identities, not
final emitted nodes. IR SHALL NOT compare them with admitted graph nodes;
their comparison against the checked time source is QSL's.

## Outputs

A successful `SelectedMonitorTypeClosure` SHALL expose
`authority() -> &SelectedMonitorAuthorityReceipt<'a>` and
`layout_context() -> SelectedMonitorLayoutContext<'a>`.

The receipt SHALL borrow from the closure and expose `clause()`,
`occurrence()`, `clock()` (the selected `CheckedMonitorClockRead` for that
occurrence), `order_value() -> &[u8]` and `progress_association() -> &[u8]`.
The two byte values SHALL be byte-identical to the row's values.

The layout context SHALL expose, with no re-encoding:

| Read | Value |
| --- | --- |
| `root_key()` | The plan's resolved root content key (`CheckedMonitorPlanRef::structural_key`). |
| `entry_keys()` | The four resolved entry content keys in `init`, `on_event`, `on_watermark`, `on_gap` order. |
| `clause_key()` | The resolved structural key of the admitted clause. |
| `package_lock()` | The exact canonical bytes of the admitted package's `lock` member, as they occur in the admitted wire. |
| `occurrence()` | The exact canonical bytes of the admitted source-map entry for the selected claim occurrence, as they occur in the admitted wire. |
| `profile()` | The admitted temporal profile selection identity text of the selected clause. |
| `subject()` | The `Display` text of the canonical Exact `NodeKey` of the clause's admitted subject (`over`) parameter. |
| `clock_binding()` | The canonical clock-binding value of the selected clock read (below). |
| `admitted_order()`, `progress_context()` | The receipt's `order_value` and `progress_association` bytes. |

The canonical clock-binding value SHALL be the RFC 8785 JSON object whose
members are exactly the [QSpec FR-252](ix://agent-ix/quire-specification/FR-252)
key components admitted by the selected profile, each holding the admitted
value: for a `model-time` binding, `binding_kind`, `selected_profile_identity`,
`time_source` (`kind` `dense`, or `kind` `tick` with `period` as canonical
decimal-string `numerator` and `denominator`), `unit` (the `Display` text of the
admitted unit's canonical Exact `NodeKey`) and `sequence_authority`
`"step-order"`. A
component the profile does not admit SHALL be absent, never empty or
defaulted. Display name, source path, span and locale SHALL NOT appear. The
value SHALL be encoded once, during the closure read, into storage charged
before allocation.

A refusal SHALL return `SelectedMonitorTypeClosureError` with
`code: SelectedMonitorTypeClosureErrorCode::Authority(SelectedMonitorAuthorityErrorCode)`,
the first native locus and charged work, and no closure, receipt or context.

## Behavior

### Authority binding

After the existing clock correspondence step and before any producer row join,
the closure read SHALL check the authority row in this order and stop at the
first failure:

1. `ForeignEmission`: the plan's admitted package does not `retain` the row's
   `admission` capability. An independent read of equal bytes, or a different
   package instance, SHALL refuse.
2. `ClauseMismatch`: the row's `clause` is not equal, by full structural
   identity, to the plan's admitted `clause` binding.
3. `OccurrenceMismatch`: the row's `occurrence` is not one of the clause's
   admitted claim occurrences retained in `clocks()`.
4. `TimeSourceMismatch`: the selected occurrence's admitted clock is not the
   model-time alternative, or its admitted time-source kind differs from the
   row's, or a tick period differs from the row's exact period.

Locus: `ForeignEmission` and `ClauseMismatch` at the plan root node;
`OccurrenceMismatch` and `TimeSourceMismatch` at the clause node with the row
occurrence. Equal clause names, labels, numeric indices or equal package
contents SHALL NOT satisfy any check.

### Charging

Every inspected identity, component and byte SHALL be charged as work before
inspection under FR-358. Retaining the receipt SHALL charge one handle for
each byte value plus one storage unit per byte copied, before the copy. The
layout context SHALL borrow admitted wire bytes and charge no storage for
them; the subject text and the clock-binding value SHALL each charge one
container plus one unit per encoded byte before allocation. The exact allowance SHALL succeed and one unit
below it SHALL refuse at the first unpaid charge with no receipt.

### Ownership boundary

The receipt and context SHALL borrow only the admitted package and the row.
They SHALL NOT contain a second plan identity, `PlanId`, digest or hash. QSL's
source `PlanId` remains metadata and SHALL NOT be read. The plan's identity
is its resolved root content key, which already covers the entries, premises
and helper call closure.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-361-AC-1 | An IR-owned strict admitted Timed monitor package with two claim occurrences of one clause and a row naming the second occurrence returns a closure whose receipt holds that occurrence, its own clock read and the exact row bytes; a row naming the first occurrence returns the first. Dense and tick sources with equal and different exact periods give the specified success or `TimeSourceMismatch`. | Test |
| FR-361-AC-2 | Each of an independent strict read of equal package bytes, another package, a different clause, a non-Claim role (refused by `try_new`), an absent ordinal, a model-time row against a non-Timed clock, a dense row against a tick source and a different exact period returns its named `SelectedMonitorAuthorityErrorCode` at the stated locus, with charged work and no closure; checks run in the stated order when two defects are present. Empty order or association bytes are refused by `try_new`. | Test |
| FR-361-AC-3 | The layout context's root, entry and clause keys equal the plan's resolved structural keys; `package_lock()` and `occurrence()` are byte-equal to the admitted wire's canonical member bytes and point into the admitted buffer; `clock_binding()` for dense and tick sources equals an independently written expected RFC 8785 object with exactly the specified members, and changing period, unit or source kind changes it. | Test |
| FR-361-AC-4 | The read with authority succeeds at its exact measured work and storage allowance; one unit below either refuses at the first unpaid authority or clock-binding charge with the original kind, amount and locus and no partial receipt; the same read on a package clone yields equal values and charges. | Test |
| FR-361-AC-5 | API and dependency inspection shows `SelectedMonitorClosureRows::authority` is a required field, `SelectedMonitorAuthorityRow` has private fields and one fallible constructor, the model crate gains no QSL, observation or runtime dependency, no byte value is decoded or re-encoded, and no `PlanId`, digest or second identity appears in the receipt or context. | Inspection |
| FR-361-AC-6 | In `quire-integration`, a genuine QSL plan root's admitted authority handoff, translated unchanged, binds through the closure read on the strictly read emission; the receipt bytes equal QSL's order value and the owner's association value, and a handoff from a second emission of the same source refuses `ForeignEmission`. | Test |

## Dependencies

- [FR-357](./FR-357-project-selected-generated-type-closure.md) owns the closure
  read this requirement extends; [FR-358](./FR-358-compose-native-identity-join-budget.md)
  owns the shared budget; FR-038 owns the admitted clock read (FR-038-AC-470,
  FR-038-AC-484) and the admission capability (FR-038-AC-492).
- QSL [FR-251](ix://agent-ix/quire-spec-language/FR-251) (AC-45 through
  AC-48) owns the order value, the same-emission handoff and the comparison of
  original checker identities; the observation owner's FR-004 owns the
  association value and its admission.
- The driver's [FR-019](ix://agent-ix/quire-driver/FR-019) consumes the receipt
  and the context to fill runtime `Layout`.
