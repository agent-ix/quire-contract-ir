---
id: SR-733
title: "criterion strength review of PR 249 (IR-504 content-only ModelOwner identity)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@fdad364d27f9e77c4525d06f2653eb2e69eca569; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-45"
review_set: subset
---
# SR-733: criterion strength review of PR 249

## Summary

Ticket: IR-504. Judged against the code at the reviewed sha, not by Jev.

FR-038-AC-45 can fail today and after IR-505:

- The admit clause fails against today's reader. An owner without `version` refuses as
  `invalid_semantic_graph` (missing member), so it is a true red-before-green criterion.
- The `unknown_member` clause fails against today's reader, which admits a `version`
  member. After IR-505 it is fixed by `deny_unknown_fields`, plus `locate_owner_failure`
  dropping `version` from its member list so that the pointer names the member.
- The version-invariance clause can fail. A reader that folded the lock row's `version`
  into the re-derivation would refuse the second read or report a different key.
- The empty `identity`/`node` clause is reachable. An empty `node` fails `validate_owner`.
  An empty `identity` cannot join, because lock class 4 refuses an empty-identity
  selection, so it refuses `invalid_semantic_graph`.

Adverse coverage: the AC covers the extra-member case and the empty cases. A missing
`node` or `identity` member (as opposed to an empty one) is covered by the general
preimage rule at FR-038 lines 99-102. AC-45 is scoped to "model-owned nominal preimages",
so it cannot catch a version-bearing model declaration node key. That gap is recorded as
SR-730 FND-001 and is not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

FR-038-AC-45 is a strong criterion for the scope it states. Its scope is too narrow
(SR-730 FND-001), not its strength.
