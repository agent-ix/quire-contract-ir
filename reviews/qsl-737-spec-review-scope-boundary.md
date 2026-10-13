---
id: SR-8685
title: QSL-737 scope-boundary review of operation-call argument record conformance
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir@70f0c54b97ab22981edf8bb0447e0fc59156a162; spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md,
  spec/checked_package/functional/FR-038-consume-checked-package-v2.md
review_set: subset
---

## Summary

Ticket: QSL-737. Checks where the reader's obligations under the record-conformance change start and stop. The inputs are QSL FR-359 (items 2, 3 and 6, cross-repo requirements 4 and 7) and the pending QSpec FR-322 amendment (FR-322-AC-31 as amended, FR-322-AC-69). The rule is compared with every place the V2 reader on origin/main 6e45fd57 compares a record type node exactly.

## Verdict

**CONDITIONAL**: the operation-call boundary is right, but one neighbouring reader check that the same QSpec amendment relaxes has no IR requirement.

What holds. Calls: FR-347 covers the operation-call argument rule exactly. `quire.op.function.call` needs no IR change, because the reader checks no callee parameter types (no constraint, and no check in operations.rs). Slots: `inner_type` and `uniform_rest` are not enforced by this reader (operations.rs:1311-1323), so a collection element or literal gains no new refusal. Equality: `same_type` stays exact under both sources. Events: deferring them until a `model`/`event_type` node exists matches FR-359 cross-repo requirement 7.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-347 replaces the exact comparison "for a record-typed parameter and for nothing else". The pending QSpec FR-322 amendment (FR-322-AC-69) also relaxes `member_of` for `quire.op.record.project`: a field `D` declares or inherits may be read from an operand typed at a record that conforms to `D`. The reader's model-owned field check stays exact. `check_model_member` refuses at `arguments/0` whenever the operand's type node is not the `field.declaration` node (operations.rs:2099-2101). QSL FR-359 item 2 has a field read carry its declaring record's key. So when QSL reads an inherited field on a child-typed operand and names the declaring ancestor, the reader refuses, and no IR requirement says otherwise. Cover the `member_of` relaxation in this PR: widen FR-347 or add a sibling FR with its own AC. Alternatively, record why the projection side cannot arise from QSL's emitter. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:35 |

## Coverage

Each reader site was set against FR-359 and the QSpec amendment: operation-call arguments (`check_model_member` operation arm), model field projection (`check_model_member` field arm), source-declared projection (`check_field_member`), function call, collection `inner_type`/`uniform_rest`, `same_type`, the receiver, and `result_type`. The `Reference<P>` open question sits correctly with FR-322's reference-conformance owner. The pending QSpec call rule keeps references exact, which matches FR-347-AC-4. Review-only, no edits or builds. The QSpec amendment is cited by FR/AC id only.

## Examined units

```yaml
scope:
- id: FR-347
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: This requirement replaces that exact comparison for a record-typed parameter and for nothing else.
- id: FR-038-model-owned-members-step-4
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An operation call's argument types are compared with its parameters' by node key, except that an argument whose record conforms to a record-typed parameter's record is admitted (FR-347).
- id: FR-347-scope
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: Only record_value_type is covered. QSL FR-359 also treats an event type as a record value, but the reader admits no model/event_type node until QSpec adds one (FR-359 cross-repo requirement 7); event conformance joins this rule with that form.
- id: FR-347-open-question
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: Whether the reader should also accept a conforming argument at a Reference<P> parameter. FR-322 Reference conformance owns reference conformance; this requirement leaves it unchanged.
bindings: []
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | Since the first review, QSpec FR-322-AC-69 (the amendment for QSL-737) also admits a conforming record at a `case` arm body against the `case` result type and at a `value`/`union_value` payload position. The reader compares both exactly today: `union_arms_hold` compares each arm body's type node with `result_type` (operations.rs:1451-1452), and the union-value check compares each payload term's type node with the member's payload type (operations.rs:1574-1578, `type-mismatch`). FR-347 still says "It changes nothing else", and no IR requirement covers these two positions. QSL FR-359 item 5 puts a child in union payloads. Cover them in this PR, or state them as dependencies with the QSL-737 ticket. The function-body root against the function's type, also in FR-322-AC-69, was not traced in the reader. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:35-39 |
| FND-003 | low | The new "Function-call arguments" section states a target rule with no AC, and defers it to "a later amendment" with no ticket, Open Question or dependency edge naming the blocking QSpec criterion (FR-322-AC-70). Nothing tracks the deferral. Add a Dependencies entry naming FR-322-AC-70 and the tracking ticket. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:143-153 |

## Dispositions

Round 1; reviewed 66a261051c487bebfe2637e155d815f4d308f68d.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 66a261051c487bebfe2637e155d815f4d308f68d |

## Disposition round 1 verdict

**CONDITIONAL**: FND-001 is fixed. FR-347 now relaxes the field-read operand (`member_of`) by the same rule, through new AC-9. This agrees with QSL FR-359 item 2 (the read names the declaring record and is resolved among `P`'s effective members) and with QSpec FR-322-AC-69. The ordering is consistent with `check_model_member`: the operand check comes before the name resolution at `arguments/0`, so an unrelated operand or object operand refuses at `arguments/0`, and a descendant-only field named on `P` refuses at `name`. See SR-8683 FND-004 for the object-type gate this also needs. New FND-002 (medium) and FND-003 (low) are open.
