---
id: SR-11801
title: IR-816 integrity review of receiver-scoped frame entries and deletes_scoped (FR-040-AC-17..29)
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir (PR 415, diff vs origin/main ); spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-053-checked-package-v2-frame-bodies.md, spec/checked_package/matrix/tests.md, spec/spec.md, spec/tests.md
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-816. Integrity (completeness, consistency, atomicity) of PR 415 across FR-040, FR-038's frame text, FR-344, FR-346, TC-053 and TC-056. Every other place that lists the frame member set was searched (`exact_members`, "three members", "Three empty members", FR-344:36-50 and:74-82, FR-038-AC-12..15). Review only; no edits.

## Verdict

**NOT MERGEABLE**: one medium consistency finding in FR-038's own frame precedence, plus one low.

What holds:

- FR-038's frame body paragraph now names four members, "Four empty members are admitted", and delegates scope/`deletes_scoped` to FR-040. FR-038's Dependencies line is updated.
- FR-344 (:36-50,:74-82) is still true: it cites FR-340 as the frame carrier and `exact_members` as a closed-set match, neither of which is member-count specific. FR-346:77 "three members" is the abstraction-relation body and is unrelated.
- FR-038-AC-12, AC-13 and AC-15 are unaffected: they concern `creates`/`deletes` eligibility, the dependency join and the cross-frame visit order.
- Code is untouched: `v2/frame.rs:109` still has `exact_members(object, &["term", "modifies", "creates", "deletes"])`. All new criteria are marked PLANNED / UNRUN (IR-816).
- The reader-order statement (state step: placement, anchors including check 5, clauses) is consistent with FR-342's Reader order and FR-342-AC-5.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038's frame precedence prose (:3953-3962) and FR-038-AC-14 still say any `invalid_model_binding` meaning-join defect outranks order and that, among meaning-join defects, the member decides first (`modifies` before `creates` before `deletes`). With this PR, a `conflicting-binding` (an `invalid_model_binding` in `modifies`) ranks after every other meaning-join defect. So a frame with a `modifies` conflict and a `creates` `missing-name` now reports the `creates` defect (FR-040-AC-25, QSpec FR-340-AC-16), which contradicts FR-038-AC-14 as written. The PR updated FR-038's body paragraph but not its precedence paragraph or AC-14. Fix: qualify both with the `conflicting-binding` exception, or point them at FR-040's precedence. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3953-3962, 4176 |
| FND-002 | low | FR-038 now adds `.deletes_scoped` to the paths of the sentence "an entry... that is not a node key, refuses...; an entry naming a node key outside the reader's node-identity domain refuses as `digest_domain_mismatch` at that same member path". A `deletes_scoped` entry is never a node key, so both clauses misdescribe it (the same was already loosely true of `modifies`). FR-040 owns `deletes_scoped`'s shape refusals. Limit the node-key clauses to `creates`/`deletes` and keep only the repeat rule general. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3900-3906 |

## Examined units

```yaml
scope:
- id: FR-038-AC-12
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `creates` or `deletes` entry's eligibility is exactly `model`/`object_type` and `model`/`process`...; the `modifies` entry shape and its eligibility are FR-040's.
- id: FR-038-AC-13
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An entry naming a digest that is not among the frame node's own `dependencies` refuses as `missing_declaration` with cause `missing-name`, located at that entry...
- id: FR-038-AC-14
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A frame node carrying both a meaning-join defect and a canonical-order defect refuses for the meaning-join defect; among two meaning-join defects in different members, the earlier member (`modifies` before `creates` before `deletes`) is selected regardless of which defect's entry digest is lower; among two meaning-join defects in the same member, the lower entry digest is selected;...
- id: FR-038-AC-15
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package carrying two defective `state`/`frame` nodes refuses at the one with the lower `node_id` digest, reporting only that frame's own defect...
- id: FR-040-AC-1
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `modifies` array holding a `relationship` entry and a `field` entry, each naming an eligible declared dependency, admits; an all-empty frame body of four empty members admits; a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` with no cause, located at that entry.
- id: FR-040-AC-17
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits.
- id: FR-040-AC-19
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame body with no `deletes_scoped` member refuses `invalid_semantic_graph` with no cause, located at the frame node's `body`, before the frame step.
- id: FR-040-AC-20
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `deletes_scoped` entry of a `kind` other than `receiver` or `parameter`, a `parameter` entry with no `name`, an entry carrying a further member, and an array repeating an entry each refuse `invalid_semantic_graph` with no cause, located at that entry or member, before the frame step.
- id: FR-040-AC-25
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame holding a `conflicting-binding` defect and any other meaning-join defect reports the other meaning-join defect.
- id: FR-040-AC-29
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A package carrying a frame-step defect and a `parameter` entry that fails the state step's scoped-deletes check reports the frame-step defect.
```

## Dispositions

Round 1, fix on.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038's precedence paragraph (:3962-3965) and FR-038-AC-14 now both carry the `conflicting-binding` exception, citing FR-040-AC-25. |
| FND-002 | fixed | FR-038:3901-3909 now limits the node-key and node-identity-domain clauses to `creates`/`deletes`/`modifies`, states that `deletes_scoped` entries are never node keys and that FR-040 owns their refusals, and locates a repeat at the second occurrence, as `reject_repeat` does. |
