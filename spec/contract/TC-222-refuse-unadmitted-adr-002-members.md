---
id: TC-222
title: "Refuse a document attempting to carry an unadmitted ADR-002 2.0.0 member"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-344
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/issues/120
    type: references
---

# TC-222: Refuse a document attempting to carry an unadmitted ADR-002 2.0.0 member

## Description

Verify FR-344-AC-1, FR-344-AC-2 and FR-344-AC-3: a `quire.checked-package/v2` document
attempting to
carry a supertype list, an abstractness flag, a subsets edge, a redefines
edge or a population node — the five ADR-002 members this reader has no
admission path for — is refused wholesale, with a typed code, and never
admitted with the offending node dropped or reinterpreted.

## Test Procedure

Starting from one admitted V2 fixture (`checked_package_v2_reader`'s
baseline vector), construct one mutated document per case:

1. A node whose `node_tag` is a synthetic value outside `CheckedNodeTag::ALL`
   (e.g. `"supertype"`), chosen to mimic a producer attempting to carry a
   supertype list or an abstractness flag as its own top-level tag.
2. A node whose `node_tag` is `"model"` and whose `semantic_form` is a
   synthetic value outside `CheckedNodeTag::Model::forms()` (e.g.
   `"abstract_flag"`), chosen to mimic a producer attempting to carry the
   abstractness flag as a new form of an existing tag.
3. A node whose `node_tag` is `"model"`, `semantic_form` is
   `"field_declaration"` (an existing admitted form), and whose JSON object
   carries an extra top-level member (`subsets` or `redefines`) alongside
   the closed node shape, chosen to mimic a producer attempting to carry a
   subsets or redefines edge as a new struct member rather than a new tag
   or form.
4. A `relation`/`population` node (an already-admitted `(tag, form)` pair)
   whose `body` is `filament-core-data#184`/#193's published population
   shape verbatim: `{"identity": ..., "displayName": ..., "kind": {...},
   "members": [...], "extent": "closed", "origin": {...}}` — no `term`
   member.
5. The same case 4 body with a single `"term": "aggregate"` member added
   alongside its existing members, so the body object is
   `{"term": "aggregate", "identity": ..., "displayName": ..., "kind":
   {...}, "members": [...], "extent": "closed", "origin": {...}}` — the
   population's own `identity`, `displayName`, `kind`, `extent` and
   `origin` remain siblings of `term` and `members` at the body root, and
   are not nested inside the `members` array. This confirms a producer
   cannot get population content admitted as an ordinary `relation` node by
   naming a closed term arm over it while leaving its real members in
   place.

## Expected Results

Every case refuses before any declaration is built:

- Cases 1 refuses `unsupported_node_tag` at `semantic_graph.nodes.node_tag`.
- Case 2 refuses `invalid_semantic_graph` at `semantic_graph.nodes.semantic_form`.
- Case 3 refuses `unknown_member` at `document`. The whole wire is decoded
  once through `decode_closed::<CheckedPackageWireV2>`, and
  `CheckedSemanticNodeV2` carries `#[serde(deny_unknown_fields)]`, so an
  extra top-level node member makes that single decode fail with serde's
  `unknown field` error, which `decode_closed` classifies as
  `unknown_member` at the fixed `"document"` path. The refusal therefore
  precedes every graph-level check — no node tag, form or body of any node
  is reached — and its locus names the document rather than the offending
  node, which is the reader's behavior at this layer and not an omission
  this test case works around.
- Case 4 refuses `invalid_semantic_graph` at `semantic_graph.nodes.body.term`,
  a path distinct from case 5's: the object carries no `term` member, so it
  fails `validate_term`'s first extraction of `object.get("term")` — the
  reader's own `"semantic_graph.nodes.body.term"` path constant — before any
  term arm is tried. Case 5 refuses `invalid_semantic_graph` at
  `semantic_graph.nodes.body` instead: the body object does carry a `term`
  string, so extraction succeeds and the `"aggregate"` arm is selected, but
  that arm's guard is `exact_members(object, &["term", "members"])`, an
  exact length-and-membership match. The population's `identity`,
  `displayName`, `kind`, `extent` and `origin` remain siblings of `term` and
  `members`, so the object has seven members where the arm admits two, the
  guard fails, and no other arm's guard matches a `term` of `"aggregate"` —
  the match falls to its final arm, whose refusal path is
  `semantic_graph.nodes.body`. Nesting those five members inside the
  `members` array instead would satisfy `exact_members` and reach the
  `aggregate` arm's own per-element walk, which refuses at
  `semantic_graph.nodes.body.term` — case 4's locus, not this one's — so
  this case's fixture is built with them as siblings.

No case admits the document with the mutated node dropped, downgraded to
an existing form, or partially interpreted; each refusal names the exact
node and structural path.

## Status

Planned. FR-344-AC-1, FR-344-AC-2 and FR-344-AC-3 already hold as an emergent
property of FR-038's
existing closed grammars (`CheckedNodeTag::from_wire`, `tag.forms()`,
`#[serde(deny_unknown_fields)]`, `exact_members`); this test case gives
that property its own named regression rather than leave it implicit and
untested. It requires no reader or lowerer change — only new fixtures and
assertions alongside FR-038's existing `checked_package_v2_reader` suite.
