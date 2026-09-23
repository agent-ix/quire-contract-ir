---
id: FR-344
title: "Admit or refuse the ADR-002 semantic IR 2.0.0 members this reader does not yet carry"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/120
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/110
    type: references
  - target: ix://agent-ix/filament-core-data/issues/184
    type: references
  - target: ix://agent-ix/quire-specification/ADR-002
    type: references
  - target: ix://agent-ix/quire-specification/FR-340
    type: references
  - target: ix://agent-ix/quire-specification/FR-154
    type: references
  - target: ix://agent-ix/quire-specification/FR-208
    type: references
---

# FR-344: Admit or refuse the ADR-002 semantic IR 2.0.0 members this reader does not yet carry

## Description

QSpec ADR-002 (accepted) sets a lifted domain package's semantic IR contract
version to `2.0.0` and names seven members that member set adds beyond the
fields, multiplicity, units, relationships, operations, opaque clauses and
closed constraint keyword set the contract already carries: a supertype
list, an abstractness flag, a subsets edge, a redefines edge, an operation
frame, a population node and an embedded meaning vocabulary. FCD is the
producing side ADR-002 names to define each member
(`filament-core-data#184`, closed by the merged `filament-core-data#193`);
this repository is the producer-neutral consumer of the resulting
`quire.checked-package/v2` wire ([FR-038](./FR-038-consume-checked-package-v2.md)'s
Inputs).

Measured against the current reader
(`crates/quire-contract-model/src/checked_package/v2/mod.rs`,
`CheckedNodeTag::ALL`, 13 variants) and the QSpec artifacts that own the
wire (FR-322, FR-340), the seven members are not uniformly missing:

- **Operation frame** already has a published QSpec wire carrier —
  FR-340's `state`/`frame` node, cited by FR-322's own `semantic_graph`
  Properties — and this repository already admits and lowers it in full
  ([FR-038-AC-12](./FR-038-consume-checked-package-v2.md) through
  FR-038-AC-15, TC-053). ADR-002 names it among the seven because
  `filament-core-data#184`/#193 corrected its *document*-schema shape
  (`Operation.frame`, dotted access-path list → declaration `NodeRef`
  list, per FCD's own FR-141-AC-4); the wire this repository reads was not
  the correction's target and is unaffected. Closing issue #120 requires
  no new work on this member.
- **Embedded meaning vocabulary** is FR-208's closed Quire meaning id set,
  and the wire already carries it as `semantic_form` — FR-038 already
  validates a `model` node against the closed eighteen-meaning list and
  every other tag against its own closed form enum, refusing an
  unrecognized value as `invalid_semantic_graph`
  ([FR-038-AC-2](./FR-038-consume-checked-package-v2.md),
  FR-038-AC-17). `filament-core-data#184`/#193 states its
  `constructDeclaration.meaning` pipeline (FCD's own FR-142) "was already
  fully implemented" before that PR and introduces no new meaning id.
  Closing issue #120 requires no new work on this member either, absent a
  future FR-208 vocabulary extension this FR does not anticipate.
- The remaining five — **supertype list, abstractness flag, subsets edge,
  redefines edge** and **population node** — have no admission or
  lowering disposition on the `quire.checked-package/v2` wire today. Four
  of the five (every member but population) have no `(node_tag,
  semantic_form)` slot in `CheckedNodeKind` at all, so any
  wire content attempting to carry one refuses at the tag or form gate
  before reaching a body. `population` is a partial exception:
  `relation`/`population` is already a recognized `(tag, form)` pair
  (`CheckedNodeKind::Relation(RelationForm::Population)`), but no dedicated body validator
  exists for it — a `relation`/`population` node's `body` falls through to
  the generic closed `SemanticTerm` grammar (`validate_term`), which
  `filament-core-data#184`/#193's published `population` shape
  (`identity`, `displayName`, `kind`, `members`, `extent`, `origin` — no
  `term` member) cannot satisfy, so it also refuses today, one validation
  stage later than the other four.

None of the five is silently absorbed today: every admission path this
reader has is a closed grammar at every layer measured (`node_tag`,
`semantic_form`, every node-struct member via `#[serde(deny_unknown_fields)]`,
and body term shape via `exact_members`'s closed-set match), and an
unrecognized member at any layer refuses the whole package rather than
admitting it with that member dropped. The gap this FR specifies is
narrower than "silently admitted": a conforming `2.0.0` domain package
carrying any of these five members is refused *wholesale* today, with no
member-specific disposition, because this repository's wire has no
admission path for any of them — matching the underlying concern issue
#120 raises (a producer using the full `2.0.0` member set cannot get a
package through this reader) even though the measured failure mode is a
comprehensive refusal, not a silent one.

FCD#184/#193 also records, for three of the five, that its own extraction
pipeline does not yet produce a real instance from any spec artifact:
`subsets`, `redefines` and `populations` are "defined in the schema and
validated by the Rust reader, but... not lowered from quire-rs extraction"
(PR#193's own "Out of scope" notes), deferred to avoid a fixture
collision with FCD#173. `supertypes` and `abstract` carry no such
caveat. This does not change what this FR specifies — a wire-carrier
requirement is still the prerequisite for all five — but it means a
wire-carrier requirement for `subsets`, `redefines` or `population` alone
does not yet make a real 2.0.0 package exercising that member available
from FCD.

## Inputs

The same untrusted `quire.checked-package/v2` wire bytes and reader limits
as FR-038's Inputs. For the five members this FR specifies, no additional
input is defined: each member's FCD-side (domain-package/document) shape
is published by `filament-core-data#184`/#193 (cited per member below),
but no QSpec-owned `quire.checked-package/v2` wire-carrier requirement —
the FR-340 counterpart for these five — has been published as of this
writing (2026-09-21; verified against `agent-ix/quire-specification`'s
`spec/objects/interfaces/` directory, whose highest published interface FR
is FR-340 itself, and `FR-322`'s `semantic_graph` tag enumeration, which
still names exactly the 13 tags `CheckedNodeTag::ALL` admits). This FR
accordingly states no admission or lowering criterion for those five: a
criterion whose subject shape is unpublished cannot be written without
guessing a member shape ahead of the producing side, which issue #120 and
ADR-0054's producer-neutral boundary rule out. The blocked admission and
lowering work, and the exact upstream artifact each member waits on, is
recorded in Dependencies and Status below, where it is a scheduling fact
rather than a claim about reader behavior.

## Outputs

For a document carrying any of the five members in any of the three shapes
a producer can encode one in today — a new `node_tag`, a new
`semantic_form` of an admitted tag, or an extra member of an admitted node
— a typed refusal (`unsupported_node_tag`, `invalid_semantic_graph` or
`unknown_member`) carrying the structural path at which admission failed,
never a lowered record, and never a package admitted with that member's
content dropped. This FR adds no lowering-disposition kind: FR-038's
seven-member record vocabulary (`lowered`, `unsupported`, `requires_bound`,
`invalid_input`, `failed`, `invalid_body`, `body_incomplete`) stays closed,
and a future admission/lowering FR for these five draws from it rather than
extending it.

## Behavior

The reader's existing closed-grammar discipline (FR-038: strict parse,
`deny_unknown_fields` node shape, closed `node_tag`/`semantic_form`
vocabulary, closed `SemanticTerm` body grammar) already governs supertype
list, abstractness flag, subsets edge, redefines edge and population node
exactly as it governs every other unrecognized wire content: refuse the
whole package before any declaration is built, never interpret or drop
the offending member. This FR does not change that behavior; it names it
as the acceptance criteria for these five members, once per shape a
producer can encode one in: a `node_tag` outside `CheckedNodeTag::ALL` or
a `semantic_form` outside that tag's closed form enum refuses at the graph's tag
and form gates; an extra member on an otherwise-admitted node object
refuses earlier still, in the strict closed-schema decode of the whole
document, as `unknown_member` at `document`; and a body that satisfies no
branch of the closed `SemanticTerm` grammar refuses at the body. Admission
and lowering for these five is out of scope for this FR and stays blocked
on the one thing this repository does not own — the QSpec wire-carrier
requirement that fixes each member's `(node_tag, semantic_form, body)`
encoding, as FR-340 already did for operation frame.

## Acceptance Criteria

Operation frame and embedded meaning vocabulary are not restated here: they
are ADR-002 members with no new reader behavior to specify (Description,
above), and FR-038's own acceptance criteria already fully cover them
(FR-038-AC-12 through FR-038-AC-15 for operation frame, verified by
TC-053; FR-038-AC-2/AC-17 for embedded meaning vocabulary, verified by
TC-048) — restating them here would duplicate an existing criterion rather
than add one.

The three criteria below are the three shapes a producer can encode one of
the remaining five members in today, each with its own refusal code, locus
and validation layer. Every one is verifiable against the reader as it
stands. Admission and lowering for those five members is not stated as a
criterion here: its subject shape is unpublished, so any criterion for it
would assert the state of the specification rather than the behavior of
the reader, and would be unfalsifiable under test. That blocked work is
recorded in Dependencies and Status instead.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-344-AC-1 | A `quire.checked-package/v2` document whose semantic graph carries a node attempting to represent a supertype list, an abstractness flag, a subsets edge or a redefines edge, via any `node_tag` outside `CheckedNodeTag::ALL` or any `semantic_form` outside that tag's closed form enum, refuses `unsupported_node_tag` at `semantic_graph.nodes.node_tag` or `invalid_semantic_graph` at `semantic_graph.nodes.semantic_form`, before any declaration is built and with the document never admitted with the offending node dropped. | Test (TC-222) |
| FR-344-AC-2 | A `relation`/`population` node whose `body` carries content satisfying no branch of the closed `SemanticTerm` grammar refuses `invalid_semantic_graph`, never admitted as an ordinary `relation` term: at `semantic_graph.nodes.body.term` when the body carries no `term` member at all — the case of `filament-core-data#184`/#193's published population shape, which carries no `term` member — and at `semantic_graph.nodes.body` when it carries a `term` value that matches no arm's own closed member set. | Test (TC-222) |
| FR-344-AC-3 | A `quire.checked-package/v2` document carrying a subsets or redefines edge as an extra top-level member of an already-admitted node — FCD's published `field.subsets` and `field.redefines` shapes, which extend a `model`/`field_declaration` node rather than introducing a new `node_tag` or `semantic_form` — refuses `unknown_member` at `document`, in the strict closed-schema decode of the whole wire and therefore before any node tag, form or body of any node is validated, with the document never admitted with the extra member dropped. | Test (TC-222) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the
  `quire.checked-package/v2` reader and lowerer this FR extends. Operation
  frame (FR-038-AC-12 through FR-038-AC-15, TC-053) and embedded meaning
  vocabulary (FR-038-AC-2, FR-038-AC-17, TC-048) are fully discharged
  there; this FR adds no criterion for either.
- QSpec [FR-322](https://github.com/agent-ix/quire-specification/blob/main/spec/objects/interfaces/FR-322-checked-package-artifact.md)
  owns the `semantic_graph` tag enumeration this FR measures the ceiling
  of; QSpec [FR-340](https://github.com/agent-ix/quire-specification/blob/main/spec/objects/interfaces/FR-340-checked-package-frame-location.md)
  is the published precedent this FR's five planned criteria are shaped
  after — each becomes real once QSpec publishes its own counterpart for
  that member.
- QSpec [ADR-002](https://github.com/agent-ix/quire-specification/blob/main/spec/decisions/ADR-002-domain-package-ir-contract-version.md)
  names the seven members and the FCD ownership boundary this FR does not
  cross.
- `filament-core-data#184` (closed by `filament-core-data#193`) owns the
  domain-package/document-schema shape of each member; this FR cites it
  per member and specifies no shape beyond what that PR published.
- QSpec [FR-154](https://github.com/agent-ix/quire-specification/blob/main/spec/functional/type-model/FR-154-admit-domain-package-model.md)
  is the nearest-miss artifact: it is the one published QSpec requirement
  that names supertypes, subsets, redefines and population together with
  refusal codes, but it governs QSL's intake of a *domain package model* —
  a different artifact from the `quire.checked-package/v2` wire this
  repository reads. It is therefore not the wire-carrier requirement these
  five members wait on, and none of its refusal codes or loci transfers to
  this reader; it is cited here so a future reader does not have to
  re-derive that distinction.

### Blocked admission and lowering work

Admission, refusal-cause and lowering for the five members below are not
criteria of this FR. Each waits on a QSpec `quire.checked-package/v2`
wire-carrier requirement — the FR-340 counterpart for that member — fixing
its `(node_tag, semantic_form, body)` encoding; none is published as of
2026-09-21. The FCD-side shape each will have to carry is already
published, and is recorded here so the future requirement has a starting
citation rather than a rediscovery:

- Supertype list — FCD's own FR-141-AC-2, `typeDefinition.supertypes: identityList`.
- Abstractness flag — FCD's own FR-141-AC-1, `typeDefinition.abstract: boolean`.
- Subsets edge — FCD's own FR-141-AC-3, `field.subsets: identityList`.
- Redefines edge — FCD's own FR-141-AC-3, `field.redefines: semanticIdentity`.
- Population node — FCD's own FR-141-AC-4,
  `{identity, displayName, kind, members, extent, origin}`, ruled canonical
  on `filament-core-data#196`.

For subsets, redefines and population a second condition also applies: FCD's
own extraction pipeline does not yet produce a real instance of any of the
three from a spec artifact (`filament-core-data#193`'s own "Out of scope"
note), so even once a wire carrier publishes, the first fixture for those
three can only be hand-constructed to the published shape rather than
traceable to a real spec.
- [issue #110](https://github.com/agent-ix/quire-contract-ir/issues/110)
  names the same ADR-002 surface touching FR-035's `ContractPackage`
  output. This FR does not author FR/TC content for that gap; the two are
  independent ADR-002 consequences with independent closing conditions.

## Status

Specified, not yet implemented. Operation frame and embedded meaning
vocabulary need no new implementation — two of the seven ADR-002 members
are already discharged by FR-038 and are not restated as criteria of this
FR. All three of this FR's criteria (FR-344-AC-1, FR-344-AC-2 and
FR-344-AC-3) hold today as an emergent property of FR-038's existing closed
grammars; TC-222 is planned to pin them explicitly as a named regression
rather than leave them implicit, and requires no reader or lowerer change.

Admission and lowering for the five members remains blocked, per
Dependencies above, on a QSpec-published `quire.checked-package/v2`
wire-carrier requirement per member, none of which exists as of
2026-09-21. That work is not a criterion of this FR and carries no test
case here; when the first wire carrier publishes, it is specified by a new
FR of its own, with its own testable criteria and its own test case.
