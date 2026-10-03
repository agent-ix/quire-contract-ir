---
id: SR-773
title: "criterion-strength review of PR 254 (IR-503)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@731e307dc132975d3f3f6fac172c8c81bfe83b18; spec/checked_package/functional/FR-038-consume-checked-package-v2.md AC-65..68; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md Catalog words; reader code to judge whether each clause can fail"
review_set: subset
---
# SR-773: criterion-strength review of PR 254 (IR-503)

## Summary

Ticket: IR-503. Each new criterion can fail against today's reader:

- AC-65: the catalog at ec4563f panics in `parse_catalog` (`expect`, operation_catalog.rs:169-171).
  The error-without-panic clause requires `parse_catalog` to return an error, and today it panics.
- AC-66: today's reader cannot decode the catalog at all.
- AC-67: the precedence clause would fail if the refusal were placed before the lookup or the class
  check.
- AC-68: the five- and seven-argument and wrong-family clauses fail against a reader that keeps the
  old clause entry.

None of them is vacuous. AC-66's "agree or contradict" clause is a real adverse case: it pins
that the refusal does not depend on laws, mode, member, leaves or arguments.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | "Passes the reader's operation checks for the entry" and "a `reference` to a `temporal`/`formula` node fits the sixth operand" cannot be observed through a package read. A clause's sixth argument names a formula node that itself refuses `unsupported_construct` (line 668), and the reader reports only the lowest-digest failing node. When the formula node's digest is lower, the package refuses at the formula node whether or not the clause passed, so a clause that wrongly fails is masked. TC-048 says "the clause passes the entry's checks" without saying how that is observed. Fix: say how the pass is observed. Either build packages where the clause is the lowest-digest failing candidate and expect the refusal at the formula node, or name a unit-level check of the clause node's operation step. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1006 |

## Verdict

Strong overall, with one clause whose outcome cannot be observed at package level (medium).

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee (delta from 731e307dc132975d3f3f6fac172c8c81bfe83b18; base origin/main af733f23f42788ea2c8dfa1eb31adaff39e85960 unchanged; GitHub MERGEABLE). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/209 rows backed, FR-038 42/67. AC ids now run to AC-69, so the next free id is AC-70.

Notes on the round-1 checks: AC-68 is now observed at the clause node's own operation check (unit level), and TC-048 runs the clause as a unit with a record-typed `over` parameter reference. Every clause of AC-68 can fail.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
