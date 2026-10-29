---
id: SR-11806
title: "IR-816 receiver-scoped V2 frame reader gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #416 diff origin/main...HEAD: FR-040-AC-17..FR-040-AC-30, TC-056, spec/checked_package/matrix/tests.md, spec/tests.md, tests/it/checked_package_v2_frame_entries.rs, crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/state.rs"
review_set: subset
---

# IR-816 receiver-scoped V2 frame reader gap analysis

## Summary

Ticket: IR-816. PR: quire-contract-ir#416. This is a planless gap analysis of FR-040-AC-17 to AC-30 against the tagged tests, the code and the status prose. Plan completion was not assessed.

`quoin matrix --json` (quoin 0.28.3) shows every one of AC-17 to AC-30 as `tagged`. The binders are four new tests plus `tc_056_operation_anchors_admit_their_three_bindings_and_joins` for AC-28. No FR-040 row is untagged. The untagged FR-038 rows are pre-existing and are not touched by this diff.

The test oracle was checked by mutation. 15 source mutants ran in a throwaway detached worktree, and only `tc_056` tests were run against each. The throwaway worktree has since been removed.

| Mutant | Result |
| --- | --- |
| conflict check after order check | killed |
| conflict ranked above other meaning-join defects | killed |
| receiver-first order dropped (Parameter < Receiver) | killed |
| `deletes_scoped` optional (old three-member body admitted) | killed |
| wildcard scope default (`_ => EveryObject`, scope optional) | killed |
| conflict located at the every-object entry | killed |
| conflict located at the later entry in array order | killed |
| shared frame back to `ambiguous-name` | killed |
| scoped-deletes check also run for an unresolved operation | killed |
| scoped-deletes refusal path at the anchor | killed |
| only the first anchor checked | killed |
| parameter names not ordered (receiver-first only) | **survived** (FND-001) |
| scoped-deletes refusal locus = anchor key | **survived** (FND-003) |
| reference-type requirement dropped (`&&` to `||`) | **survived** (FND-002) |
| scope dropped from the `modifies` order key | survived. This is an equivalent mutant: FR-040 and FR-340 state that the scope component places no admitted entry. |

No assertion can be skipped: `expect` and `model_expect` panic on any non-refusal, and the admission checks `assert!`/`panic!` on refusal. The one conditional, `if let Some(locus)` in `expect`, is the cause of FND-003, where `model_expect` passes no locus.

## Verdict

**NOT MERGEABLE.** Three medium gaps (FND-001, FND-002, FND-004) leave AC-21, AC-27 and the status claims unbacked or untrue. FND-003 and the low findings are smaller.

What holds:

- AC-17, AC-18, AC-19, AC-20, AC-22, AC-23, AC-24, AC-25, AC-26, AC-28, AC-29 and AC-30 each have a tagged test that fails for the intended reason under the mutants above.
- AC-23 is asserted in both listings, the TC-1043 X8 and X9 shapes. The `[receiver, every-object]` listing is also misordered, so it covers AC-24.
- The change to the AC-7 "a shared frame" case is the change AC-28 prescribes, word for word. The duplicate-pair case keeps its refusal assertion. The edits in the other three test files and in support only add the required member, and no assertion is weakened.
- Observation on TC-1043 X6, which is not a finding against this PR. The code refuses the scoped delete with path `/semantic_graph/nodes/<frame>` and locus the frame node key (state.rs:306-311). That matches FR-040-AC-27 ("located at the frame node") and merged QSpec FR-342 check 5 ("with the frame node as the locus"). QSL TC-1043 Expected Results, Step 4, says "X6 refuses, in the state step at `merge`'s anchor and not in the frame step, `missing_declaration`/`missing-name`". That wording differs and is a separate spec item for QSL.
- Observation, cross-repo. quire-spec-language origin/main has no `deletes_scoped` in any `.rs` file, so QSL-emitted packages will refuse under AC-19 once this merges. `selected_population::duplicate_max_on_reader_admitted_population_is_not_population` (env-gated on `QUIRE_SPEC_LANGUAGE_DIR`) will fail until QSL adapts. Under the IR pairing rule, IR merges on its own gates and QSL adapts afterwards.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-21's parameter-name order is unbacked. The only misorder case is `[parameter a, receiver]`. There is no `[receiver, parameter b, parameter a]` case (TC-1043 X5), which TC-056's procedure claims ("`deletes_scoped` in a wrong order (X4, X5)"). A mutant that checks only receiver-first survives. AC-21's own example `parameter a, receiver, parameter b` is not the case used either. | tests/it/checked_package_v2_frame_entries.rs:194-211 |
| FND-002 | medium | AC-27's "naming no reference parameter" branch is vacuous. The `"n"` case is meant to name a non-reference (Integer) parameter. But `orders_document`'s `scaled` slot is unnamed (`"params": [slot(NATIVE_INTEGER)]`), so `"n"` refuses because no slot has that name, not because of the type rule. With the reference-type check dropped (`&&` to `||`), every test still passes. TC-056 claims "a `parameter` entry naming a non-reference parameter (`n`)". | tests/it/checked_package_v2_frame_entries.rs:2437-2447, 1695 |
| FND-003 | medium | AC-27's frame-node locus is not asserted. `model_expect` compares only code, cause and path, so a mutant that reports the refusal with the anchor's key as locus survives. This is the exact point on which QSL TC-1043 X6 ("at `merge`'s anchor") differs from FR-040-AC-27 and QSpec FR-342 check 5 (frame node as the locus). Pin the locus to the frame node key. | tests/it/checked_package_v2_frame_entries.rs:2233-2247, 2437-2447 |
| FND-004 | medium | The status rows are untrue at this head. spec/tests.md:15 says "FR-040-AC-17 through FR-040-AC-30 are PLANNED / UNRUN (IR-816)". checked_package/matrix/tests.md:16 says "AC-17 through AC-30 are PLANNED / UNRUN", and :41 says "PLANNED/UNRUN (IR-816), with no tagged implementation tests yet". FR-040 and TC-056 Status now call them implemented. | spec/tests.md:15; spec/checked_package/matrix/tests.md:16, 41 |
| FND-005 | low | TC-056's procedure claims fixtures the tests do not author: "an object type with a field `value` and a field `step`, an operation with no parameter, and an operation `merge` with two reference parameters `a` and `b` and an integer parameter `n`". The tests use the families fixture's `balance`/`total` and `Order.scaled` with an unnamed Integer slot. | spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md:70-73 |
| FND-006 | low | Shared-frame prose is stale now that AC-28 is implemented. FR-040 still says a shared `frame` target "refuses as `ambiguous_declaration`/`ambiguous-name` ... this is what the reader does today" and "supersedes ... when it is implemented". TC-056 Status keeps "superseded by FR-040-AC-28 when that is implemented". | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:213-221; spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md:107 |
| FND-007 | low | AC-17's "with each entry's scope and kind exposed" is asserted only as admission. No test reads the admitted node body back and checks the `scope` and `deletes_scoped` kinds. | tests/it/checked_package_v2_frame_entries.rs:194-232 |

## Examined scope

```yaml
scope:
- id: FR-040-AC-21
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A `deletes_scoped` that does not hold the `receiver` entry first and the `parameter` entries ascending by name as UTF-8 bytes refuses `invalid_semantic_graph` with no cause, located at the frame node; `receiver`, `parameter a`, `parameter b` admits and `parameter a`, `receiver`, `parameter b` refuses."
- id: FR-040-AC-27
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A `parameter` entry of a frame's `deletes_scoped` naming no reference parameter of the operation an anchor referencing the frame resolves refuses `missing_declaration`/`missing-name`, located at the frame node, in the state step after that anchor's own joins hold, and not in the frame step."
- id: FR-040-AC-17
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits."
- id: FR-040-AC-18
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A `field` entry with no `scope`, a `field` entry with a `scope` other than `receiver` or `every-object`, and a `relationship` entry carrying `scope` each refuse `invalid_semantic_graph` with no cause, located at that entry, before the frame step."
- id: FR-040-AC-19
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame body with no `deletes_scoped` member refuses `invalid_semantic_graph` with no cause, located at the frame node's `body`, before the frame step."
- id: FR-040-AC-20
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A `deletes_scoped` entry of a `kind` other than `receiver` or `parameter`, a `parameter` entry with no `name`, an entry carrying a further member, and an array repeating an entry each refuse `invalid_semantic_graph` with no cause before the frame step, located at the offending entry and, for a repeat, at the second occurrence of the repeated entry"
- id: FR-040-AC-22
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "The `modifies` entries of one declaring node ascend by name as UTF-8 bytes; entries (`step`, `receiver`) then (`value`, `every-object`) admit and the reverse order refuses `invalid_semantic_graph` with no cause, located at the frame node."
- id: FR-040-AC-23
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "Two `field` entries sharing a declaration and name and differing in scope refuse `invalid_model_binding`/`conflicting-binding` with the declaring node key as the locus and, as the path, that of the entry that sorts second by the order key"
- id: FR-040-AC-24
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame holding a `conflicting-binding` defect and a canonical-order defect reports the `conflicting-binding` and not the `invalid_semantic_graph` of the misordered array."
- id: FR-040-AC-25
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame holding a `conflicting-binding` defect and any other meaning-join defect reports the other meaning-join defect."
- id: FR-040-AC-26
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A `scope` or a `deletes_scoped` entry changed in a frame node left under its old key refuses `invalid_package`/`stale-node-key` before the frame step."
- id: FR-040-AC-28
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "Two anchors of one `context` referencing one `frame` node both admit, and the frame's `parameter` entries are checked against each anchor's operation, the first failure in ascending anchor node-id digest order being reported."
- id: FR-040-AC-29
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A package carrying a frame-step defect and a `parameter` entry that fails the state step's scoped-deletes check reports the frame-step defect."
- id: FR-040-AC-30
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "An anchor whose `context` is not a model declaration node, and so whose `operation` is admitted unresolved, is not checked for scoped deletes: a frame `parameter` entry naming no parameter admits in the state step for it."
- id: TC-1043
  path: "agent-ix/quire-spec-language:spec/test-cases/TC-1043-receiver-scoped-frame-entries-and-scoped-deletes-in-the-frame-node-and-decision.md"
  role: context_only
  excerpt: "X6 refuses, in the state step at `merge`'s anchor and not in the frame step, `missing_declaration`/`missing-name`."
```

## Dispositions

Round 1, checked against fix. Gates and mutants were re-run in a throwaway worktree (removed): `cargo test --test it` with `QUIRE_SPEC_LANGUAGE_DIR` set passed 608, failed 0, ignored 1. All three mutants that survived round 0 are now killed: parameter-name order unchecked, scoped-delete locus at the anchor, and the Reference check dropped. No new finding.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `tc_056_scoped_frame_order_conflict_and_stale_key` adds `[receiver, parameter b, parameter a]` (TC-1043 X5), which refuses `invalid_semantic_graph` at the frame body. The param-order mutant is killed. |
| FND-002 | fixed | `orders_document`'s `scaled` slot is now `named_slot(NATIVE_INTEGER, "n")`, so the `"n"` case refuses on the reference-type rule. The `&&` to `||` mutant is killed. |
| FND-003 | fixed | `model_expect` now returns the refusal. The `n` and `missing` cases assert the locus equals the frame node's digest, and the anchor-locus mutant is killed. |
| FND-004 | fixed | spec/tests.md:15, checked_package/matrix/tests.md:16 and :41 now state that AC-17 to AC-30 are implemented (IR-816) with tagged TC-056 reader tests. |
| FND-005 | fixed | The TC-056 procedure now names the in-repo `Account` fixture (`balance`, `total`) and the selected `Order` fixture's `scaled(n: Integer)` and `reset()`, which are what the tests author. |
| FND-006 | fixed | The FR-040 Operation anchor text drops "this is what the reader does today" and the "when it is implemented" clause. The AC-28 row and TC-056 Status now speak in the present tense. The AC-7 row keeps its shared-frame clause with "(FR-040-AC-28 supersedes the former shared-frame refusal)", which states the supersession explicitly. |
| FND-007 | fixed | `admitted_frame_body` reads the admitted package's frame node. The test asserts `deletes_scoped` kinds `receiver` and `parameter`, and `modifies` scopes `receiver` and `every-object` with kind `field`. |
