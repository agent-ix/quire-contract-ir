---
id: SR-1557
title: "Failure-domain review, disposition round 1, of quire-contract-ir PR #295 (IR-627 derived-shape key re-derivation)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@ab860cac3a7162c5eea073d99bb5bcf5433e1237; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Anonymous structural node bodies and keys', FR-038-AC-107, FR-038-AC-112, FR-038-AC-123..132), spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md"
review_set: subset
---

## Summary

Ticket: IR-627. These are new findings from disposition round 1 against fix commit ab860ca, which
replaced the arithmetic body check with re-derivation, at admission, of the ten node shapes that
`MemberType::node_key` derives. Confirmed from code:
- **The ten shapes are enumerated correctly** (model_members.rs:444-492): scalar boolean and
  integer, composite reference, option, set, bag, sequence and ordered_set, and bounded_domain
  integer_range and collection_bounds.
- **The stage is placeable as specified.** validate_graph runs `validate_structural_nodes`, then
  `validate_application_keys`, then `validate_nominal_nodes`, then the declaration, frame, state,
  temporal, abstraction and operation steps (mod.rs:1726-1760).
- **The stage visits every node of the decided shapes,** whether or not anything references it.
- **QSL's preimage reproduces.** I recomputed QSL FR-092's Integer vector (07f6dca9...) with an
  independent JCS hash.

The same hash shows that QSpec's own positive fixtures carry placeholder keys on derived-shape nodes,
and the decided stage would refuse them. Ranges reached through gated referrers also remain open,
despite the text saying "every route" is closed.

## Verdict

Not mergeable: one high and one medium new finding, plus one low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The decided stage contradicts FR-038-AC-107 and AC-112. QSpec's `positive-all-families.json`, `positive-clause-operations.json` and `positive-union-nodes.json` carry derived-shape nodes with placeholder keys: `scalar_type`/`integer` keyed `7f7f...`, `composite_type`/`reference` keyed `b2b2...`, and `scalar_type`/`boolean`, none of them the QSL FR-092 key (recomputed). Items 1 and 4 refuse each of these `stale-node-key`, but AC-107 requires the three fixtures to admit. AC-112 applies `adverse.json` to `positive-all-families.json`, and every expected outcome from a later stage is pre-empted by `stale-node-key`, because the new stage runs before the nominal key stage. The in-repo fixtures (tests/it/support/checked_package.rs, the self-typed `aaaa` scalar) are affected the same way. The spec neither amends AC-107 and AC-112 nor routes the divergence to QSpec, whose fixtures assume anonymous keys are unchecked | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1786-1803, :2493, :2497 |
| FND-002 | medium | "This one rule closes every route to a tampered range" overclaims. Re-derivation makes each derived-shape node truthful for its own key, but a gated referrer can still be re-pointed at a genuinely keyed narrower node. Examples: a `parameter` value node's `semantic_type`, which the scalar-operand route reads through `operand_type_node`; an `alias`, `record`, `tuple` or `union` composite; and a declared, named `bounded_domain` such as `type Small = Int[0, 9]`. These routes stay open until AC-131's gate lifts. The text should state them as the gated limit rather than claim closure | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1815-1827, :1838-1857, :2516 |
| FND-003 | low | "Valued by a canonical decimal string" is undefined for signed bounds. AC-128 needs `-170141183460469231731687303715884105728` to admit, but the reader's existing integer grammars are non-negative only (structural.rs:671-681), and `collection_bounds` bounds must not be negative. State the grammar per form, for example `^(0|-?[1-9][0-9]*)$` for `integer_range` and `^(0|[1-9][0-9]*)$` for `collection_bounds` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1805-1813, :2509, :2513 |

## Dispositions

Round 2, reviewed at 3c2fe37b2444da05b109c83d4274da0005f04b9a (fix commits a2bd266, d0b4cd7, 3c2fe37).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 3c2fe37b2444da05b109c83d4274da0005f04b9a |
| FND-002 | fixed | a2bd2663fb59cd430006c70432d7349a95792415 |
| FND-003 | fixed | a2bd2663fb59cd430006c70432d7349a95792415 |
