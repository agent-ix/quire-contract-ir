---
id: SR-202611081512300010
title: "spec-review base of PR #1005 AC-511 bound amendment"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md; spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md; spec/checked_package/matrix/tests.md; IR-1404"
review_set: subset
---

## Summary

Ticket: IR-1404. Base checklist over the FR-038-AC-511, FR-038-AC-513, FR-358-AC-12, TC-472 and matrix edits in PR #1005. Sub-analyses were not run: the edits add no requirement, relationship or domain object, so integrity, dependency and object-review scopes do not apply, and the verification-method question for AC-513 is covered here. No hash, digest or pin was introduced. `quire validate` on the four edited files exited 0 with warnings on lines the PR did not change.

## Verdict

CHANGES REQUIRED. The bound itself is stated and testable, and no stale whole-read exact-equality claim remains in active spec. The matrix, however, records AC-511 as Tested beyond what the tests support, and TC-472 contradicts both the AC and the code in two places.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The matrix records FR-038-AC-511 as Tested, but the AC still says "every read" over every TC-048 fixture and the QSpec packages. The PR removed the bound check from the measured-read wrapper and from `observe`, so that corpus is now asserted only through `the_actual_tc048_catalogue_and_refusal_corpus_is_exactly_accounted`, which needs a QSpec checkout. Scope the row to the shapes shown, or keep it Planned for the corpus. | spec/checked_package/matrix/tests.md:20; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6530; tests/it/support/qspec_read_proof.rs:295 |
| FND-002 | medium | "the same bound holds when the input is made larger" is asserted for every read, but growth is exercised on one shape (a reference chain of 50 to 3200 nodes), as TC-472 itself says. A finite run cannot establish the universal clause. Name the shapes and sizes the clause covers. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6530; spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md:145 |
| FND-003 | medium | TC-472 says charged storage "differs" from requested bytes by at most the bound, which is two-sided, while the AC and `unpaid` are one-sided (`saturating_sub`). A read that charges 1 MiB more than it requests passes the test yet violates the TC-472 sentence. Say "exceeds" or assert both directions. | spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md:20; tests/it/support/read_observation.rs:52 |
| FND-004 | medium | TC-472 step 5 says every probe asserts requested bytes exceed charged bytes by no more than the bound, but `every_request` still asserts `stopped.requested == before`. The procedure does not describe the test, and an allocation the AC permits would fail it. | spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md:95; tests/it/support/read_allocation_proof.rs:179 |
| FND-005 | low | FR-358-AC-12's new wording "has a read-storage delta that counts the bytes it requested" no longer states a relation a test can check. Tie it to the FR-038-AC-511 bound. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:253 |

## Coverage

- `FR-038-AC-511` (examined): bound stated as 131072 bytes; see FND-001 and FND-002.
- `FR-038-AC-513` (examined): Planned status is consistent across FR-038, TC-472 step 10 and the matrix; the trace conflict is in the gap-analysis artifact.
- `FR-358-AC-12` (examined): see FND-005. The per-allocation equality in its first sentence is a per-class claim, not the whole-read equality the owner removed.
- `TC-472` (examined): see FND-002, FND-003 and FND-004.
- `spec/checked_package/matrix/tests.md` (examined): see FND-001.
