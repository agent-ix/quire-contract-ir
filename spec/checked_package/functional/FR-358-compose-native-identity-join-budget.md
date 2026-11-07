---
id: FR-358
title: "Compose native selected-identity joins within one work and storage budget"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---
# FR-358: Compose native selected-identity joins within one work and storage budget

## Description

When a consumer assembles selected Exact identity witnesses from one admitted
checked package, Contract IR SHALL expose one caller-owned, finite work and
storage budget that composes across stage-row acquisition and all five native
source joins. The budgeted path owns no QSL identity judgment: the admitted
structural joins and the producer-supplied opaque identities remain governed by
[FR-352](./FR-352-project-admitted-exact-value-types.md). This requirement is
the native-join prerequisite for [FR-357](./FR-357-project-selected-generated-type-closure.md).

## Inputs

An admitted `CheckedPackageV2`; its selected declaration and field stage rows;
source union, record, tuple, enum and collection rows; and a mutable budget
created with finite `u64` work and storage limits. The caller passes the **same**
budget to `SelectedExactIdentities::from_stage_rows` and each
`with_source_*_rows` join. Each join consumes its input rows once. The caller
owns row construction before entry; the IR budget covers the work and logical
storage IR takes ownership of or allocates during the call.

## Outputs

Each successful operation returns one complete `SelectedExactIdentities` value
and records its work and storage delta in the budget. The budget exposes
cumulative consumed work and storage plus the most recent successful
operation's delta. A refusal returns the existing typed native cause and first
structural locus when its diagnostic storage is affordable, no partially
authoritative selected-identity value, and the budget's first unpaid charge
when a limit denies work or storage. If diagnostic storage is unaffordable,
the refusal retains the exhausted kind, attempted charge and consumed work
in a fixed-size form without an owned structural locus. The budget
itself has fixed-size counters and charge metadata; it retains no per-charge
log or caller row.

Before a native read, a driver MAY call `NativeJoinBudget::resume` and then
`accounted_charge(kind, amount, locus)` to admit selection, adapter work and
logical storage before performing that work or allocation. These public calls
use the same finite counters as the subsequent native read. `resume` clears
the previous operation's delta and denial metadata but leaves admitted charges
and original limits unchanged. A denied external charge records the first
unpaid kind, full attempted amount and caller-supplied non-owning locus without
incrementing consumption. Later external charges in that phase return the
first refusal without performing work. No public method restores spent allowance.

`NativeJoinLocusSource` SHALL include the unindexed native source
`MonitorAuthority` for Contract IR's own authority-row inspections made before
the selected occurrence is resolved, `AdapterRow(usize)` for a producer
row being converted into an IR-owned row by an external adapter,
`AuthorityRow` for an external preparer's charges while building the
same-emission authority row of
[FR-361](./FR-361-bind-monitor-authority-and-runtime-layout-context.md),
`PreparationOutput(usize)` for an owned output item an external preparer
allocates after a native read, and `PreparationSelection(usize)` for an
external preparer's selection comparison, indexed by its position in that
preparer's comparison order. The index is the position in the adapter's
input or the preparer's output sequence, and the locus `form` names the row
channel or output kind. The external variants let every external charge in
one preparation report a non-owning first-unpaid locus without allocating;
they do not change any native read's own loci.

`NativeJoinLocusSource` SHALL also include `PackageRead(PackageReadStage,
usize)` for a charge of the strict V2 checked-package read of
[FR-038](./FR-038-consume-checked-package-v2.md). `PackageReadStage` is a
closed, `Copy` enum of the read's stages in their order: `Envelope`,
`Canonical`, `WireDecode`, `WireChecks`, `NodeIndex`, `Lock`, `ModelDocument`,
`Graph`, `SourceMap`, `Equivalence`, `Extension`, `Capabilities`,
`Diagnostics`, `OperationEffects`, `CheckedTermData` and `Retained`. The index
and `form` of each stage are those FR-038's stage table names.

The budget SHALL expose its original configured work and storage limits through
`work_limit` and `storage_limit`; successful charges, denied charges and later
operations SHALL NOT change these values. A native resource refusal carried by
`ExactTypeProjectionError` SHALL expose `configured_bound` as the original
limit for its denied resource, including zero. This bound SHALL remain present
when diagnostic storage is unavailable and the refusal has no owned locus.
If allocating a work refusal's diagnostic also exceeds storage, the compact
refusal SHALL retain the original work denial and work limit. Non-resource
refusals SHALL carry no configured bound.

## Behavior

`NativeJoinBudget` SHALL be the sole public budget for this acquisition path.
The public stage-row constructor and the union, record, tuple, enum and
collection row joins SHALL require `&mut NativeJoinBudget`.
No join SHALL create a local fresh work or storage allowance. An operation starts with the
budget's current cumulative counters. On full success it records the exact
delta from that starting point; on refusal its success delta is absent. The
counters are monotone across calls, including a refused call. They count each
admitted charge, with no refund for temporary storage later released. A denied
charge does not increment either counter; it records its kind and attempted
amount at the first unpaid native step. Checked arithmetic exhaustion is a
denial, not a wrapped counter. A caller may inspect cumulative consumption
after success or refusal and may reuse the remaining allowance.

A **work unit** is one bounded native inspection or validation step over an
admitted node, source, row, member, identity or candidate entry. A scan over
caller-controlled text or a sequence charges each examined byte or element
before advancing. Charge order follows the admitted and input row order used
by the join; an unrelated graph node is not visited solely to consume budget.
The budgeted stage-row constructor charges declaration and field rows,
identity uniqueness checks and inserts. Each source join charges its row,
selected-source checks, member or case validations and duplicate/identity
checks. Successful work is never discarded between operations.

For the native joins, a **storage unit** is a logical ownership unit, not an
allocator byte count (the strict package read below charges heap bytes):
one unit for each newly owned container, retained row, key, handle, element or
map/set entry, plus one unit per byte of newly copied owned text or binary
payload. An opaque fixed-size identity counts as one handle; an owned dynamic
payload counts its contained elements and bytes. Moving a caller-owned row
into the IR selection charges its retained logical footprint. Cloning a prior
map or row charges that footprint again; merely borrowing it charges no
storage. A shared `Arc` handle costs one handle unit, and `Arc::make_mut`
charges the cloned map, its entries and any deep-copied payload before the
copy begins. Temporary uniqueness sets, buffers and owner maps use the same
rule. The charged total is deliberately conservative and deterministic for
the same admitted package and ordered rows; it does not claim exact heap
capacity or allocator overhead. IR SHALL compute and admit the required
storage charge before every corresponding reserve, clone or insertion. A
storage refusal cannot expose a partly extended selection.

**Strict package read accounting.** The strict V2 checked-package read of
FR-038 charges the same budget under one exact rule, which governs every
charge at a `PackageRead` locus. Its storage unit is one byte of heap: each
allocation charges the size in bytes the allocation requests, and each growth
of an existing allocation charges the bytes by which it grows, before the
allocation or growth is made. The classes are:

- a `Vec`, boxed slice or `String`: element size times the capacity added, the
  read reserving exactly the capacity it computed;
- a `Box<T>` or `Box<str>`: the size of `T` or the string's byte length;
- an `Arc<T>`, `Arc<[T]>` or `Arc<str>`: the size of the shared allocation,
  its two reference counts followed by the value, padded to alignment; cloning
  an `Arc` charges nothing;
- the shared reader's arena and scratch lists and the shared writer's work
  lists: the bytes their `quire_canonical::Reservation` asks, which is the
  capacity they add;
- decoded or copied text: its byte length, inside its containing class.

A zero-size allocation charges nothing. The rule is cumulative with no refund:
memory released during the read, including the reader's scratch lists, the
parsed package document when it is not retained, temporary tables and a
partly built diagnostic, keeps its charge, so the read's storage delta is the
sum of every allocation and growth it made. Charges at `PackageRead` loci and
the logical-unit charges of the native joins share the budget's one storage
counter and its configured limit; a caller sizes that limit for both. The
read's work counter keeps FR-038's validation-visit units.

IR SHALL supply the budget to `quire_canonical`'s reader and writer as a
`Reservation` that charges `Storage` at the active `PackageRead` locus. A
denied reservation records the first unpaid `Storage` charge with the bytes
asked and that locus; the reader or writer then allocates nothing for it and
returns no document or output.

When a native stage-row or source-join budget charge is denied, IR SHALL attempt to charge
the complete owned diagnostic before constructing it. Its logical-storage
floor is the footprint of the requested node, the footprint of the first
native locus, one form container and its bytes, and one error envelope unit.
When the floor is affordable, the refusal carries that first locus and the
admitted diagnostic charge remains in cumulative storage. When it is not
affordable, IR SHALL return a fixed-size exhausted refusal without allocating
an owned node, form or envelope. The first unpaid charge remains the original
work or storage denial, including when diagnostic storage is also denied.

The budgeted path SHALL retain FR-352's structural, selected-document,
producer-order and opaque-identity comparisons. At sufficient limits its
selected values and typed non-budget refusals SHALL equal the corresponding
unbounded native decisions, including on an admitted package clone. Existing
in-repository callers SHALL pass a caller-owned budget; no compatibility
wrapper, fallback limit or second budgeted algorithm is introduced. Downstream
QSL callers must adapt before updating their IR dependency.

The owned diagnostic payloads of clock admission and Exact projection are governed by
[FR-359](./FR-359-bound-native-projection-refusals.md). This FR defines the
shared accounting rules and governs native stage-row and source-join budget
denials; it does not claim that other diagnostic payloads are precharged.

The admitted equality read SHALL accept the caller's same mutable
`NativeJoinBudget` and charge inspected equality decisions, selected identity
rows, copied row material, QSV materialization work, and owned typed refusals
against its cumulative work and logical storage. An equality work or storage
denial SHALL retain the first unpaid kind and amount in the budget. The read
SHALL precharge the full owned diagnostic before allocation. A nonzero budget
that cannot hold QSV's full cause SHALL retain its fixed-size typed code and
first QSV locus without owning that cause; at zero available storage it SHALL
return a fixed-size typed refusal with no owned node or form.
Successful reads record their work and storage delta. No local equality work
limit or compatibility entry point is permitted.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-358-AC-1 | A public `NativeJoinBudget` with finite work and storage limits is passed by mutable reference to stage-row acquisition and each union, record, tuple, enum and collection join. Two individually affordable operations whose combined work or storage exceeds one limit stop at the first unpaid step of the second operation; no operation silently resets either counter. The public item list and call graph contain one mandatory budgeted acquisition path and no compatibility wrapper or parallel one-shot join entry. | Test, Inspection |
| FR-358-AC-2 | For each of stage-row, union, record, tuple, enum and collection acquisition, a real admitted fixture at both exact work and exact storage limits yields the complete same selected value as an ample allowance. One unit below either measured requirement refuses with the typed exhausted kind and attempted charge, without a partially authoritative result. When the diagnostic floor is affordable, the refusal carries the first native locus; otherwise it carries no owned locus or form and preserves the budget's first unpaid charge. | Test |
| FR-358-AC-3 | Every IR-owned row, key, handle, container and map/set entry retained or cloned by the budgeted path, and each byte of copied dynamic payload, is charged by the defined logical-storage rule before the corresponding reserve, clone or insert. On a budget denial the complete owned diagnostic floor is charged before any owned diagnostic allocation; at zero storage the refusal allocates no owned diagnostic. The tests make a prior nonempty map require an `Arc::make_mut` copy and show that a limit one below that copy's charge refuses before the new row appears. A borrowed inspection incurs no storage charge. | Test, Inspection |
| FR-358-AC-4 | Cumulative counters equal the sum of admitted charges across a successful stage acquisition and all five joins, while each success delta equals that operation's charge. A refusal clears the current success delta, leaves earlier admitted consumption visible, and records the first unpaid kind and amount without incrementing the denied charge; overflow cannot wrap either counter. Repeating the ordered inputs and limits, including against an admitted package clone, gives equal counters, values or first stops. | Test |
| FR-358-AC-5 | Each budgeted join preserves FR-352's selected-document, structural node, declaration/member identity, producer order and duplicate checks at sufficient allowance. Missing, foreign, conflicting, stale, wrong-form and incomplete rows retain their existing typed native causes and first loci, without a partial selected value. No caller-created opaque identity becomes authoritative through the budget API. | Test, Inspection |
| FR-358-AC-6 | The exported API is cycle-free and has no QSL or runtime dependency. In-repository callers use the mandatory budget parameter; the IR implementation introduces no compatibility path, pin, copied artifact or diagnostic-payload claim belonging to IR-1359. | Test, Inspection |
| FR-358-AC-7 | The admitted equality read requires the same caller-owned `NativeJoinBudget` and charges selected decisions, owned identities, row material, QSV work and typed refusal storage before the corresponding inspection or allocation. A complete admitted decision at exact work and storage allowances equals the ample value; one unit below either allowance refuses with the exhausted kind and first unpaid amount. Two individually affordable equality reads composed on a shared budget refuse at the first unpaid step of the second, retain cumulative consumption and clear the success delta. A nonzero allowance below a variable-size QSV cause's full diagnostic charge returns a compact typed code and original locus without the owned cause; its exact full charge retains the original cause. At zero storage a refusal owns no node, form or materialization payload. | Test, Inspection |
| FR-358-AC-8 | The public work and storage getters return the caller's original, distinct configured limits before acquisition, after successful stage and type projection operations, and after a later shared-budget denial; neither getter reports the remaining allowance. Each full or compact `ExactTypeProjectionError` resource refusal carries the original bound of its denied resource, including zero and the first work denial when diagnostic storage is also denied. A non-resource refusal has no configured bound. | Test |
| FR-358-AC-9 | Two external call sites use public `resume` and `accounted_charge` on one `NativeJoinBudget` before passing it to a native stage and projection read. Exact combined work and storage limits succeed; one below refuses with typed first unpaid charge, original bound and first locus. An external zero-storage denial occurs before allocation and does not consume storage. Resuming preserves all spent counters and configured limits; the budget offers no public counter reset or clone. | Test, Inspection |
| FR-358-AC-10 | An external adapter charging row `i` of a channel with `NativeJoinLocusSource::AdapterRow(i)`, a charge with `AuthorityRow`, a preparer charging output item `k` with `PreparationOutput(k)`, and a preparer charging comparison `k` with `PreparationSelection(k)` each admit at an allowance exactly equal to the cumulative consumption including that charge, and, one unit below, leave exactly that source, form, kind and attempted amount as the first unpaid locus without allocating; admitted charges with these sources are cumulative with the following native read. | Test |
| FR-358-AC-11 | `NativeJoinLocusSource::PackageRead(stage, index)` exists for every `PackageReadStage` of FR-038 in its stage order, is `Copy` and owns no node, key or string. A strict V2 read denied in each stage leaves `first_unpaid_locus` naming exactly that stage, the index FR-038 names and its static `form`, with no allocation made for the locus. The existing variants and their loci are unchanged. | Test, Inspection (TC-472) |
| FR-358-AC-12 | At a `PackageRead` locus, the charged storage of each allocation class — exact-capacity `Vec`, boxed slice and `String` growth, `Box<T>`, `Box<str>`, `Arc<T>`, `Arc<[T]>` and `Arc<str>` allocation, and shared reader and writer reservations — equals the bytes a counting global allocator observes for that allocation or growth; a zero-size allocation and an `Arc` clone charge nothing and are observed as nothing. Released memory keeps its charge: a read whose scratch and temporary tables are dropped before it returns has a storage delta equal to the total bytes allocated and grown, not its retained bytes. | Test (TC-472) |
| FR-358-AC-13 | The strict read passes the caller budget to `quire_canonical`'s reader and writer as a `Reservation`; a reservation one byte past the remaining storage is denied with first unpaid `Storage` equal to the bytes asked at the active `PackageRead` locus, the reader or writer allocates nothing for it and returns no document. A strict read followed by a native join on the same budget sums their storage charges in one counter against one configured storage limit, and the join's exact and one-below limits shift by exactly the read's storage delta. | Test (TC-472) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns admission, selected
  source identity and typed first structural refusals, and the stages and
  outcomes of the budget-charged strict read. This FR owns that read's
  accounting unit, retention rule and loci.
- `agent-ix/quire-canonical` owns the shared strict reader and the canonical
  writer, and their caller `Reservation` entries, which state their own
  accounting unit as bytes of capacity added.
- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns the native
  identity joins and Exact projection semantics. This FR owns their shared
  work and storage accounting.
- [FR-357](./FR-357-project-selected-generated-type-closure.md) consumes the
  budgeted joins for a future complete selected monitor closure.
- [FR-359](./FR-359-bound-native-projection-refusals.md) applies this budget
  to bounded clock and Exact projection reads and their diagnostic payloads.
