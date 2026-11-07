---
id: SR-202611070939005
title: "spec-review/scope-boundary and architecture of IR-1404 AD-001 decisions"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md, spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md, spec/assurance/AD-001-contract-ir-architecture.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Independent spec-review/scope-boundary of quire-contract-ir PR #948 at 81c8c041e23e63edaf33b378e2b7702bd5a99569 (base main c838196ce), scope `git diff origin/main...HEAD`. Ticket: IR-1404. Cross-checked against quire-canonical PR #17 and PR #18 heads and the current crate source. Focus: the AD-001 decisions and the author decisions D-1 and D-4.

## Examined scope

- FR-358-storage-unit (examined): For the native joins, a **storage unit** is a logical ownership unit, not an allocator byte count (the strict package read below charges heap bytes)
- FR-358-AC-13 (examined): A strict read followed by a native join on the same budget sums their storage charges in one counter against one configured storage limit, and the join's exact and one-below limits shift by exactly the read's storage delta.
- FR-038-AC-496 (examined): A read clears the budget's previous success delta and first-unpaid record ... a read followed by an FR-358 native join on the same budget composes the same way.
- FR-038-limits-input (examined): the V2 read takes its bytes, nodes, edges, occurrences and diagnostics members as structural validation caps and does not consult the `work` member, which belongs to the V1 reader of FR-035
- FR-038-AC-495 (examined): `CheckedPackageV2::read(bytes, limits, evidence, &mut NativeJoinBudget)` and `read_checked_package` with the same budget parameter are the only V2 read entries ... Changing `CheckedPackageReadLimits.work` changes no V2 read outcome
- AD-001-decision (examined): Read the package bytes and every selected model document with the one shared strict JSON reader owned by `agent-ix/quire-canonical`, and compare canonical bytes with its writer, both through their caller-reservation entries.

## Verdict

FAIL on D-1 (one counter, two units). D-4 is acceptable with one gap. The AD-001 single-read and shared-reader decisions are sound boundaries.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | D-1: one storage counter sums heap bytes and logical units, so a storage limit has no meaning | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:123-124 |
| FND-002 | low | D-4: V2 ignores limits.work, but the default-limits claim and the V1 branch of read_checked_package are not updated | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:185-190 |

## Finding detail

### FND-001

After this PR the budget's single storage counter adds PackageRead heap bytes to the joins' logical units (one per container or entry plus text bytes) and to external accounted_charge logical storage (FR-357, FR-361 preparers). The mechanical AC can be written (FR-358-AC-13: the join's limits shift by the read's delta), but no caller can size the limit as either memory or entries, and an incomplete or exhausted 'consumed' value mixes units. Coherent alternative: make heap bytes the one storage unit for every charge on NativeJoinBudget, moving the native joins and accounted_charge callers to the same exact rule (they already admit before each reserve), as one build-plan PR; until that lands, state that the limit is a sum of mixed units. Two separate storage meters would also be coherent but breaks the one-meter ruling.

### FND-002

Ignoring `work` for V2 is acceptable (one meter), but FR-038 still says the default's members are all finite 'so that the default admits no unbounded read', which no longer holds for V2 work under an unlimited budget, and read_checked_package now takes a budget without saying whether its V1 branch charges it. Fix: say the V2 bound comes from the budget, and that the V1 branch ignores the budget (or charges it).

