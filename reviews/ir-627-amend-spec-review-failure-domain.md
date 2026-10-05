---
id: SR-1586
title: "failure-domain review of quire-contract-ir PR #298 (IR-627 recursion-group amendment)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@a5e2e32ea87a3c649a827caf296ef294d243a583; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Derived-shape nodes inside a recursion group; FR-038-AC-136, AC-137, AC-138; Q4), spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md"
review_set: subset
---

## Summary

Ticket: IR-627. Failure-domain lens over `git diff origin/main...HEAD` of PR #298,
re-measured against fresh `origin/main` of quire-spec-language (a9cfe11) and
quire-specification (2b2dd28) and against reader code at the PR head. The key
soundness question was whether a tamperer can set `recursion_group` on a node to
escape re-derivation. They can: admission constrains only "every cyclic SCC
shares one label" (`validate_recursion`, `crates/quire-contract-model/src/checked_package/v2/mod.rs:1847-1960`)
and never "a labelled node lies on a cycle". The existing test
`tc_048_package_id_covers_exactly_the_identity_preimage`
(`tests/it/checked_package_v2_reader.rs:771`, case "recursion group" at :842)
adds `recursion_group: "solo"` to an acyclic node, refreshes the identity, and
asserts admission; it passes at the PR head (run in a throwaway worktree). So the
skip predicate as written reopens the IR-627 tamper hole for any `integer_range`
or `collection_bounds` node, and the "Stated soundness limit" describes only the
label-removal direction.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | high | Skip predicate keys on an attacker-settable wire member. Nothing in admission requires a `recursion_group`-carrying node to be on a cycle (`validate_recursion` only checks that cyclic SCCs share one label), and an existing passing test admits a lone label on an acyclic node. A tamperer adds `recursion_group: "x"` to the `Int[0, 1000]` node, changes `max`, patches `identity_projection` and recomputes `package_id`, and the decided stage skips it. The AC-123 regression still passes, because it never adds the label. A predicate keyed on SCC membership alone is not enough either: `dependencies` is unconstrained on derived-shape nodes (`structural.rs` constrains it only for application, parameter and compound-unit nodes), is outside the decided preimage, and is an edge in IR's cycle graph, so a self-dependency forges a one-node cyclic SCC. Fix: skip only a node that sits in a cyclic SCC of the QSL FR-092 "names" graph (body `reference`, `semantic_type`, `result_type`, literal `type`, member `declaration`; not `dependencies`), whose SCC is exactly the set of nodes carrying that label. Refuse, or verify as ungrouped, every other labelled node. Add AC rows for the lone-label tamper and the self-dependency tamper, each refusing at the node. | FR-038 Decided: skip (line 1889), Stated soundness limit (1899), FR-038-AC-136, mod.rs:1847-1960, tests/it/checked_package_v2_reader.rs:842 | wrong-requirement |
| FND-002 | medium | Probe 3 of AC-137 (the `Sequence<Tree>` body `reference` re-pointed from the record to the `collection_bounds` node) changes topology. The record leaves every cycle (SCC {Sequence, collection_bounds}) but keeps its label. Under today's code it admits, as the AC says. Under any fix of FND-001 it becomes a graph-shape refusal. The AC pins an outcome that depends on the unstated label-implies-cycle rule. State that rule and set the probe's outcome, or replace the probe with one that keeps the SCC intact. | FR-038-AC-137 | wrong-requirement |
| FND-003 | low | The gate rationale says every group QSL forms holds a declared record or function (FR-092 "Groups that collide"), so the reader cannot compute any group digest. QSL FR-092's own golden vector G1 ("an `option` node over itself, a one-member group") has no declared member, and the reader could re-derive such a group today. Either narrow the claim to groups that hold a declared member, or record G1 as an exception in Q4 for the QSL owner. | FR-038 "Why the decided stage cannot verify an in-group node", Q4, FR-038-AC-138 | wrong-requirement |

## Dispositions

Round 1, reviewed at 49b85f5d96d54b655660117c5fff54ba4973eade.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f70b5c: The predicate now requires one of six shapes, the label, a names-graph cycle with dependencies excluded, and component equal to the labelled set. integer_range, boolean, integer and reference are always verified. The lone-label and self-dependency routes are closed. Residual and new defects are in SR-1587. |
| FND-002 | still-open | AC-148 row 3 re-points the Sequence<Tree> body reference to 'another node of the group on the cycle', which can only be the collection_bounds node or the Sequence node itself. That takes the record off the cycle while it keeps its label. Under the new 'component equal to the labelled set' rule the remaining members are then verified as ungrouped and refuse stale-node-key, but AC-148 says they are not refused. See SR-1587 FND-001. |
| FND-003 | fixed | 7f70b5c: G1 is now named as owner-free, in 'Why the decided stage cannot verify a node of a real group', in Q4 and in AC-150. |
| FND-002 | fixed | 4f953d9: round 2. AC-148 row 3 now names the collection_bounds target, and its admission follows from the final predicate: the {Sequence, collection_bounds} component is fully labelled, and the off-cycle record is a gated form. |
