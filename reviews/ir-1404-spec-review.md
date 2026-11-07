---
id: SR-202611070939001
title: "spec-review of IR-1404 exact budget charging of the strict V2 read"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md, spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md, spec/assurance/AD-001-contract-ir-architecture.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Independent spec-review of quire-contract-ir PR #948 at 81c8c041e23e63edaf33b378e2b7702bd5a99569 (base main c838196ce), scope `git diff origin/main...HEAD`. Ticket: IR-1404. Cross-checked against quire-canonical PR #17 and PR #18 heads and the current crate source. Base review; the sub-analyses carry the detailed findings (failure-domain SR-202611070939002, integrity SR-202611070939003, dependency SR-202611070939004, scope-boundary/architecture SR-202611070939005).

## Examined scope

- FR-038-AC-510 (examined): When the diagnostic's own storage is denied, or the denied allocation was a refusal diagnostic, the read returns a fixed-size `exhausted` outcome ... At zero storage, every read returns `exhausted` with no allocation observed.
- FR-038-AC-509 (examined): Precedence: invalid UTF-8 anywhere refuses first; a repeated member followed inside the same object by a truncation refuses `malformed_wire` with no pointer; a repeated member in an object closed before a later grammar fault refuses `duplicate_member`
- FR-038-AC-117 (examined): an otherwise canonical document whose node body nests a term 20 levels deep, 126 JSON levels deep (a body of 61 aggregates) or 300 levels deep each refuse `malformed_wire` at the first value outside the body grammar (FR-038-AC-114)
- FR-038-AC-495 (examined): `CheckedPackageV2::read(bytes, limits, evidence, &mut NativeJoinBudget)` and `read_checked_package` with the same budget parameter are the only V2 read entries ... Changing `CheckedPackageReadLimits.work` changes no V2 read outcome
- FR-038-AC-496 (examined): A read clears the budget's previous success delta and first-unpaid record ... a read followed by an FR-358 native join on the same budget composes the same way.
- FR-038-AC-512 (examined): Repeating a read with equal bytes, evidence and limits charges equal work and storage and stops at the same first unpaid charge and locus.
- FR-038-AC-26 (examined): storage at the value being decoded or validated when the denied allocation was requested (none for a shared reader or writer reservation)
- TC-472 (examined): Verify that the one strict V2 read charges every metered work step and every heap allocation it makes to the caller's `NativeJoinBudget` before the step or allocation

## Verdict

NOT MERGEABLE as written. The direction (one budgeted read, shared quire-canonical reader, no serde_json on the read path) is sound, but the exact-charging claim AC-511 is unachievable against the reservation unit PR #18 actually implements and against uncharged third-party allocations still on the read path, and D-2 leaves the public CheckedJson API undefined. Author decisions: D-1 defect (two units in one counter; use heap bytes for every storage charge); D-2 accepted in principle, defect in the spec (API, construction and serde undefined); D-3 sound; D-4 acceptable, small spec gap; D-5 sound change, not a narrowing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-510 'at zero storage every read returns exhausted' is false for an over-byte-limit input | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6157 |
| FND-002 | low | D-3 precedence change not checked against the QSpec adverse corpus; 300-level body clause duplicated with AC-117 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6156 |

## Finding detail

### FND-001

The shared reader checks max_input_bytes before asking the Reservation anything (quire-canonical PR #18 read_with_reservation doc), and an `incomplete` Bytes outcome owns no heap (CheckedPackageIncomplete.path is None for Bytes, shared.rs:593-604). So at zero storage an over-limit input returns `incomplete` Bytes with nothing allocated, not `exhausted`. Scenario: TC-472 step 5 reads the byte-limit fixture at zero storage and the AC fails, or a coder forces an exhausted outcome the outcome rules do not call for. Fix: 'At zero storage, every read that reaches a storage charge returns exhausted; an over-byte-limit read returns incomplete Bytes'.

### FND-002

The new duplicate-at-object-close order is sound and matches PR #18, but neither AC-509 nor the prose says that no QSpec adverse vector read through the FR-038-AC-112 harness pins the old serde_json order (duplicate before a later fault in the same object). If one does, the harness fails with no AC owning the change. Fix: add one sentence that the QSpec adverse corpus holds no such pair (or name the vector and its expected change). The 300-level body clause is also stated in both AC-117 and AC-509; keep it in one.


## Dispositions

Verified against fix commit 03518d07b1afac4ecd962d57d44c2a7742ce7cb6 (81c8c041e..03518d0).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 03518d0; after: At a zero read-storage limit, every read that reaches a read-storage charge returns `exhausted` with no request observed, while an input over the byte limit ret |
| FND-002 | fixed | Fix commit 03518d0; after: A test enumerates the QSpec adverse corpus read under FR-038-AC-112 and fails on any vector holding a repeated member together with a later grammar fault or inv |
