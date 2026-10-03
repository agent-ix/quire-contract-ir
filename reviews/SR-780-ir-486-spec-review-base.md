---
id: SR-780
title: "base and correctness review of PR 255 (IR-486 equality over a recursive record is admitted)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@154fc38ef518b036ef68ca1bb25fd6911846750d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Operation leaves, Recursive compared types, deviations paragraph, AC-43 amended, AC-69..71); crates/quire-contract-model/src/checked_package/v2/operations.rs (LeafWalk, check_leaf_count), v2/mod.rs (WorkMeter), shared.rs (CheckedPackageReadLimits::bounded), common.rs (on_stack_for); quire-spec-language origin/main FR-093 (Text leaves, AC-11, AC-19); quire-specification origin/main proposals/checked-package-v2/schema.json LeafSegment"
review_set: subset
---
# SR-780: base and correctness review of PR 255 (IR-486 equality over a recursive record is admitted)

## Summary

Ticket: IR-486. Base and correctness, checked against the reader code at the reviewed sha, QSL
FR-093 on origin/main (read-only) and QSpec's published v2 schema.

What I confirmed independently:

- Code today. `LeafWalk::enter` (operations.rs:1873-1909) checks the global per-node `memo` first,
  then `on_stack` (the composites on the current path), and returns `LeafWalkEnd::Cycle` on a
  revisit of a node still on the path. `check_leaf_count` maps `Cycle` and `Unresolved` to
  `ill_typed`/`operator-ineligible` at `operation.leaves` (2205-2209). The count (`count`,
  1911-1946) and the derivation (`first_leaf_fault`, 1955-2027) are iterative over heap stacks.
  Every `enter` and every derivation step charges one unit to `WorkMeter`, whose exhaustion is
  `incomplete` for `Work` (mod.rs:465-480), propagated unchanged (2210). The PR's descriptions of
  the walk, budget, memo and refusals are true of the code. One correction to the PR body: the
  cycle refusal and AC-43's cyclic clause came from #238 (IR-483, 0a889f9a), not #240. #240 kept
  them and added the exact-leaves derivation. The unit test that pins the refusal today is
  `tc_048_leaf_walk_refuses_a_cycle_and_an_unresolved_node` (operations.rs:4318).
- `OperationLeafWire.path` is a plain `Vec<Box<str>>` (operations.rs:312-318). The reader does not
  validate segments against a pattern, so a `recursion:<d>` segment reaches `check_leaf_count`.
- QSL FR-093 Text leaves, rule 3: at a reentry into an open composite C entered at `d` segments,
  QSL appends `{path: p + "recursion:d", laws: [], mode: null}` whenever a `Text` type is
  reachable from C. A recursion leaf "stands for every text leaf below its path", and the list is
  lossless. AC-11 gives `Two`: both `Node` paths, each with its own `recursion:1` leaf.
  E14/E16/E17 give the Node, A/B and Option<Node> lists. FR-322's QSpec schema `LeafSegment`
  pattern admits only `field:`, `position:` and `inner`, so QSpec's schema rejects the recursion
  segment, as the spec says.
- The path set versus the planner's "visited set". A global visited set would skip `Two.y`'s Node
  and contradict QSL AC-11. The path set ("open composites", line 707) is the reading that agrees
  with QSL. It is coherent: each path enters a composite at most once, so the expected list is
  finite and well defined. One gap remains, for cycles with no record or tuple (FND-002).
- Admitting the recursion leaf is necessary. QSL emits one for every text-bearing recursive case
  (E14-E17, AC-11), so refusing it would refuse QSL's corpus. Making it optional is the author's
  choice and is not required by the decision (FND-004).
- Refusal codes for a malformed recursion leaf. A wrong path or laws gives `operation-law-mismatch`
  at `path`/`laws`, and a present mode gives `operation-mode-mismatch` at `mode`. That fits the
  reader's taxonomy: path and law-shape faults are law-mismatch, and mode-shape faults are
  mode-mismatch at `mode`.
- AC-43's amendment is explicit, and the PR body and the matrix justify it. Nothing else in AC-43 is
  weakened.
- AC-71's numbers against `CheckedPackageReadLimits::bounded()` (shared.rs:46-56): bytes 1 MiB,
  nodes 10 000, edges 100 000, work 1 000 000. Ten all-referencing records unfold to about 10^6
  composite entries, roughly 10^7 work units, so the default work limit is exceeded. For the
  20000-record cycle see FND-005. `on_stack_for` (common.rs:266-273) runs a read on a stacker
  segment of 256 KiB plus 4 KiB per JSON level. A recursive walk 20000 frames deep would still
  overflow that segment, so the 256 KiB stack clause can still catch a recursive walk.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The memo condition is ambiguous, and the natural reading counts wrong. "The count memoised per type node applies only where the node's subtree reaches no reentry to a composite open above it" can be read as a property of the node, decided once when its walk meets no reentry to an ancestor, or as a check at each use site against the composites open there. Only the second is correct. Take X { t: Text; n?: Y }, Y { u: Text; x?: X } and Wrap { y: Y; x: X }. Under `y`, Y's walk reenters only Y, so Y is memoised at 2. Under `x`, X is open above Y and reachable from it, so Y's correct count there is 1. The correct list has 4 text leaves. The reading memoised once gives 5, and keeping today's global memo while replacing `Cycle` with 0 gives 3 (operations.rs:1888-1893 checks memo before on_stack). Fix: say the memo entry for a node applies at a use only when no composite reachable from the node is open at that use (or key the memo by node and the open set). Add the Wrap case to AC-69 (SR-783 FND-001). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:739-742 |
| FND-002 | medium | A cycle with no record or tuple on it is refused by no stated mechanism. The open set holds only `record` and `tuple` nodes (lines 697-700), and reentry is detected only against it. A walk over `T = Option<T>`, or `L = Sequence<L>`, therefore never meets a reentry. It runs until the work budget is spent and gives `incomplete`/`work`, not the `ill_typed`/`operator-ineligible` that this paragraph, AC-43 and AC-70 require. The stated reason, "has no value to compare", is false: `Option<T>` holds `none`, `some(none)` and so on. "Whose only cycle" also leaves open a type with both a record cycle and an option-only cycle. Fix: state the detection. For example, keep options, collections and aliases on the path too, and refuse when one is reentered with no record or tuple entered since it. Apply it to any such cycle, not only the type's only cycle. Give the reason as QSL never writing one, plus there being no composite to anchor `d`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:727-729 |
| FND-003 | medium | The base leaf rules are not scoped to text leaves, so read literally they refuse the recursion leaf the new subsection admits. "Each leaf carries exactly one law", "more supplied leaves than text leaves ... refuse ... at the first extra leaf" and "each leaf's `mode` must be of kind `text_profile`" all apply to `{laws: [], mode: null}`. The subsection says the recursion leaf is not counted, but it does not say how a supplied entry is classed as a recursion leaf (a last segment `recursion:<n>`?). It also leaves three things open: which entries the fewer/more comparison counts, whether "the first extra leaf" indexes the supplied array with recursion leaves included, and what a `Node` list holding only a recursion leaf refuses (`operation-law-missing` by text count, or `operation-law-mismatch` at `path`). Fix: say the count, law and mode rules apply to text leaves, define how an entry is recognised as a recursion leaf, and state the order: recursion-leaf shape, then text count, then path and laws. Index pointers into the supplied array as written. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:661-691 |
| FND-004 | medium | The optional recursion leaf deviates from QSL without saying so. QSL rule 3 always appends the recursion leaf when text is reachable, and calls it meaning-bearing: it "stands for every text leaf below its path", and the list is lossless. This reader admits the list with or without it. So one operation has two admitted byte forms with two node keys, which breaks AC-44's "`operation.leaves` is exactly the derived leaves". The deviations paragraph names only the reference-reader deviations. The planner decision says leaves along a cycle are not counted or required. Whether the QSL marker leaf is required is an interpretation (see the leader questions). Requiring it where text is reachable admits QSL's corpus just as well and keeps the list canonical. Fix, after the leader rules: either require it, so a missing one refuses `operation-law-missing` or `operation-law-mismatch` at its place, or keep it optional and list that as a deviation from QSL FR-093 rule 3 in the deviations paragraph. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:714-721 |
| FND-005 | medium | The 20000-record case cannot admit under the limits AC-71 names. It raises node and work limits only. The default byte limit is 1 MiB (`CheckedPackageReadLimits::bounded`, shared.rs:46-56). 20000 record nodes, each with a 64-hex key, a recursion member and two field references, plus a 20000-segment leaf, come to several MiB. The read then stops `incomplete` for `bytes` before any leaf is walked. Fix: say "node, byte and work limits raised to admit it" (or "every read limit raised"), and the same in TC-048. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:991 |
| FND-006 | low | AC-43 says a type "that reaches itself through a record admits", but the subsection's open composites are records and tuples. AC-43 also names "options and collections" where the subsection says "options, collections and aliases". No criterion has a tuple cycle. Fix: say "through a record or tuple" and match the subsection's list, or add a tuple cycle case to AC-69. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:967 |

## Verdict

Request changes. The decision is implemented in the right shape: admission, the path set, cut
paths, budget-bounded iteration and a stated deviation. Four medium correctness gaps remain: the
memo rule's context condition, detecting a cycle with no record or tuple, how recursion leaves
interact with the base count and shape rules, and the optional-versus-required recursion leaf (a
leader decision). A fifth medium gap is that AC-71's limit set cannot admit its own case. AC-43's
record/tuple wording is low.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | The record-free cycle rule does not say when the note on an option or collection is dropped, and "wherever the walk meets it" reads as global. Node keys are content digests, so one `Option<Text>` node serves every optional text field. Take `R { a?: Text; b?: Text }`. The walk enters that option under `field:a`, leaves it, and meets it again under `field:b` with no record or tuple entered in between. Read globally, the rule refuses this ordinary acyclic record `ill_typed`. The author describes a per-path marker, which is the correct design, but the text does not say so, and no AC case shares one option node between two sibling fields. Fix: say the note is held only while the node is on the current path (dropped when the walk leaves it), so only a revisit on the path refuses. Add a sibling-shared option, such as `R { a?: Text; b?: Text }` with its two leaves, as an admission case in AC-71 (or AC-43). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:849-859 |
| FND-008 | medium | The pointer for a recursion leaf that cannot be placed contradicts AC-71. The general rule (lines 877-880) refuses an entry the pass cannot place at its `path` "when the place holds another leaf", and at `operation.leaves/<i>` "when no place is left". The only exception (lines 881-883) covers a recursion leaf at a reentry that reaches no text, or with another `d`. Over `Node` the reentry is the last place. So `[label, [field:next, recursion:0]]` (wrong prefix) and `[label, rec, rec]` (a second one) both reach "no place left", and the general rule points at the index. AC-71 requires `path` for both. With a reentry being an optional place, the rule also leaves open whether a non-matching recursion leaf there is a mismatch or is skipped. Fix: say that an entry recognised as a recursion leaf that the pass does not consume refuses at its `path`, whatever the cause, and that only text leaves with no place left refuse at the index. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1165; 877-883 |
| FND-009 | low | The count paragraph still says "fewer supplied leaves than text leaves refuse as `operation-law-missing`". The scoping sentence (lines 779-782) covers only "this and the next paragraph", which is the exact-leaves and mode paragraphs, not the count paragraph before them. Read literally, `Node` with only the recursion leaf has 1 supplied leaf for 1 text leaf and is not "fewer". The subsection and AC-70 say it refuses `operation-law-missing`. Fix: say "fewer supplied text leaves". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:775-777 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@2f54f96507fd81b6d1b428cbf1113ad4cd50f035. The PR was rebased onto origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, after #254 merged, and is one commit on it. Its diff against that base is the delta I reviewed; the rebase is excluded. GitHub reports MERGEABLE. `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70 (main: 163/209, 42/67). The new ids are AC-70 to AC-72, and the only FR-038-AC-69 row is #254's. No AC id is duplicated. The diff and the PR body contain no SHA.

Notes on the round-1 checks:

- FND-001: the memo applies at a use only when no record or tuple reachable from the node is open there. AC-70 pins the X/Y/Wrap case at 4 text leaves, with the two wrong counts (5 and 3) named in the prose. Both wrong readings fail the AC.
- FND-002: a record-free cycle is now refused, and mixed and independent cycles are defined. Leaving options and collections out of the open set is the right choice. I traced `Option<Node>`, where the outer option is the group's own option node. The first visit records 0 open composites. The second visit comes after `Node` was entered, so it is not refused. The walk enters it again and reenters `Node` at `d = 1`, giving `inner`, `field:next`, `inner`, `recursion:1`, which is exactly QSL E17. Had wrappers been in the open set, the reentry would be at the option, at another path and `d`. The wording of the mechanism has a new problem, FND-007.
- FND-003: the base rules are scoped to text leaves (lines 779-782). A recursion leaf is recognised by a last segment `recursion:<decimal>`. The order is: count text leaves, then one in-order pass, then law selection, then modes. Pointers index the supplied list. One pointer rule is inconsistent (FND-008), and one count sentence is unscoped (FND-009).
- FND-004: the leaf stays optional, and the four-part deviation is stated, including the QSL FR-093 rule 3 deviation. AC-44 is amended. The key consequence is stated and true of the code. `validate_application_keys` (operations.rs:132-158) digests `application_preimage`, whose `body` is the whole application body, `operation` and its `leaves` included (operations.rs:194-202). A mismatch is `invalid_package`/`stale-node-key` (operations.rs:152-156), as AC-70 says. My note on whether to keep the leaf optional goes to the leader, not here. The planner ruling that it stays optional was relayed by the leader, and I have not confirmed it.
- FND-005: AC-72 now raises byte, node, edge and work limits and says the 1 MiB default is too small. Those match `CheckedPackageReadLimits::bounded()` (bytes 1 MiB, nodes 10 000, edges 100 000, work 1 000 000). Ten records under the default limits still run out of work first.
- FND-006: AC-43 says "record or tuple". AC-70 adds a declared tuple `Pair`.
- The PR body now names #238 as the source of the cycle refusal.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-002 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-003 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-004 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-005 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-006 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |

Round 2, reviewed at agent-ix/quire-contract-ir@7775e142e702e2a3e21a8426eca9356638838889: the delta from 2f54f96507fd81b6d1b428cbf1113ad4cd50f035, on base origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, which is unchanged. GitHub reports MERGEABLE. `make spec` at round 2: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70. The delta adds no SHA. The leader relayed a planner decision, recorded on IR-486, that the recursion leaf is now required wherever text is reachable. I have not confirmed it independently.

Notes on the round-2 checks:

- FND-007: an option or collection is now noted only while it is on the current path. AC-70 and TC-048 admit `R { a?: Text; b?: Text }` with `["field:a", "inner"]` and `["field:b", "inner"]`. The `Option<Node>` path still matches QSL E17: the group's option is entered again after `Node`, with a record entered in between.
- FND-008: a recursion leaf the pass cannot place now refuses at its `path`, whatever the cause. Only a text leaf with no place left refuses at `operation.leaves/<i>`. I traced each AC-71 case against the new order (count first, then one in-order pass). The recursion leaf placed before the text leaf gives `/0/path`. The wrong prefix and the wrong `d` give `/1/path`. A second recursion leaf gives `/2/path`. A recursion leaf on the integer `List` gives `/0/path`. A recursion leaf carrying a law gives `laws`, and one carrying a mode gives `mode`. All agree with AC-71.
- FND-009: the count paragraph now counts derived leaves, text and recursion.

The required-leaf rewrite is coherent. The derived leaves are the text leaves plus one recursion leaf at each reentry into a composite from which text is reachable, and none at a reentry into a text-free composite. That is QSL FR-093 rule 3. A missing recursion leaf is short of the derived count, so it refuses `operation-law-missing`. One form gives one key and one package id, and the earlier "two node keys" text is gone. AC-44 is amended, the deviations paragraph still lists four items (cycles, the recursion leaf against QSpec's LeafSegment schema, floats, unresolved operands), and no stale "optional" or "present or absent" text is left in FR-038, TC-048 or tests.md.

I re-derived each AC-70 list from the stated rules and QSL FR-093 (E14, E16, E17, AC-11), and each one is correct:
- Node: `field:label`, then `field:next, inner, recursion:0`. The text leaf alone, or the recursion leaf alone, is 1 of 2 and refuses `operation-law-missing`.
- Option<Node>: `inner, field:label`, then `inner, field:next, inner, recursion:1`.
- A/B: at `A`, `field:name`, `field:b, inner, field:tag`, `field:b, inner, field:a, inner, recursion:0`; at `B`, the mirror.
- Two: x label, x `recursion:1`, y label, y `recursion:1`, each `d` being 1.
- Tree2: the bounded domain adds no segment, giving `field:kids, inner, recursion:0`.
- Pair: `position:0`, then `position:1, inner, recursion:0`.
- Wrap: `y,u`; `y,x,inner,t`; `y,x,inner,n,inner,recursion:1` (Y was entered at 1); `x,t`; `x,n,inner,u`; `x,n,inner,x,inner,recursion:1`. That is six leaves. The four text leaves alone refuse `operation-law-missing`. Six plus one more text leaf refuses at `/6`, the seventh entry.
- Node with a further text leaf: `/2`.

The memo rule still holds with required leaves. When no record or tuple reachable from a node is open at its use, its subtree's reentries can only target composites opened inside that subtree. Its recursion leaves are then context-free as well.

One new medium finding, below.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-007 | fixed | 7775e142e702e2a3e21a8426eca9356638838889 |
| FND-008 | fixed | 7775e142e702e2a3e21a8426eca9356638838889 |
| FND-009 | fixed | 7775e142e702e2a3e21a8426eca9356638838889 |

Round 3, reviewed at agent-ix/quire-contract-ir@4a0bcc7c336d47f945b83daab5932b784707d092: the delta from 7775e142e702e2a3e21a8426eca9356638838889, on base origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, which is unchanged. GitHub reports MERGEABLE. `make spec` at round 3: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70. The delta adds no SHA and touches only AC-70 (the typo), AC-72, one sentence of the iterative-walk paragraph and TC-048's ring step. I found no regression.

Notes on the round-3 checks:

- FND-010: the AC-72 ring is now compared "with its 12 text leaves and its one recursion leaf", and TC-048's ring step matches. The other AC-72 cases still hold with required leaves. The 20000-record ring carries its one text leaf and one recursion leaf: the last record's field reenters the first, which reaches text. Ten records `R0` to `R9` with `leaves` empty have to finish the count before the comparison with the empty list can fall short. That count (about 10^7 units by the SR-780 estimate, recursion leaves included or not) is well above the default 1 000 000, so the result is `incomplete` for `work` either way.
- The new charging sentence ("emitting the recursion leaf there, and consuming it in the pass, costs no unit beyond the one that reentry edge was charged") fits how the code charges. `LeafWalk::enter` charges one unit per entered edge before it inspects the target (operations.rs:1873-1877), and `first_leaf_fault` charges one per entering step (1967-1971). The reentry edge is therefore paid for as it is entered, and producing or matching the recursion leaf there needs no step of its own. The rule only fixes the reader's charging so that it is deterministic. AC-72 bisects against the work a read actually used, so the bisection is sound under any fixed charging, this one included.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-010 | fixed | 4a0bcc7c336d47f945b83daab5932b784707d092 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | medium | The ring case in AC-72 cannot admit now that the recursion leaf is required. It reads "a ring of 12 records, each holding one text field and an optional field naming the next, compared at the first with its 12 text leaves ... the exact work admits". The last record's `next` reenters the first, from which text is reachable, so the derived list is the 12 text leaves plus one recursion leaf (eleven `field:next, inner` pairs, then `field:next, inner, recursion:0`). A list of the 12 text leaves alone refuses `operation-law-missing`, so the "exact work admits" half fails for a correct reader. Fix: say "with its 12 text leaves and its recursion leaf" (TC-048's "bisect the work limit of a ring of 12 records" can stay as it is). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1164 |
