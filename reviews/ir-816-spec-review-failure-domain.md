---
id: SR-11802
title: IR-816 failure-domain review of receiver-scoped frame entries and deletes_scoped (FR-040-AC-17..29)
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir@9526be4cce3d637705b22ec4804705ff718dcb3a (PR 415, diff vs origin/main 32e433a); spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-053-checked-package-v2-frame-bodies.md, spec/checked_package/matrix/tests.md, spec/spec.md, spec/tests.md
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-816. Failure modes, identity confusion and edge cases in the new frame rules of PR 415 at 9526be4: conflict location, listing-order dependence, shared frames, unresolved contexts and step precedence, against QSpec FR-340/FR-342 and QSL TC-1043 rows X6, X8, X9, I8 and step 2b. Review only; no edits.

## Verdict

**Mergeable after the base findings**, on this lens: one medium and one low finding, neither blocking alone.

What holds:

- Step precedence is total and matches FR-340/FR-342: schema (AC-18..20) before stale key (AC-26) before frame step (AC-22..25) before state step (AC-27..29). AC-29 pins frame-step-over-state-step and TC-1043 X6 pins "in the state step, not in the frame step".
- `conflicting-binding` after other meaning-join defects matches TC-1043 I8's rationale ("the conflict is checked once every entry resolves") and X8/X9.
- The order-defect locus (frame node `body`, frame node key) is one refusal for every member (frame.rs:448-462), so no tie between a misordered `deletes` and a misordered `deletes_scoped` can change the reported refusal.
- Shared-frame scoped-deletes failure is deterministic: ascending anchor node-id digest order, first failure (AC-28), matching FR-342 check 5 "checked against each anchor's operation".
- The reference-parameter definition (`Reference<T>` for an object type `T`) matches TC-1043 I5 (`n: Integer` refuses).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The conflict's path is "the later of the two entries in array order", so TC-1043 X8 and X9 (the same two entries in two listings) refuse at different paths. QSpec FR-340 says the scope component "fixes only the order in which the two entries of a conflict are named" (every-object first). That is an order-key rule, independent of listing. Choosing the entry that sorts second by order key (the `receiver` one) would make the path follow QSpec's naming order and be the same for X8 and X9. The IR text gives no reason for preferring array order. FR-346's "second entry in body order" convention has no order key to defer to. Either adopt the order-key choice or state why array order is kept. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:160-168, 322 |
| FND-002 | low | FR-040 skips the scoped-deletes check whenever the anchor's `context` is not a model declaration node. A source-declared context's frame can then carry a `parameter` entry naming no parameter and admit. QSpec FR-342 check 5 has no such exemption: "every operation whose anchor references the frame". The skip follows from IR's existing unresolved-operation reading, but the text does not label it an IR reading as FR-040 does for the conflict locus, and no criterion pins it. Label it an IR reading and add it to AC-27, or say how such a frame is handled. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:208-213 |

## Examined units

```yaml
scope:
- id: FR-040-AC-23
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). Two `field` entries sharing a declaration and name and differing in scope refuse `invalid_model_binding`/`conflicting-binding`, with the later entry in array order as the path, the declaring node key as the locus and both entries named, whichever order they are listed in.
- id: FR-040-AC-24
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame holding a `conflicting-binding` defect and a canonical-order defect reports the `conflicting-binding` and not the `invalid_semantic_graph` of the misordered array.
- id: FR-040-AC-25
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame holding a `conflicting-binding` defect and any other meaning-join defect reports the other meaning-join defect.
- id: FR-040-AC-26
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `scope` or a `deletes_scoped` entry changed in a frame node left under its old key refuses `invalid_package`/`stale-node-key` before the frame step.
- id: FR-040-AC-27
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `parameter` entry of a frame's `deletes_scoped` naming no reference parameter of the operation an anchor referencing the frame resolves refuses `missing_declaration`/`missing-name`, located at the frame node, in the state step after that anchor's own joins hold, and not in the frame step.
- id: FR-040-AC-28
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). Two anchors of one `context` referencing one `frame` node both admit, and the frame's `parameter` entries are checked against each anchor's operation, the first failure in ascending anchor node-id digest order being reported.
- id: FR-040-AC-29
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A package carrying a frame-step defect and a `parameter` entry that fails the state step's scoped-deletes check reports the frame-step defect.
```
