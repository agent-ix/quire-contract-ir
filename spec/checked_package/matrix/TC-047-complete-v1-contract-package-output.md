---
id: TC-047
title: "Complete-V1 lowering emits one canonical ContractPackage"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-330
    type: references
---
# TC-047: Complete-V1 lowering emits one canonical ContractPackage

## Description

Verify FR-035-AC-5 through FR-035-AC-8 and the aggregate FR-330 package
separately from TC-044's per-item semantic and refusal tests. Focused FR-330
emitter and strict-reader controls run in
`tests/it/complete_v1_contract_package.rs`.

## Test Procedure

Lower a mixed request with represented and refused items. Parse the emitted
package as `quire.contract-package/v1`. Assert the six FR-330 properties,
exactly one `semantic_graph.nodes` collection, one occurrence per typed key,
ascending typed-key order, total `source_map` correspondence, and one
`feature_accounting` record per requested item. Reorder equivalent request
and graph-construction inputs, canonicalize the package and compare identity.
Independently compute the digest from the RFC 8785 canonical projection of
exactly `contract_version`, `package`, `semantic_graph` and
`feature_accounting`, excluding `source_map` and `identity`, and compare it
with the emitted identity. Change one represented node, the selected profile,
one accounting result and a valid source-map location separately; compare the
digest after each change. Corrupt or omit source-map correspondence and check
admission. Remove one required property, add an independent
type/expression/clause graph, duplicate or disorder a key, or substitute an
out-of-domain contract version and test consumer refusal before artifact generation.

From a genuinely admitted mixed graph, query the typed view for a lowered root
whose local closure uses `semantic_type`, declared `dependencies`, and body
references, including a shared node and an explicit recursion group. Independently
enumerate those validated edges and compare the reached keys, node values and
source-map entries with the view in typed-key order. Include an adversarial body
reference whose target is omitted from a declared dependency list, so a walk
limited to that list fails the comparison. For lowered roots with nonempty bound
and claim subsets, compare both exposed typed-key subsets with independently
derived IR lowering results, including exact membership and order. Repeat for
roots with empty bound and claim subsets and require both empty subsets. Include a
selected external reference and an identically keyed local node; confirm the
view retains the selected identity and target key, and admission refuses a
missing, stale or wrong selected package. Query refused, unrequested reached
and absent keys, then compare all six wire properties and identity before and
after the read-only queries.

## Expected Results

One FR-330 package carries the exact represented node set and its reached
local closure in key order; a refused request contributes a node only when
reached by represented meaning. Its source map and accounting are complete.
Equivalent construction order has the same canonical semantic identity and
the digest equals the independently computed four-member projection. Changing
represented meaning, profile or an accounting result changes the digest;
changing only a valid source-map location preserves it. Corrupt or incomplete
source correspondence refuses admission despite its exclusion from the digest.
Missing properties, duplicate or disordered keys, extra graph partitions and
wrong contract versions refuse before consumption.
The lowered-root view contains exactly the key-ordered local reached closure,
with exact nodes and sources, and exposes selected external references at their
package boundary. Every lowered-root view exposes both bound and claim subsets,
empty when appropriate, with the exact keys and order of IR's own lowering
classification. Refused requests return their typed disposition;
unrequested reached or absent roots return typed absence. No such query changes
the six-field wire or identity.

## Status

Implemented with focused tests in `tests/it/complete_v1_contract_package.rs`
for the six-field FR-330 emitter, strict reader, canonical semantic identity,
closed accounting and source-map correspondence. Aggregate gates remain pending.
The admitted reached-closure traversal controls for FR-035-AC-6 through
FR-035-AC-8 are planned and unrun.
