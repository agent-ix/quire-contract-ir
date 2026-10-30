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

Verify FR-035-AC-2, FR-035-AC-6 through FR-035-AC-8, FR-035-AC-10 through FR-035-AC-18,
and planned FR-035-AC-23 through FR-035-AC-30,
and the aggregate FR-330 package separately from TC-044's per-item semantic
and refusal tests. Focused FR-330
emitter and strict-reader controls run in
`tests/it/complete_v1_contract_package.rs`.

## Test Procedure

Lower a mixed request with represented and refused items. Read the emitted package as `quire.contract-package/v1`. Assert exactly five FR-330 members, one ordered semantic graph node collection, total source correspondence and one accounting record per request. Equivalent construction orders produce equal complete canonical bytes. Change a represented node, selected profile, accounting result and valid source-map region separately; compare the corresponding typed member and complete bytes, then require strict admission or the applicable independent refusal. An extra `identity` member, missing member, extra graph partition, duplicate/disordered key and wrong version refuse before artifact generation. No digest or hash is computed.
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

For FR-035-AC-12, remove each of the five required top-level members
(`contract_version`, `package`, `semantic_graph`, `source_map`,
and `feature_accounting`) separately from otherwise valid canonical
packages and assert `Refused(invalid_package)` at root path `""` with cause
`missing_member` for every omission. Include the separate `contract_version`
reader branch. For FR-035-AC-13, prepend a space, tab, LF, or CR separately to
otherwise valid canonical bytes and assert `Refused(noncanonical_wire)` with no
path or cause.
For FR-035-AC-14, change only the first `feature_accounting.node_id` in an
otherwise valid canonical package to a different valid requested node,
and assert
`Refused(invalid_package)` at `/feature_accounting` with cause `invalid_value`.
Each case admits no `ContractPackageV1`.

For FR-035-AC-17, construct an otherwise valid package with one nested
`invalid_body.refusal`. For each of `unknown_contract_version` and
`unknown_required_capability`, admit the record without a cause, then add
`unsupported-wire` or `unsupported-feature` respectively. Require
`Refused(invalid_package)` with cause `invalid-value` at that record's
`/refusal/cause` pointer; no package is admitted.

For FR-035-AC-18, supply two admitted direct dependencies whose own locks
select one name with unequal complete package values. Require
`Refused(invalid_package)` with cause `conflicting-definition` before graph
references, and assert both importer-rooted selection pointers exactly.

For FR-035-AC-2, change the source package name, selected dependency name and an exact represented body independently. Require the normal selected-source, missing-import, graph or handoff refusal when the change breaks a join. A correctly renamed and otherwise valid package admits. There is no own-identity digest or stale-dependency content-mismatch refusal.

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
and absent keys, then compare all five wire properties and canonical bytes before and
after the read-only queries.

For FR-035-AC-27 through FR-035-AC-30, lower one admitted V2 source with
separate requested `temporal/fairness`, `temporal/formula` and
`temporal/temporal_clause` roots, a healthy sibling, and a temporal-clause root
whose admitted fairness argument references a distinct fairness node with the
least typed key among temporal nodes in the clause's reached closure, making
that node the first unsupported reachable node under
a profile that does not support the temporal family. Include a refused root
absent from the represented V1 graph. The fairness node's application has
zero operands and only its admitted declaration dependency; it does not
reference the temporal clause.
At each `feature_accounting` index, compare the producer's requested-root
`CheckedNodeKind` with the independently located V2 source root and compare
the unchanged `unsupported_node_id` and `node_tag` with the first unsupported
reachable node. Interpose a healthy request and verify the accounting indices
and sibling results. Strict-read the producer's canonical V1 bytes both with
the exact reader-admitted V2 source through `read_with_source` and without a
source through plain `read`. The authenticated result must match the producer's
requested-root kinds; the plain result must report typed `NotAdmitted` for each
`unsupported` index, including a requested root also represented through a
different lowered root. Query a healthy record and an out-of-range index and
require typed absence, distinct from `NotAdmitted`. Supply a mismatched source
identity and lock separately, and a source missing the requested root through
a controlled test-only mutation; require a typed handoff error with no
partially authenticated package. Also assert existing V1 reader refusal and
limit precedence over source handoff. Compare all five wire members and canonical
bytes before and after the typed queries.

## Expected Results

One FR-330 package carries the exact represented node set and its reached local closure in structural key order. Its source map and accounting are complete. Equivalent construction order yields equal complete canonical bytes; changing a represented body, profile, accounting or valid source region changes the corresponding typed member. Corrupt correspondence, missing required properties, duplicate or disordered keys, extra graph partitions and wrong versions refuse before consumption.
The wrong-version reader refusal retains its exact typed code, path and supplied
version. Every exhausted read limit returns the typed incomplete fields and
admits no package. The mixed fixture admits at work limit 27 and occurrence
limit 18 under FR-035-AC-15 and FR-035-AC-16; its work stops at limits 7, 20
and 26 and occurrence stops at limits
0 and 17 have the exact counts and pointers specified above. The
`requested_items` occurrence stop names its first charged occurrence.
The malformed, noncanonical and bad-accounting fixtures return the exact
FR-035-AC-12 through FR-035-AC-14 refusal fields. Source and dependency selection mutations return FR-035-AC-2’s applicable independent refusal fields.
The two causeless nested reader codes admit with no cause and refuse a present
cause as `invalid_package`/`invalid-value` at their own `/refusal/cause` pointer.
The lowered-root view contains exactly the key-ordered local reached closure,
with exact nodes and sources, and exposes selected external references at their
package boundary. Every lowered-root view exposes both bound and claim subsets,
empty when appropriate, with the exact keys and order of IR's own lowering
classification. Refused requests return their typed disposition;
unrequested reached or absent roots return typed absence. No such query changes
the five-field wire or canonical bytes.
The unsupported requested-root-kind view distinguishes fairness, formula and
temporal-clause roots despite their shared `temporal` tag, independently of
which reachable node made lowering unsupported. Producer and authenticated
reader return identical typed kinds; plain reader returns `NotAdmitted` for
unsupported records and typed absence for other or out-of-range indices.
Mismatched or missing source returns a handoff error without changing ordinary
reader refusal or limit precedence. The closed wire and complete content are unchanged.

For FR-035-AC-23 through FR-035-AC-25, admit V2 source operations with
populated mode, law definitions and member declaration and with all three
legitimately absent. Lower each into an in-memory `ContractPackageV1` and
compare the source and keyed lowered `CheckedTerm` for represented-body equality,
then compare each fallible typed accessor's admitted value. Repeat with represented
nodes reordered; key lookup must still return the matching operation views.
Compare canonical V1 bytes before and after view retention.
Construct a direct-deserialized term lacking validated views; assert its
accessors return typed `NotAdmitted`, distinct from admitted absence. Read a
canonical V1 package with a represented application operation through the
plain strict reader without V2 source evidence, including at the exact work
limit; assert admission, exact node/kind and keyed term lookup, and producer/plain
term equality for every represented key under AC-22. Check both a body-root and
a nested application operation. On each plain-read operation, assert all five
fallible view accessors return typed `NotAdmitted` although the corresponding
producer accessors retain admitted populated or absent values. Read a canonical
package without represented operations through the same reader and compare
every represented term with the producer's term for AC-22 equality. A wrong
dependency, source selection or exhausted limit must still follow the plain reader's
existing refusal or incomplete result, without a new operation-source code.
Read the same bytes through an explicit authenticated handoff with the exact
reader-admitted V2 source. Compare source, producer and reader terms and all
three typed accessors, including their admitted absence cases. In otherwise
canonical V1 packages, separately supply a wrong source identity, wrong lock,
absent represented node, changed represented operation body and changed
represented non-operation body; each
must return a typed handoff error and no partially authenticated package. Repeat the plain reader's
strict wire, source selection, dependency and limit probes through the authenticated
handoff to show it does not bypass or reorder those decisions. At a consumer
holding only `&ContractPackageV1`, use the authenticated value to assert
routed quantity conversion and checked-graph association, and assert a
`NotAdmitted` view cannot silently omit an operation-dependent step or cause a
fallback JSON parse.

## Status

PLANNED/UNRUN (IR-1141): the existing tagged V1 tests and current implementation still contain six-member and digest assertions. The structural five-member FR-330 contract requires code and test changes after QSpec #517/#518. Existing unaffected node, source-map, limit and typed-view controls retain their prior evidence only where their assertions do not depend on a removed identity field.
