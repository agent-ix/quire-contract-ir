---
id: SR-12422
title: "IR-874 PR 451 spec-review-integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Checked identity and causal clauses for internal consistency and correspondence to QSpec. Two criteria overstate accepted invariants. Ticket: IR-874. Frozen head: e3c91ccff561f7a80402c07d79c594e16957b978.

## Examined units

- **FR-038-AC-282** (`examined`): Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-283** (`examined`): A protocol node derives its `quire.application-node/v1` key from its exact tag, form, Boolean type, declaration path, `null` recursion and body, with occurrences excluded; a second reference, source reformatting or a different source declaration order preserves the key and package id, while a changed name, referenced target, literal or ordered child changes both. Role, channel, finish, terminal, control and compensation declarations use the fixed qualified-name segments of QSpec FR-464; queues have no declaration and generated occurrences, while other protocol members have declaration occurren
- **FR-038-AC-289** (`examined`): A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit causally before its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; the package crea
- **FR-038-AC-290** (`examined`): For each clause the reader constructs the QSpec FR-464 derived causal relation from control containment, sequencing, receive/send, effect/attempt and await edges, not source order or timestamps. A receive before its send or an await anchored at a later control refuses `invalid_package`/`definition-cycle` naming the cycle; a same pair on independent parallel branches admits. A repeat whose body can complete with no progress on a path refuses `unsupported_construct`/`declaration-form`, while a repeat with a real step admits with or without a maximum and loop proof.
- **TC-048** (`examined`): Use QSpec FR-464, FR-465 and FR-466 and their TC-946, TC-947 and TC-948 vectors as the independent expected outcomes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Commit-before-effect criterion rejects admitted choice and non-all-join paths; QSpec rejects only a dominating commit. QSpec FR-466 Compensation scope rule 4 and AC-11 admit a commit in one choice case or non-all join branch before the effect. TC-948 CP9c and CP10b cover positive controls. Say commit that dominates the forward effect, and add positive/negative dominance controls to TC-048. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4891 |
| FND-002 | medium | Unqualified declaration reorder promises stable keys even when ordered captures or sequence children change. QSpec FR-464-AC-3 and TC-946 K1 promise stability only when roles are declared in another source order; sequence children and capture lists retain authored order and change keys when reordered. Narrow the stable-order case to role or other set-backed declarations; keep ordered-list changes as separate key-changing cases in AC-283 and TC-048 step 2. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4885 |

## Verdict

Checked identity and causal clauses for internal consistency and correspondence to QSpec. Two criteria overstate accepted invariants.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR #451 head bd0f2fdef5793d005952505b0e456a629eb76f70 against QSpec FR-464/466 and TC-946/948; the corresponding TC-048 control was updated. |
| FND-002 | fixed | Verified at PR #451 head bd0f2fdef5793d005952505b0e456a629eb76f70 against QSpec FR-464/466 and TC-946/948; the corresponding TC-048 control was updated. |
