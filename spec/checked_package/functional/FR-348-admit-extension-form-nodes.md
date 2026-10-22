---
id: FR-348
title: "Admit checked-package extension form nodes"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-467
    type: references
  - target: ix://agent-ix/quire-specification/FR-469
    type: references
---
# FR-348: Admit checked-package extension form nodes

## Description

When the `quire.checked-package/v2` reader of [FR-038](./FR-038-consume-checked-package-v2.md)
reads an `extension`/`extension_form` node, the reader SHALL admit the node only when its
published QSpec FR-469 shape, structural key and five reader joins hold. The reader SHALL
return FR-469's first applicable refusal, in FR-322's reader order, without partially
admitting the package. The extension's node kind is body data; adding a selected
extension SHALL NOT add a node tag or semantic form to the closed reader vocabulary.

QSpec FR-469 and its published V2 schema and `ExtensionFormNode` preimage define the
wire contract. This requirement states the Contract IR reader's intake, evidence and
refusal behavior. The reader does not parse source extension syntax or infer a
missing definition from a node kind, feature name or source text.

## Inputs

- Untrusted V2 package bytes and the existing [FR-038](./FR-038-consume-checked-package-v2.md)
  read limits, domain-package documents and dependency-package evidence.
- Validated extension definitions supplied by the caller under their exact
  `{authority, identity}` `DefinitionRef`, beside the package bytes. Each supplied
  definition has FR-467's `grammar_schema` and `typed_node_schema`; the reader
  consults its typed-node schema at the declaration join. The supplied definitions
  are reader evidence, never members of the package, its identity preimage or its
  `required_features`.

## Outputs

The complete admitted `CheckedPackageV2` with typed extension nodes and their
source/identity facts, or one typed `CheckedPackageRefusal` with code, cause when
catalogued, RFC 6901 `path`, and `locus` when FR-469 requires it. No partial
package or guessed extension semantics are returned.

## Behavior

### Wire and identity

The reader SHALL decode `extension` as a closed `CheckedNodeTag` and
`extension_form` as that tag's single closed semantic form, once at intake.
An unknown tag still refuses `unsupported_node_tag`; an unsupported form of
`extension` still refuses by [FR-038](./FR-038-consume-checked-package-v2.md)'s
form rule. Every exhaustive family/form match SHALL decide the added variant.

The form SHALL carry a declaration, source owner, one `declaration` occurrence
of ordinal zero and the one whole-form source region. Its body SHALL carry
`definition`, `node_kind` and ordered `clauses`; each clause SHALL be a named
`binding` whose value is a `reference` to a separately keyed subnode. Its
`semantic_type` SHALL equal its own node id, and its `dependencies` SHALL be
exactly the unique clause targets in ascending node-id digest order. The node
SHALL carry neither recursion group nor nominal preimage. The reader SHALL
recompute the key from QSpec's closed `quire.structural-node/v1`
`ExtensionFormNode` preimage, including the definition, node kind, declaration,
owner and clause target keys, and SHALL keep source regions outside that key.
The node and its declaration use FR-322's types namespace, so an extension
form and a record of the same qualified name refuse the existing
`ambiguous_declaration`/`ambiguous-name` rule.

A body outside the published `extension_form` production, a wrong occurrence
shape or an inline clause value SHALL refuse `malformed_wire` at strict wire
validation. A dependency list unlike the clause targets SHALL refuse
`invalid_semantic_graph` at `/semantic_graph/nodes/{i}/dependencies` in the
graph-shape stage. A retained stale key SHALL refuse `invalid_package`/
`stale-node-key` at `/semantic_graph/nodes/{i}/node_id` at the derived-shape
key stage. The reader SHALL NOT repair any of those members.

### Held definitions and declaration joins

The caller SHALL supply each held extension definition through reader evidence
before `read`. The evidence SHALL retain one validated definition for each
exact `DefinitionRef`; repeating an equal definition is idempotent, while
supplying different definitions under one exact reference SHALL return a typed
insertion conflict without replacing the retained definition. A definition
absent from the reader evidence is not inferred from
`lock.definition_selections`, `lock.required_features` or the graph.

After the protocol-reference declaration joins and before frame checks, the
reader SHALL visit extension nodes in ascending node-id digest order. For each
node it SHALL check the first failing join in this order, using FR-469's code,
cause and pointer:

1. Its body `definition` is an exact `lock.definition_selections` entry;
   otherwise `missing_declaration`/`missing-selection` at
   `/semantic_graph/nodes/{i}/body/definition`.
2. The reader evidence holds that definition; otherwise
   `unknown_required_capability` with no cause, the node id as `locus` and
   the same definition pointer. This check SHALL NOT become
   `unknown_required_feature`.
3. The definition's typed-node schema declares the body's `node_kind`;
   otherwise `missing_declaration`/`missing-name` at
   `/semantic_graph/nodes/{i}/body/node_kind`.
4. The clause names equal that schema entry's names in declaration order.
   The first extra, wrong-position or repeated name refuses
   `invalid_package`/`invalid-value` at its clause pointer; a missing
   trailing clause refuses `invalid_package`/`missing-member` at
   `/semantic_graph/nodes/{i}/body/clauses`, naming the first absent name.
5. In clause order, each referenced subnode meets the schema entry's core
   `type-ref` or extension node-kind type. A missing target refuses
   `missing_declaration`/`missing-name`; a present target of the wrong type
   refuses `ill_typed`/`type-mismatch` naming the clause, expected type or
   node kind and actual semantic type. Both use the clause's `/value` pointer.

The reader SHALL use the first refusal of the first failing node, preserving
FR-322's earlier strict-wire, graph-shape and derived-key stages. The package's
lock and source map SHALL remain unchanged by a refusal. The extension
identity enters `package_id` only through the existing identity projection;
held-definition evidence itself SHALL NOT enter that projection.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-348-AC-1 | With the FR-467 `retention` definition selected and held, the QSpec `retention Audit` form reads as one `extension`/`extension_form` node with the `Audit` declaration, source owner, one declaration occurrence and whole-form source region, `subject` then `days` reference clauses to Text and bounded-Integer subnodes, and exactly those dependencies in digest order; the full reader admits it. A second extension node kind uses the same closed tag and form without changing the schema or reader enum. | Test (TC-459) |
| FR-348-AC-2 | The reader recomputes the `ExtensionFormNode` key and package identity from the definition, node kind, declared name, owner and clause target keys; changing each one with recomputed identities changes both ids, while changing only the source region changes neither. A retained old key refuses `invalid_package`/`stale-node-key` at that node. | Test (TC-459) |
| FR-348-AC-3 | With no held definition, a selected extension node refuses `unknown_required_capability` with no cause, the node id as `locus` and its body definition pointer; without the matching `definition_selections` row it instead refuses `missing_declaration`/`missing-selection` there. No extension identity appears in `required_features`, and neither case is `unknown_required_feature`. Equal repeated evidence is idempotent and conflicting evidence is rejected without replacement. | Test (TC-459) |
| FR-348-AC-4 | An undeclared node kind refuses `missing_declaration`/`missing-name` at `/body/node_kind`; reordered, unknown and repeated clause names each refuse `invalid_package`/`invalid-value` at the first offending clause, while a missing trailing clause refuses `invalid_package`/`missing-member` at `/body/clauses` naming it. | Test (TC-459) |
| FR-348-AC-5 | A `days` reference to a Text subnode refuses `ill_typed`/`type-mismatch` naming `days`, `Int[1, 3650]` and Text; an absent target refuses `missing_declaration`/`missing-name`; and an extension-kind clause referencing another kind refuses `ill_typed`/`type-mismatch`. Each reports its clause value pointer. | Test (TC-459) |
| FR-348-AC-6 | An inline clause value, wrong form, second or wrong-role occurrence each refuse `malformed_wire`; missing clause dependency refuses `invalid_semantic_graph` at `/dependencies`; a record of the form's name refuses `ambiguous_declaration`/`ambiguous-name` at the later node by node-id order. | Test (TC-459) |
| FR-348-AC-7 | Two extension nodes with failures report the first node by ascending node-id digest and its first failing join; a package with an earlier strict-wire, graph-shape, structural-key or protocol-reference failure reports that stage before the extension join, and a frame failure reports after it. | Test (TC-459) |
| FR-348-AC-8 | The reader loads QSpec's selected-checkout `positive-all-families.json` and `positive-extension-nodes.json` directly, without copying their bytes or schemas into this repository. The test constructs the X01..X17 controls described in QSpec TC-951 against those fixtures, checks X02's identity-only branches through the published preimage rule, and checks each specified reader admission or refusal by its result, code, cause, path and locus as applicable; both positive fixtures admit with held definitions. | Test (TC-459) |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 read order, typed
  refusal and evidence boundary; this requirement adds only the extension
  family, definition evidence and declaration joins.
- QSpec FR-469 owns the published node, key and vectors; QSpec FR-467 owns
  extension-definition validation. STD-236 may restore those published
  artifacts after this reader admits them.
