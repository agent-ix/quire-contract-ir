---
id: SR-11803
title: IR-816 EARS conformance review of receiver-scoped frame entries and deletes_scoped (FR-040-AC-17..29)
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir (PR 415, diff vs origin/main ); spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-053-checked-package-v2-frame-bodies.md, spec/checked_package/matrix/tests.md, spec/spec.md, spec/tests.md
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-816. EARS and AC-quality pass over FR-040-AC-17..29, the edited AC-1 and AC-7, and TC-056's new procedure in PR 415. Review only; no edits.

## Verdict

**Mergeable after the base findings**, on this lens: one medium and one low finding.

What holds:

- The direct-assertion wording ("A ... refuses X at Y") matches the neighbouring FR-040-AC-1..16 and avoids the validator's `ac:non-canonical-shape` warning. `quire validate` emits no AC-shape warning for FR-040.
- AC-17, 18, 19, 21, 24, 25, 26, 27, 28 and 29 each state one observable outcome with code, cause and locus, and are testable from TC-056's procedure. AC-21 carries a concrete admit/refuse pair, matching TC-1043 X4.
- TC-056 names TC-1043 W1-W6 and X1-X9 by id only and re-authors the cases from IR's own vocabulary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-20 bundles four refusals "located at that entry or member" without saying which locus each case gets. The repeat case is ambiguous three ways. FR-040's prose (:94-96) says a repeat refuses "at that member". FR-038 says "at that member's own path". The reader's `reject_repeat` (frame.rs:150-158) points at the second occurrence's index. Two implementers will assert different pointers. State the exact pointer per case, with the repeat at the second occurrence as the reader does today. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:319, 94-96 |
| FND-002 | low | AC-22's "then by scope with `every-object` first" is unobservable. Two entries with equal declaration and name are either a schema repeat or a `conflicting-binding`, and the conflict outranks order (AC-24), so no admitted or order-refused array ever depends on the scope component. FR-040's own prose says so ("places no admitted entry"). The example only tests name order. Drop the scope clause from AC-22, or restate it as what it actually fixes (the conflict naming order, see SR-11802 FND-001). | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:321 |

## Examined units

```yaml
scope:
- id: FR-040-AC-1
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `modifies` array holding a `relationship` entry and a `field` entry, each naming an eligible declared dependency, admits; an all-empty frame body of four empty members admits; a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` with no cause, located at that entry.
- id: FR-040-AC-17
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits.
- id: FR-040-AC-18
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `field` entry with no `scope`, a `field` entry with a `scope` other than `receiver` or `every-object`, and a `relationship` entry carrying `scope` each refuse `invalid_semantic_graph` with no cause, located at that entry, before the frame step.
- id: FR-040-AC-19
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A frame body with no `deletes_scoped` member refuses `invalid_semantic_graph` with no cause, located at the frame node's `body`, before the frame step.
- id: FR-040-AC-20
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `deletes_scoped` entry of a `kind` other than `receiver` or `parameter`, a `parameter` entry with no `name`, an entry carrying a further member, and an array repeating an entry each refuse `invalid_semantic_graph` with no cause, located at that entry or member, before the frame step.
- id: FR-040-AC-21
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). A `deletes_scoped` that does not hold the `receiver` entry first and the `parameter` entries ascending by name as UTF-8 bytes refuses `invalid_semantic_graph` with no cause, located at the frame node; `receiver`, `parameter a`, `parameter b` admits and `parameter a`, `receiver`, `parameter b` refuses.
- id: FR-040-AC-22
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-816). The `modifies` entries of one declaring node ascend by name, then by scope with `every-object` first; entries (`step`, `receiver`) then (`value`, `every-object`) admit and the reverse order refuses `invalid_semantic_graph` with no cause, located at the frame node.
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

## Dispositions

Round 1, reviewed the fix.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-20 now locates each case at the offending entry, and a repeat at its second occurrence. FR-040 :94-96 and FR-038 say the same, and so does `reject_repeat` (frame.rs:150-158). |
| FND-002 | fixed | AC-22 now asserts name order only; the conflict naming order is carried by AC-23. |
