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

Verify FR-035-AC-2, FR-035-AC-5 through FR-035-AC-8, and FR-035-AC-10 through FR-035-AC-17,
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
amount at the failed charge; assert no path for bytes,
`/semantic_graph/nodes` for nodes,
`/semantic_graph/nodes/<i>/dependencies` for edges, and the exact occurrence
or region value under `/source_map` whose charge failed for occurrences.
For work, assert `/semantic_graph/nodes/<i>/body` when value
validation stops or the failed-charge pointer reported by reference validation,
which may name another node member. Assert neither outcome admits a package.
For FR-035-AC-15, use `mixed_fixture()` and `request()` for the work
boundary: value validation
charges 7 visits, reference validation charges 20, and the combined count is
27. A work limit of 27 admits. Limits 7, 20, and 26 return `Incomplete` with
`consumed` 8 at `/semantic_graph/nodes/0/node_id`, 21 at
`/semantic_graph/nodes/2/node_id`, and 27 at
`/semantic_graph/nodes/2/body/members/1/value/type`, respectively. Assert
`limit_kind = Work` and the selected limit in each case. A lower limit that
stops in value validation reports its body pointer.
For FR-035-AC-16, use the same fixture and charge one unit for each
occurrence before one unit for
each of its regions, in array order within `nodes`, `requested_items`, then
`refusals`. The 9 occurrences and 9 regions give 18 charges. An occurrence
limit of 18 admits. Limit 0 returns `Incomplete` with `consumed` 1 at
`/source_map/nodes/0/occurrences/0`; limit 17 returns `Incomplete` with
`consumed` 18 at `/source_map/refusals/1/occurrences/0/regions/0`.
Also stop at the first charged occurrence under `requested_items`: select the
number of charges before it as the limit, and assert `consumed` is that limit
plus one and `path` names that occurrence. Assert `limit_kind = Occurrences`
and the selected limit at each stop; `/source_map` alone is never the stop
pointer when a value was charged.

For FR-035-AC-12, remove each of the six required top-level members
(`contract_version`, `package`, `semantic_graph`, `source_map`,
`feature_accounting`, and `identity`) separately from otherwise valid canonical
packages and assert `Refused(invalid_package)` at root path `""` with cause
`missing_member` for every omission. Include the separate `contract_version`
reader branch. For FR-035-AC-13, prepend a space, tab, LF, or CR separately to
otherwise valid canonical bytes and assert `Refused(noncanonical_wire)` with no
path or cause.
For FR-035-AC-14, change only the first `feature_accounting.node_id` in an
otherwise valid canonical package to a different valid requested node,
recompute the package identity, and assert
`Refused(invalid_package)` at `/feature_accounting` with cause `invalid_value`.
Each case admits no `ContractPackageV1`.

For FR-035-AC-17, construct an otherwise valid package with one nested
`invalid_body.refusal`. For each of `unknown_contract_version` and
`unknown_required_capability`, admit the record without a cause, then add
`unsupported-wire` or `unsupported-feature` respectively. Require
`Refused(invalid_package)` with cause `invalid-value` at that record's
`/refusal/cause` pointer; no package is admitted.

For FR-035-AC-2, change only the package's own identity digest and assert
`Refused(invalid_package)` at `/identity/digest` with cause `invalid_value`.
Select a dependency under `package.source.lock.dependency_selections`, then
change its selected package digest, recompute the contract package identity,
and supply the original dependency. Assert `Refused(stale_dependency)` at
`/package/source/lock/dependency_selections/0` with cause `content_mismatch`.
These two cases admit no package. The tests
`reader_refuses_stale_own_identity_at_digest` and
`reader_refuses_stale_selected_dependency_at_selection` carry FR-035-AC-2;
the separate malformed-shape and dependency-selection tests retain FR-035-AC-5
for their unrelated checks.

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
admits no package. The mixed fixture admits at work limit 27 and occurrence
limit 18 under FR-035-AC-15 and FR-035-AC-16; its work stops at limits 7, 20
and 26 and occurrence stops at limits
0 and 17 have the exact counts and pointers specified above. The
`requested_items` occurrence stop names its first charged occurrence.
The malformed, noncanonical and bad-accounting fixtures return the exact
FR-035-AC-12 through FR-035-AC-14 refusal fields. Stale own identity and stale
selected dependency return the exact FR-035-AC-2 refusal fields.
The two causeless nested reader codes admit with no cause and refuse a present
cause as `invalid_package`/`invalid-value` at their own `/refusal/cause` pointer.
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
FR-035-AC-10 is backed by the tagged TC-047 test
`reader_refuses_wrong_string_version_before_shape_identity_and_nodes`.
The tagged `reader_incomplete_reports_each_exhausted_meter_and_pointer` test
backs amended FR-035-AC-11 with exact failed-charge counts and pointers.
Tagged boundary tests back FR-035-AC-15 and FR-035-AC-16, including cumulative
work across both validation passes and occurrence and region charges in order.
FR-035-AC-12 through FR-035-AC-14 are backed by tagged TC-047 reader tests
covering all six missing top-level members, four leading whitespace bytes, and
a later represented requested root in the first accounting record. The stale
own-identity and selected-dependency refusals carry FR-035-AC-2 tags and
assert the specified cause and path. TC-044's FR-330 status remains
PLANNED/UNRUN.
The tagged nested-refusal test backs FR-035-AC-17 for both reader codes.
