---
id: SR-612132
title: IR-1093 PR 659 protocol catalog code review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 659; crates/quire-contract-model/src/checked_package/v2/refinement.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1093. Reviewed PR #659 at 9c75a5ba against QSpec FR-432 and FR-464 and IR FR-038-AC-325/329. The corrected class spelling admits the four measured QSL read-backs, but catalog reconstruction still assigns a nested clause's member to its prefix clause.

## Verdict

FAIL: an admitted pair of protocol clauses with prefix-related qualified names can have its valid complete catalog refused.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The prefix-only member scan counts a member of an inner selected clause under an outer selected clause as well. With clauses `Example::Outer` and `Example::Outer::Inner`, an inner `[Example, Outer, Inner, finish, Done]` node is reconstructed as both `Example::Outer::Done` and `Example::Outer::Inner::Done`, although FR-464 assigns that node to exactly one clause. A correct catalog containing only the inner class is refused at the concrete catalog array. Use admitted clause membership, or at least require the fixed member-kind segment immediately after the selected clause path, and test this pair. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:2532 |

## Coverage

- FR-038-AC-329, examined: for selected clauses of differing path depth, reconstruct complete clause path plus authored member name, omit fixed kind and control ancestors, and refuse wrong class spellings.
- FR-038-AC-325, examined: reconstruct every class of the selected model's admitted clauses, excluding foreign or unselected clauses.
- QSpec FR-464 membership and containment, context: every protocol member node belongs to exactly one clause; the declaration prefix is followed immediately by that member operator's fixed segment.
- QSpec FR-432 class spellings, context: source class uses the selected protocol path and authored node name.
- Rust style, panic and wire boundary review: no new panic path or compatibility shim found in the changed code.
- CI workflow diff: none. Aggregate gates belong to the code lead after findings are resolved.
