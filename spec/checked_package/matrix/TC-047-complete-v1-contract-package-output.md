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

Verify FR-035-AC-5 and the aggregate FR-330 package separately from TC-044's
per-item semantic and refusal tests. The FR-330 emitter cases are planned and
have not run.

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
wrong contract versions refuse before consumption. No output from the current
IR-only package shape counts as this result.

## Status

Planned / unrun for the FR-330 package emitter. The existing
`tests/it/complete_v1_contract_package.rs` asserts the earlier IR package
shape and cannot discharge this test until IR-743 updates it.
