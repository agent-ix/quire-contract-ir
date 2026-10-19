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

Verify FR-035-AC-2, FR-035-AC-5 through FR-035-AC-8, and FR-035-AC-10 through FR-035-AC-14,
and the aggregate FR-330 package separately from TC-044's per-item semantic
and refusal tests. Focused FR-330
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
For the wire reader itself, change only `contract_version` to another string in
an otherwise valid canonical package. Assert `Refused` with
`unknown_contract_version`, path `/contract_version` and the exact supplied
version. For a valid package, select each byte, node, work, edge and
occurrence limit separately at a value that exhausts that reader charge. Assert
`Incomplete` with the exhausted limit kind, selected ceiling and `consumed`
amount of the meter that stopped; assert no path for bytes,
`/semantic_graph/nodes` for nodes,
`/semantic_graph/nodes/<i>/dependencies` for edges, and `/source_map` for
occurrences. For work, assert `/semantic_graph/nodes/<i>/body` when value
validation stops or the failed-charge pointer reported by reference validation,
which may name another node member. Assert neither outcome admits a package.
The work meter's scope is open under IR-889.

For FR-035-AC-12, remove `source_map` from an otherwise valid canonical
package and assert `Refused(invalid_package)` at root path `""` with cause
`missing_member`. For FR-035-AC-13, prepend one space to otherwise valid
canonical bytes and assert `Refused(noncanonical_wire)` with no path or cause.
For FR-035-AC-14, change the first `feature_accounting.node_id` to a different
valid requested node, recompute the package identity, and assert
`Refused(invalid_package)` at `/feature_accounting` with cause `invalid_value`.
Each case admits no `ContractPackageV1`.

For FR-035-AC-2, change only the package's own identity digest and assert
`Refused(invalid_package)` at `/identity/digest` with cause `invalid_value`.
Select a dependency under `package.source.lock.dependency_selections`, then
change its selected package digest, recompute the contract package identity,
and supply the original dependency. Assert `Refused(stale_dependency)` at
`/package/source/lock/dependency_selections/0` with cause `content_mismatch`.
These two cases admit no package. The later tests-first PR shall retag
`reader_refuses_malformed_outer_shape_and_stale_own_identity` and
`selected_dependencies_are_bound_by_library_identity_before_graph_checks`
from FR-035-AC-5 to FR-035-AC-2 for these assertions, splitting their mixed
assertions where needed so AC-5 still owns its unrelated checks.

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
The wrong-version reader refusal retains its exact typed code, path and supplied
version. Every exhausted read limit returns the typed incomplete fields and
admits no package.
The malformed, noncanonical and bad-accounting fixtures return the exact
FR-035-AC-12 through FR-035-AC-14 refusal fields. Stale own identity and stale
selected dependency return the exact FR-035-AC-2 refusal fields.
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
Tagged TC-047 tests cover FR-035-AC-6 through FR-035-AC-8.
FR-035-AC-10 and FR-035-AC-11 are backed by the tagged TC-047 reader tests
`reader_refuses_wrong_string_version_before_shape_identity_and_nodes` and
`reader_incomplete_reports_each_exhausted_meter_and_pointer`. AC-11 work-limit
scope remains open under IR-889.
FR-035-AC-12 through FR-035-AC-14 and the FR-035-AC-2 TC-047 retags are
PLANNED/UNRUN until the later tests-first PR adds the trace evidence. TC-044's
FR-330 status remains PLANNED/UNRUN.
