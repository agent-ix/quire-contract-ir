---
id: SR-2960
title: "Code review of the IR-663 private intake retention code"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir branch code/ir663-private-intake-retention, head commit 'Charge missing-origin group candidates with an independent visit oracle' (3 commits over main); crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/checked_package/v2/model_members.rs; crates/quire-contract-model/src/checked_package/v2/model_members/intake_origin.rs; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs; crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-663. Code review with the `rust-review` lane folded in, against the
merged FR-038 declaration-refusal retention contract (AC-174, AC-175,
AC-186 through AC-196), TC-048 steps 1 to 7 and QSpec FR-154's retention
paragraph on QSpec main (read by reference, not copied). Static review only:
no cargo command was run, per the dispatch brief. The author reports 11
selected tests passing and both Clippy lanes clean on an earlier head; the
final commit's production charge and its two new assertions are unrun.

What the production change does, checked by reading it:

- `SelectionRefusal` gains private owned `declaration_identity: Option<Box<str>>`
  and `declaration_origin: Option<IntakeDeclarationOrigin>`. The `Clone, Copy`
  derives are dropped and `Debug, Eq, PartialEq` remain. Every item stays
  `pub(super)` or `pub(in crate::checked_package::v2)`. No `pub` item is added
  or changed, `CheckedPackageRefusal` is untouched, and no dependency,
  Cargo.toml or Cargo.lock change exists. There is no compatibility layer.
- `intake_origin.rs` holds one borrowed `OriginView::decode`, which is now
  also the relationship admission grammar (`valid_relationship_origin`
  delegates to it). The removed body of `valid_relationship_origin` is the
  same predicate, so FCD origin parsing has one owner. `into_owned` copies a
  decoded view into the owned typed enum. `Generated` has no span field, so
  the enum cannot represent a span or a sentinel.
- `charge_origin` is iterative and at most two levels deep, with no
  recursion. It charges 1 per node, per branch, per member, per input element
  and per text byte before decode. All arithmetic saturates. Production code
  has no `unwrap`, `expect`, indexing or `as` casts.
- `SelectionRefusal::located` retains identity and origin from the actual
  offending JSON node. `SelectionRefusal::group` locates the first candidate,
  then walks every remaining candidate with a 1-unit visit charge and a full
  derived `PartialEq` comparison of typed origins. It retains an origin only
  when every candidate's origin is `Some` and equal. The walk does not stop at
  the first disagreement, so the work charged does not depend on candidate
  order.
- `Defects` records the declaration context of each defect. The
  `defects.context(..)` calls switch to field, operation, parameter and
  relationship, and reset to the owner after each list. The relationship
  duplicate defect (known or local) is pushed while the later relationship
  is the context, which satisfies AC-196.
- In `v2/mod.rs`, the `SelectionFailure::Refused` arm destructures the
  refusal and explicitly `drop`s the identity and origin before projecting
  `ValidationFailure::refused_because(code, path, cause)`. The public output
  is unchanged.

How the consumer-only Clippy lane can pass: `declaration_origin` is genuinely
read in production by `group`, through `is_some` and the derived `PartialEq`,
which dead-code analysis does not ignore. `IntakeDeclarationOrigin`'s fields
are live through that same `PartialEq`. In production, `declaration_identity`
is read only by the move into the explicit `drop` in the mapping arm. That
move is the "explicit ownership teardown" AC-194 names as passing. It is not
a blanket `allow(dead_code)`, an observer or a public API. The pre-existing
`allow(dead_code)` lines in `operation_catalog.rs` are outside this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-188 context resets are not pinned. No test puts a non-empty nested list before an owner-level or operation-return defect. Deleting any `defects.context(value)` or `defects.context(operation)` reset leaves every test green. Example: an operation with one valid parameter (origin `param.md`) and an unresolved `returns` would retain the parameter's identity and origin. The operation test uses `"params": []`, so the reset at model_members.rs:2458 is never needed. The same holds for supertype, relationship-list and operation-list defects after earlier fields, operations or relationships (resets at :2441, :2469, :2497). Field and operation "nested metadata absent => None" is also never asserted; only the parameter case is. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:2441,2458,2469,2497; crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs:107-189 |
| FND-002 | medium | AC-186 requires "all four end-coordinate presence combinations". The tests assert two: endColumn only, and endLine only. Both-present and both-absent are never asserted as retained (the both-absent value appears only as a distinct origin that forces None). The endLine-only check is a `matches!` with `..`, which ignores sourceIdentity, path and startColumn. A mutant that drops `end_line` when `endColumn` is present, or swaps path and identity on that branch, survives. | crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs:5-18,59-87,232-237 |
| FND-003 | medium | AC-192 retention charges for a located declaration are unpinned. The exact-work and work-minus-one oracles re-measure `used` from the same build, so they still pass when the identity charge (model_members.rs:1196) or `charge_origin` (intake_origin.rs:535-562, called at :571) is deleted. Only the group missing-candidate visit charge has an independent oracle. An uncharged copy of an adversarially large origin would regress silently. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1196; crates/quire-contract-model/src/checked_package/v2/model_members/intake_origin.rs:535-573; crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs:344-396 |
| FND-004 | low | The AC-189 salvage cases are weak. The both-branch case uses two empty branches, so a "take the valid branch" salvage never has one to take. The partial source branch appears only in a group against a valid origin, where any salvaged value still differs and yields None. No partial or unknown-member generated branch is tried. Shared-decoder tests under AC-173 reduce the risk. | crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs:238,287-294 |
| FND-005 | low | Weak assertions. `matches!(.., Err(SelectionFailure::Limit(_)))` on the admitted-document work-minus-one case ignores the limit kind, limit, consumed value and pointer. The refused case right below asserts all four. The endLine `matches!` with `..` is also noted in FND-002. | crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs:77-86,354-357 |
| FND-006 | low | `OriginView::Generated` keeps `inputs: &[Value]`. `into_owned` then re-checks `as_str` through an `Option` path that cannot fail, and would silently turn into "no origin" if `decode` ever diverged. Carrying the validated strings, or documenting the invariant at `decode`, would make the impossible state unrepresentable. | crates/quire-contract-model/src/checked_package/v2/model_members/intake_origin.rs:409-423,516-528 |

## Mutant audit (brief item 1)

- Picks a winner among distinct valid origins: killed. Distinct valid
  origins in both permutations, for conflicting-binding and for bad-kind
  groups, must yield None.
- Keeps an origin when one candidate is missing or malformed: killed, in
  both permutations.
- Skips missing-origin candidates in the walk: killed by the independent
  `three_work > two_work` oracle. It holds because parsing stays in one
  1024-byte work band and `read_semantic_ir` charges nothing per type
  before the group refusal. This assertion is unrun at head.
- Wrong context for nested declarations: owner substitution is killed for
  field, operation, parameter and relationship. A stale nested context
  leaking into a later defect is not killed (FND-001).
- Synthesizes a span or sentinel for generated origins: the type makes this
  impossible.
- Retains an origin on `CheckedPackageRefusal` or the public path: there is
  no field to hold it (AC-193 Inspection).
- Releases late or leaks: the values are owned `Box` data, read after the
  test drops its input and after intake drops its document. They are
  dropped explicitly in the mapping arm.
- Picks the earlier relationship in AC-196: killed, under one owner and
  across Gadget/Widget owners with document order inverted.

## Verdict

Not merge-ready. The production change is correct on every path traced. It
keeps one origin grammar owner, adds no public surface, charges only to the
existing budget with bounded iterative walks, and has a spec-sanctioned
explicit teardown. Three medium test gaps leave named AC clauses (AC-186's
four combinations, AC-188's context resets, AC-192's located charges) unable
to kill a plausible regression. The final head's production charge and its
two new assertions have not run. FND-001 to FND-003 must be fixed and the
full gates run twice before merge.

## Validation

Static review only. `quire validate` on this file is recorded in the review
report.
