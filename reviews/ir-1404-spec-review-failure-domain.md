---
id: SR-202611070939002
title: "spec-review/failure-domain of IR-1404 strict V2 read exact charging"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md, spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md, spec/assurance/AD-001-contract-ir-architecture.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Independent spec-review/failure-domain of quire-contract-ir PR #948 at 81c8c041e23e63edaf33b378e2b7702bd5a99569 (base main c838196ce), scope `git diff origin/main...HEAD`. Ticket: IR-1404. Cross-checked against quire-canonical PR #17 and PR #18 heads and the current crate source. Focus: whether 'charged == observed' (AC-511, FR-358-AC-12) can hold, and whether every allocation on the read path has a charge point.

## Examined scope

- FR-038-AC-511 (examined): Whole-read accounting. A test binary whose global allocator counts the bytes of each allocation and of each reallocation's growth on the reading thread reads ... For every read, the storage delta the budget admitted equals the observed allocated bytes during the call, and no allocation is observed after a denied charge.
- FR-358-AC-12 (examined): At a `PackageRead` locus, the charged storage of each allocation class ... and shared reader and writer reservations — equals the bytes a counting global allocator observes for that allocation or growth
- FR-358-strict-accounting (examined): each growth of an existing allocation charges the bytes by which it grows ... the shared reader's arena and scratch lists and the shared writer's work lists: the bytes their `quire_canonical::Reservation` asks, which is the capacity they add
- FR-038-AC-513 (examined): The V2 read path (`CheckedPackageV2::read` and every function it reaches up to returning its result) calls no `serde_json` parser, deserializer or `RawValue`, constructs no `serde_json::Value`, and builds no `BTreeMap`, `BTreeSet`, `HashMap` or `serde_json::Map`
- FR-038-AC-512 (examined): Repeating a read with equal bytes, evidence and limits charges equal work and storage and stops at the same first unpaid charge and locus.
- FR-038-container-rule (examined): Lookup tables are sorted exact-capacity slices searched by binary search; the read builds no `BTreeMap`, `BTreeSet`, `HashMap` or `serde_json::Map`. Sorting uses an unstable in-place sort or a charged buffer.
- AD-001-decision (examined): Read the package bytes and every selected model document with the one shared strict JSON reader owned by `agent-ix/quire-canonical`, and compare canonical bytes with its writer, both through their caller-reservation entries.

## Verdict

FAIL. Three independent reasons make AC-511 unachievable as specified: the reservation unit mismatch, uncharged third-party allocations, and the lazily parsed operation catalog. Each needs a spec decision before coders start.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | quire-canonical Reservation charges the full new capacity at each growth; FR-358/AC-511 count only the growth, so charged != observed | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:140-159 |
| FND-002 | high | Uncharged allocations in quire-walk and num-bigint stay on the read path; no allocation class or hook covers them | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6158 |
| FND-003 | high | The operation catalog is parsed lazily with serde_json inside the first V2 read (OnceLock) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6160 |
| FND-004 | medium | 'No allocation is observed after a denied charge' contradicts the incomplete outcome, which charges and allocates its diagnostic after the denial | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6158 |
| FND-005 | low | 'Sorting uses an unstable in-place sort or a charged buffer': the size of std's stable-sort buffer is implementation-defined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5632-5633 |

## Finding detail

### FND-001

quire-canonical PR #18 Provision::admit computes target = max(needed, 2*capacity, MIN_CAPACITY) and charges item_size * target (the whole new capacity), documented as 'bytes of list capacity, charged in full at each growth'. FR-358 says each growth 'charges the bytes by which it grows' and that the reservation asks 'the capacity they add'; TC-472 step 1 counts 'the growth of every reallocation'. For any reader or writer list that grows twice (every non-trivial document) the reservation charge exceeds the allocator-observed growth, so AC-511 and FR-358-AC-12 fail. Fix (one of): define the unit as the full size of every allocation and reallocation request (count realloc at its new size in the counting allocator, and say growth charges the new allocation's full size), which matches quire-canonical's 'everything it requested from the allocator'; or change PR #18 to charge the delta. Pick one in the spec; the first needs no upstream change.

### FND-002

The FR keeps term walks on 'the shared quire-walk crate's' heap stack (FR-038:394), but quire-walk's walker grows its tasks and pending Vecs by push from Vec::new (quire-walk src/walker.rs:83-100) with amortized growth and no reservation hook. The read path also parses and reduces BigInt/BigUint (structural.rs:1117-1128 canonical_rational used by interval and operation checks, operations.rs:542, timed_model.rs:1515), whose limb allocations num-bigint sizes internally. Neither is a listed allocation class, neither can be 'computed before the allocation', so the counting allocator will observe bytes no charge covers. Fix: add a quire-walk caller-reservation hook as an owner prerequisite (or specify an IR-owned charged walk stack for the read), and specify that rational/integer canonical checks on the read path use charged decimal-text arithmetic or charged limb buffers, not num-bigint.

### FND-003

operation_catalog() (v2/operation_catalog.rs:269-281) initializes a static OnceLock on first use by read_catalog, which calls serde_json::from_str and serde_path_to_error and builds keyed maps (operation_catalog.rs:308-320). The first CheckedPackageV2::read in a process reaches it, so AC-513 (no serde_json parser and no map on any function the read reaches) is violated, and the first read observes catalog allocations that no PackageRead charge covers (AC-511), while a later read does not (the counting test depends on test order). The spec never mentions the catalog. Fix: state that the catalog is a build-time constant table, or is initialized by an explicit call outside the read whose allocations are out of the measured call, and exclude it from AC-513 explicitly.

### FND-004

The outcome rule (FR-038 'Outcomes of a denial') records the first unpaid charge and then charges and allocates the owned incomplete diagnostic (its JsonPointer Box<str>). AC-511 forbids any allocation after a denied charge, so every affordable-diagnostic one-below case in AC-497..AC-507 fails AC-511. Fix: 'no allocation is made for the denied request, and every allocation after it is the charged outcome diagnostic'.

### FND-005

slice::sort/sort_by allocate a scratch buffer whose size the standard library chooses and changes between releases, so a coder cannot compute and charge it exactly. Fix: require sort_unstable* on the read path, or an IR-owned merge buffer of a stated size; drop 'or a charged buffer' for std sorts.

