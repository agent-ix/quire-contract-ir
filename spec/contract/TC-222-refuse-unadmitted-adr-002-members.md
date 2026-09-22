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

Verify FR-344-AC-1 and FR-344-AC-2: a `quire.checked-package/v2` document
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
5. The same case 4 body, additionally wrapped to look like the closed
   `aggregate` term shape (`{"term": "aggregate", "members": [...]}`) with
   the population's real members nested inside, to confirm a producer
   cannot get population content admitted as an ordinary `relation` node
   by shaping it to fit the generic grammar.

## Expected Results

Every case refuses before any declaration is built:

- Cases 1 refuses `unsupported_node_tag` at `semantic_graph.nodes.node_tag`.
- Case 2 refuses `invalid_semantic_graph` at `semantic_graph.nodes.semantic_form`.
- Case 3 refuses `unknown_member` (or the strict-parse duplicate/unknown-member
  refusal FR-038-AC-1/AC-2 already exercise) before the node's own shape is
  further validated.
- Case 4 refuses `invalid_semantic_graph` at `semantic_graph.nodes.body.term`,
  a path distinct from case 5's: the object carries no `term` member, so it
  fails `validate_term`'s first extraction of `object.get("term")` — the
  reader's own `"semantic_graph.nodes.body.term"` path constant — before any
  term arm is tried. Case 5 refuses `invalid_semantic_graph` at
  `semantic_graph.nodes.body` instead, because the population identity,
  `displayName`, `kind`, `extent` and `origin` members have no home in the
  `aggregate` term's own closed member set (`exact_members(object, &["term",
  "members"])`), so the wrapped shape fails the same closed-set match an
  unrelated extra member would.

No case admits the document with the mutated node dropped, downgraded to
an existing form, or partially interpreted; each refusal names the exact
node and structural path.

## Status

Planned. FR-344-AC-1 and FR-344-AC-2 already hold as an emergent property of FR-038's
existing closed grammars (`CheckedNodeTag::from_wire`, `tag.forms()`,
`#[serde(deny_unknown_fields)]`, `exact_members`); this test case gives
that property its own named regression rather than leave it implicit and
untested. It requires no reader or lowerer change — only new fixtures and
assertions alongside FR-038's existing `checked_package_v2_reader` suite.
