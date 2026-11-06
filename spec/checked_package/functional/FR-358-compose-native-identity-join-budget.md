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
structural locus, no partially authoritative selected-identity value, and the
budget's first unpaid charge when a limit denies work or storage. The budget
itself has fixed-size counters and charge metadata; it retains no per-charge
log or caller row.

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

A **storage unit** is a logical ownership unit, not an allocator byte count:
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

The budgeted path SHALL retain FR-352's structural, selected-document,
producer-order and opaque-identity comparisons. At sufficient limits its
selected values and typed non-budget refusals SHALL equal the corresponding
unbounded native decisions, including on an admitted package clone. Existing
in-repository callers SHALL pass a caller-owned budget; no compatibility
wrapper, fallback limit or second budgeted algorithm is introduced. Downstream
QSL callers must adapt before updating their IR dependency.

The owned diagnostic payloads of clock admission and Exact projection are governed by
[FR-359](./FR-359-bound-native-projection-refusals.md). This FR defines the
shared accounting rules used by their reads; its own acceptance criteria
remain scoped to native selected-identity acquisition.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-358-AC-1 | A public `NativeJoinBudget` with finite work and storage limits is passed by mutable reference to stage-row acquisition and each union, record, tuple, enum and collection join. Two individually affordable operations whose combined work or storage exceeds one limit stop at the first unpaid step of the second operation; no operation silently resets either counter. The public item list and call graph contain one mandatory budgeted acquisition path and no compatibility wrapper or parallel one-shot join entry. | Test, Inspection |
| FR-358-AC-2 | For each of stage-row, union, record, tuple, enum and collection acquisition, a real admitted fixture at both exact work and exact storage limits yields the complete same selected value as an ample allowance. One unit below either measured requirement refuses with the typed exhausted kind, first native locus and attempted charge, without a partially authoritative result. | Test |
| FR-358-AC-3 | Every IR-owned row, key, handle, container and map/set entry retained or cloned by the budgeted path, and each byte of copied dynamic payload, is charged by the defined logical-storage rule before the corresponding reserve, clone or insert. The tests make a prior nonempty map require an `Arc::make_mut` copy and show that a limit one below that copy's charge refuses before the new row appears. A borrowed inspection incurs no storage charge. | Test, Inspection |
| FR-358-AC-4 | Cumulative counters equal the sum of admitted charges across a successful stage acquisition and all five joins, while each success delta equals that operation's charge. A refusal clears the current success delta, leaves earlier admitted consumption visible, and records the first unpaid kind and amount without incrementing the denied charge; overflow cannot wrap either counter. Repeating the ordered inputs and limits, including against an admitted package clone, gives equal counters, values or first stops. | Test |
| FR-358-AC-5 | Each budgeted join preserves FR-352's selected-document, structural node, declaration/member identity, producer order and duplicate checks at sufficient allowance. Missing, foreign, conflicting, stale, wrong-form and incomplete rows retain their existing typed native causes and first loci, without a partial selected value. No caller-created opaque identity becomes authoritative through the budget API. | Test, Inspection |
| FR-358-AC-6 | The exported API is cycle-free and has no QSL or runtime dependency. In-repository callers use the mandatory budget parameter; the IR implementation introduces no compatibility path, pin, copied artifact or diagnostic-payload claim belonging to IR-1359. | Test, Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns admission, selected
  source identity and typed first structural refusals.
- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns the native
  identity joins and Exact projection semantics. This FR owns their shared
  work and storage accounting.
- [FR-357](./FR-357-project-selected-generated-type-closure.md) consumes the
  budgeted joins for a future complete selected monitor closure.
- [FR-359](./FR-359-bound-native-projection-refusals.md) applies this budget
  to bounded clock and Exact projection reads and their diagnostic payloads.
