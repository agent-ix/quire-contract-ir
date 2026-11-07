---
id: SR-202611070939004
title: "spec-review/dependency of IR-1404 strict V2 read exact charging"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md, spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md, spec/assurance/AD-001-contract-ir-architecture.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Independent spec-review/dependency of quire-contract-ir PR #948 at 81c8c041e23e63edaf33b378e2b7702bd5a99569 (base main c838196ce), scope `git diff origin/main...HEAD`. Ticket: IR-1404. Cross-checked against quire-canonical PR #17 and PR #18 heads and the current crate source. Focus: prerequisites and enablement edges the six-PR build plan needs.

## Examined scope

- AD-001-decision (examined): Read the package bytes and every selected model document with the one shared strict JSON reader owned by `agent-ix/quire-canonical`, and compare canonical bytes with its writer, both through their caller-reservation entries.
- FR-038-AC-509 (examined): Precedence: invalid UTF-8 anywhere refuses first; a repeated member followed inside the same object by a truncation refuses `malformed_wire` with no pointer; a repeated member in an object closed before a later grammar fault refuses `duplicate_member`
- FR-038-AC-514 (examined): A `literal` term's `value` and each diagnostic detail of an admitted package are `CheckedJson` handles into the package's one retained document. Cloning one shares the retained document without allocating a tree
- FR-358-AC-13 (examined): A strict read followed by a native join on the same budget sums their storage charges in one counter against one configured storage limit, and the join's exact and one-below limits shift by exactly the read's storage delta.

## Verdict

FAIL. The owner hooks S-1 and S-2 are both necessary and are not recorded as prerequisites; PR #17/#18 are open; a quire-walk hook is missing entirely.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | S-1, S-2, a quire-walk hook and the open quire-canonical PRs are unrecorded prerequisites | spec/assurance/AD-001-contract-ir-architecture.md:205-212 |

## Finding detail

### FND-001

S-1 is necessary: PR #18 returns ReadError::Malformed { offset, kind: DuplicateName } with no pointer and no document, so IR cannot produce AC-509's duplicate_member RFC 6901 pointer without re-parsing. S-2 is necessary: NodeRef<'d> borrows the Document and its id is private, so an owned CheckedJson (Arc<Document> plus node) cannot be built. A quire-walk reservation hook (see SR-202611070939002 FND-002) has no ticket. PLAT-1265 (PR #17) and PLAT-1247 (PR #18) are still open. None of these is a relationship or a stated prerequisite in FR-038, FR-358 or AD-001, so a coder can start a build-plan PR that cannot pass. Fix: list each as a blocking prerequisite with its owning ticket in FR-038 Dependencies and in Linear blocks edges.


## Dispositions

Verified against fix commit 03518d07b1afac4ecd962d57d44c2a7742ce7cb6 (81c8c041e..03518d0).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 03518d0; after: ### Owner prerequisites of the budget-charged read -- quire-canonical PR #17 (writer reservation), PR #18 (reader reservation), duplicate-name refusal pointer ( |
