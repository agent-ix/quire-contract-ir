---
id: SR-11805
title: "IR-816 receiver-scoped V2 frame reader code review (with Rust review)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #416 diff origin/main...a38dbe7: crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/state.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_abstraction_relation.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_union.rs, tests/it/support/checked_package.rs"
review_set: subset
---

# IR-816 receiver-scoped V2 frame reader code review (with Rust review)

## Summary

Ticket: IR-816. PR: quire-contract-ir#416 at a38dbe7aa03b8e7992cbdd6fce1e1f8bddb2fbf3. This is the independent code review, with the rust-review lane folded in. It is not the coder's own pre-handoff review, which is `reviews/26-10-16-ir-816-frame-reader-rust-review.md` (SR-11804). Authority used: FR-040 at this head (AC-17 to AC-30, Frame body, Operation anchor body, Reader order); merged QSpec FR-340 and FR-342 (quire-specification origin/main); and QSL TC-1043 W and X rows (quire-spec-language origin/main), used as the oracle.

Focused gates, run with `CARGO_TARGET_DIR` set to the reviewed worktree's own `target/` and with `-j2` under `flock /tmp/luna-ir-heavy.lock`:

- `cargo fmt --all -- --check`: clean. rustfmt leaves frame.rs:536-537 over the 100-column limit without reporting it (FND-004).
- `cargo clippy -p quire-contract-model --all-targets -D warnings` and `cargo clippy --test it -D warnings`: clean.
- `cargo test --test it -- tc_056`: 25 passed.
- `cargo test -p quire-contract-model --lib checked_package::v2`: 170 passed.
- `cargo test --test it` (whole suite): 606 passed and 1 failed. The failure is `selected_population::duplicate_max_on_reader_admitted_population_is_not_population`, which panics with "requires QUIRE_SPEC_LANGUAGE_DIR". It depends on the environment and is not caused by this diff.
- 15 source mutants were run in a throwaway worktree, which has since been removed. 11 were killed, 3 survived on test gaps (recorded in SR-11806) and 1 is equivalent.
- A throwaway probe test, never committed, reproduced FND-001.

## Verdict

**NOT MERGEABLE.** There is one medium correctness finding (FND-001). The other findings are low.

What holds:

- **Four members, no fallback.** `read_frame_body` requires exactly `term`, `modifies`, `creates`, `deletes` and `deletes_scoped` (frame.rs:131-136). A body in the old three-member shape refuses `invalid_semantic_graph` at `body`. Nothing reads it as an empty `deletes_scoped`. The `deletes_scoped_optional` mutant is killed.
- **Typed variants.** `FieldScope { EveryObject, Receiver }` and `DeletesScopedEntry { Receiver, Parameter { name } }` are typed enums with derived `Ord`. Declaration order gives `every-object` before `receiver` and the receiver entry before parameter entries. `Box<str>` compares names by UTF-8 bytes. A wire scope other than the two spellings refuses (`_ => return Err(invalid())`). No wildcard arm maps to a value, and the `wildcard_scope_default` mutant is killed. The relationship/field `match` in `order_key` is exhaustive. A relationship entry's order key carries an empty name and `None` scope.
- **Validation order.** Shape refusals come from the per-node body loop, so they precede the frame step. A changed `scope` or `deletes_scoped` under an old key refuses `stale-node-key`. In the frame step, the meaning-join defects (`min_by` rank) come first, then the conflict scan, then the canonical-order check. The state step re-checks scoped deletes per anchor, after that anchor's context, frame, type and operation joins. Mutants `conflict_after_order` and `conflict_before_meaning` are killed.
- **Conflict locus.** For one pair, the path is the `receiver` entry's index in both listings (TC-1043 X8 and X9 shapes), and the locus is the declaring node key. The other entry is not carried. Mutants `conflict_at_other_entry` and `conflict_at_later_in_array` are killed.
- **State step.** The parameter check needs a named slot whose type is `MemberType::Reference`. Path and locus are the frame node (state.rs:299-311). The check is skipped when `resolve_operation` returns `Ok(None)`, the unresolved case of AC-30. The `frames` set is gone, so a shared frame is admitted per anchor, and a duplicate (context, operation) pair still refuses `ambiguous-name`. Mutants `shared_frame_ambiguous`, `state_check_unresolved_op` and `first_anchor_only` are killed.
- **Panic and unsafe surface.** There is no new `unwrap`, `expect`, indexing panic or `unsafe` in production code.
- **Edited fixtures.** The three other test files and `support/checked_package.rs` only add `"deletes_scoped": []` or `"scope": "every-object"`, the minimum needed to keep their existing assertions valid under the four-member shape. No assertion is weakened.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The conflict scan returns the first conflicting pair it finds in array order, not the one with the least order key. FR-340 precedence rule 3 and FR-040 ("remaining ties within one member break by ascending order key"; "regardless of ... where in its array an entry sits") require the least. Reproduced with a throwaway probe test: `modifies` `[total/every-object, total/receiver, balance/every-object, balance/receiver]` refuses at `/modifies/1` (total), where the spec requires `/modifies/3` (balance). The canonical listing reports balance. The reported diagnostic therefore depends on the listing. | crates/quire-contract-model/src/checked_package/v2/frame.rs:517-543 |
| FND-002 | low | `let DeletesScopedEntry::Parameter { name } = entry else { continue; };` is a non-exhaustive refutable pattern over `DeletesScopedEntry`. Any variant other than `Parameter`, including one added later, silently skips the FR-342 check-5 test instead of failing to compile. Use an exhaustive `match` with an explicit `DeletesScopedEntry::Receiver => {}` arm, as the brief's no-wildcard rule requires. | crates/quire-contract-model/src/checked_package/v2/state.rs:295-298 |
| FND-003 | low | The state-step scoped-deletes check re-parses the frame body once per anchor and discards its work count (`let (frame_body, _) = read_frame_body(...)`). It then scans `entries × parameters` with nothing charged to the `WorkMeter`. FR-038 requires each node validation to be charged before it proceeds. The work is bounded by package size, but it is uncharged and grows as anchors × frame size. | crates/quire-contract-model/src/checked_package/v2/state.rs:294-305 |
| FND-004 | low | The conflict refusal hard-codes the path `crate::checked_package::shared::CheckedPackageRefusalCause::ConflictingBinding` inline, and lines 536-537 run to 103 and 109 columns. rustfmt gives up on the expression silently, so `fmt-check` passes over it. Import the cause, as state.rs does. The `scope` wire spellings are also matched as inline string literals, unlike `FrameEntryKind::from_wire`. | crates/quire-contract-model/src/checked_package/v2/frame.rs:139-143, 534-539 |

## Examined scope

```yaml
scope:
- id: FR-040-AC-17
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits."
- id: FR-040-AC-19
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame body with no `deletes_scoped` member refuses `invalid_semantic_graph` with no cause, located at the frame node's `body`, before the frame step."
- id: FR-040-AC-21
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A `deletes_scoped` that does not hold the `receiver` entry first and the `parameter` entries ascending by name as UTF-8 bytes refuses `invalid_semantic_graph` with no cause, located at the frame node"
- id: FR-040-AC-23
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "Two `field` entries sharing a declaration and name and differing in scope refuse `invalid_model_binding`/`conflicting-binding` with the declaring node key as the locus and, as the path, that of the entry that sorts second by the order key (`every-object` before `receiver`, so the `receiver` entry), whichever order the two are listed in; the other entry is not carried."
- id: FR-040-AC-24
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame holding a `conflicting-binding` defect and a canonical-order defect reports the `conflicting-binding` and not the `invalid_semantic_graph` of the misordered array."
- id: FR-040-AC-25
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A frame holding a `conflicting-binding` defect and any other meaning-join defect reports the other meaning-join defect."
- id: FR-040-AC-27
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "A `parameter` entry of a frame's `deletes_scoped` naming no reference parameter of the operation an anchor referencing the frame resolves refuses `missing_declaration`/`missing-name`, located at the frame node, in the state step after that anchor's own joins hold, and not in the frame step."
- id: FR-040-AC-28
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "Two anchors of one `context` referencing one `frame` node both admit, and the frame's `parameter` entries are checked against each anchor's operation, the first failure in ascending anchor node-id digest order being reported."
- id: FR-040-AC-30
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: "An anchor whose `context` is not a model declaration node, and so whose `operation` is admitted unresolved, is not checked for scoped deletes: a frame `parameter` entry naming no parameter admits in the state step for it."
- id: FR-340
  path: "agent-ix/quire-specification:spec/objects/interfaces/FR-340-checked-package-frame-location.md"
  role: context_only
  excerpt: "3. Remaining ties, among multiple meaning-join defects in the same member, break by the ascending order key of the offending entry, the same order rule the member array itself must satisfy. This precedence rule fixes exactly one diagnostic for a frame node carrying several defects, regardless of which member happens to carry which defect or where in its array an entry sits."
- id: FR-342
  path: "agent-ix/quire-specification:spec/objects/interfaces/FR-342-checked-package-operation-anchor-body.md"
  role: context_only
  excerpt: "5. Scoped deletes. Every `parameter` entry of the `frame` target's `deletes_scoped` names a reference parameter of the resolved operation, else `missing_declaration`/`missing-name` with the frame node as the locus. A frame shared by several anchors is checked against each anchor's operation."
```
