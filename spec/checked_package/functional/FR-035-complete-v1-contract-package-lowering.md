---
id: FR-035
title: "Lower complete-V1 contracts into the target-neutral ContractPackage"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-195
    type: references
  - target: ix://agent-ix/quire-specification/FR-330
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# FR-035: Lower complete-V1 contracts into the target-neutral ContractPackage

## Description

When Contract IR lowers an admitted checked complete-V1 request, the lowerer SHALL produce one immutable QSpec FR-330 `ContractPackage` under
`quire.contract-package/v1` and one exact disposition for each requested item.
The package is the single published target-neutral lowering value for backend
consumption. Contract IR emits this wire; codegen consumption remains IR-744.

## Inputs

An immutable admitted `quire.checked-package/v2`, its selected admitted
dependency packages, requested item set, resolved `CompleteLoweringProfileV2`
and resource limits. The resolved profile owns `reference: LoweringProfileRef`,
`required_source_selections: Vec<CheckedSelection>`, and its existing
`supported_tags`, `require_bounds`, and `work_limit`. `CheckedSelection` is the
existing typed FR-322 `Selection` `{role, definition}`, where `definition` is
`DefinitionRef` `{authority, identity}`. The producer of the resolved profile
supplies its declared requirements directly; no lookup from the reference,
parsed identifier, display name, or untyped settings map substitutes for them.
The requirements are lowering input, not emitted ContractPackage members.

## Outputs

The package has exactly the FR-330 properties `contract_version`, `package`,
`semantic_graph`, `source_map`, and `feature_accounting`.
`contract_version` is exactly `quire.contract-package/v1`; `package` retains
the originating checked-package identity and lock and the explicit closed
`LoweringProfileRef` `{authority, identity}` separately from the lock's source
profile selections. `semantic_graph` is one
canonically ordered tagged-node collection containing all represented type,
expression, model, relation, clause and analysis meaning. Node tags, typed
keys, source occurrences and compatible reference targets follow QSpec
FR-322. There are no independent type, expression, clause or protocol graphs.

`feature_accounting` has exactly one record for each requested item, drawn
from the closed seven-member vocabulary FR-038 declares: `lowered`,
`unsupported`, `requires_bound`, `invalid_input`, `failed`, `invalid_body` and
`body_incomplete`. A refusal retains its exact source locus and contributes no
substitute node. `source_map` gives total native semantic and source
correspondence for represented nodes and requested items, including the locus
of each refusal. The reader validates exact source correspondence. A valid source-location edit changes the complete package value while retaining the selected package name; invalid or incomplete correspondence refuses admission. Presentation is excluded from semantic authority. No content digest is produced.

## Behavior

QSpec FR-330's "Typed source-selection compatibility" is the sole owner of
source-selection matching, its charged per-request procedure, refusal precedence
and exact lock preservation. The lowerer SHALL implement that contract using
its typed `CompleteLoweringProfileV2.required_source_selections` field and
existing FR-038 per-request work meter. The lowerer SHALL NOT introduce a separately
normalized requirement representation, replacement budget or reference-derived
requirement lookup.

The typed adapter SHALL map FR-330's compatibility refusal to the existing
`CompleteLoweringRecordV2::InvalidInput { node_id }` and its `invalid_input` wire
projection. A stopped charge maps to the existing `Failed` variant for `Work`,
retaining the specified requested key, limit and consumed counter; it does not
convert a work failure into an input refusal. Continued lowering uses that same
meter. The output adapter retains `profile.reference` as the sole emitted
lowering-profile value and retains the original checked source provenance under
FR-330, without adding a requirement member, diagnostic payload or vocabulary.

One call SHALL assemble the graph from its `lowered` records and the admitted
local nodes those reach: each node once. An explicit recursion group's represented
members SHALL retain their admitted graph order, because FR-322 includes each
member's graph-order ordinal in its structural id. A group forms one block keyed
by its least represented FR-322 typed key. The lowerer SHALL order group blocks
and ungrouped nodes by that key. Thus a graph without recursion remains in
ascending typed-key order.
A refused request contributes no node of its own unless a lowered item reaches
that node as exact represented meaning. Every local semantic-node reference SHALL resolve to a compatible node in this graph by its key, never by display
name or array position. A `dependency_reference` SHALL retain its selected
dependency package identity and resolve against that package's admitted graph
under FR-322; its target SHALL NOT be flattened into this package's local
graph. An explicit FR-322 recursion group remains a valid keyed graph cycle, while
serialized values never embed recursive objects. An undeclared cycle refuses.

The lowerer SHALL preserve scalar and composite types, bounded domains, typed
expressions, pure functions, model identities and relations, state frames and
transitions, temporal formulas, protocol control/queues/obligations, claims,
and exact source correspondence. It SHALL refuse a requested item whose
meaning cannot be represented exactly under the selected profile. An absent
selected dependency, wrong selected package name, wrong target family,
invalid source/type/anchor/bound join, or unknown contract version SHALL never
produce substitute meaning or a backend artifact. A valid body edit in a
selected package under the same name is checked by its independent graph,
owner and body rules and may admit; no content-staleness refusal exists. A
refusal for one item SHALL leave independent sibling dispositions unchanged. If the package bytes exceed the declared ceiling,
every requested record is `failed`, because no package can carry a lowered
node (FR-038-AC-95).

For an exact `CheckedNodeId` that has a `lowered` feature-accounting record,
the admitted `ContractPackageV1` SHALL expose a read-only typed view of that
root and its local reached closure. The view SHALL include the root and every
local node reachable through `semantic_type`, declared `dependencies`, and
body references interpreted by the same FR-322 body walk used during admission
and lowering, once each in ascending typed-key order. It SHALL retain each
reached node's exact native semantic and source-map correspondence. Reached
nodes are represented by their existing typed keys and nodes; a reached node
has no independent `ir_id` or closure record. A selected external dependency
reference SHALL remain a typed pair of selected dependency identity and target
key at its admitted package boundary. It SHALL NOT be resolved through an
identically keyed local node or flattened into this package's local closure.
For every lowered root, the view SHALL expose both bound and claim typed-key
subsets, including an empty subset when that class has no reached member. Each
subset SHALL contain exactly the keys derived by Contract IR's existing
lowering classification in typed-key order; consumers SHALL NOT infer either
subset from node tags.
For an exact requested key with a non-`lowered` record, the view SHALL return
that typed disposition. When a key has no lowered request record, the view SHALL
return typed absence, including for an absent key or a node present only because
another root reaches it. Neither case SHALL synthesize a root closure.
This read-only view SHALL leave the admitted five-field FR-330 wire and its
canonical bytes unchanged.

For each represented application operation in a package assembled from an
admitted V2 source, lowering SHALL carry that source operation's validated
typed views into the `ContractPackageV1` term at the same `CheckedNodeId`.
The source and lowered terms SHALL compare equal as represented typed bodies;
separately, their populated views and legitimate absence SHALL agree:
`mode()`, `law_definitions()` and
`member_declaration()` SHALL report the same admitted typed values on both.
Lowering SHALL NOT derive these views by deserializing the emitted operation
JSON or by accepting an unvalidated term as admitted. The retained views are
read-only admission facts, not new FR-330 wire members, and SHALL leave the
five wire members, canonical bytes unchanged.

Contract IR SHALL distinguish authenticated operation views from an operation
with no such views on a `ContractPackageV1` term reconstructed by strict
reading of V1 wire bytes. The sole plain strict V1 reader SHALL continue to
admit every otherwise valid canonical package, including one with represented
application operations, under its existing dependency, identity and exact-limit
rules. It SHALL return the represented terms through the existing keyed lookup.
The FR-038 accessors SHALL return typed `NotAdmitted` on those terms without
authenticated operation views, distinct from admitted absence.

`CheckedTerm` equality SHALL compare the represented typed body. In an
application operation it SHALL compare the operation's represented record and
execution point; it SHALL NOT compare the private validated-view cache or its
presence. The cache is an admission fact outside the represented V1 body.
Thus an operation term returned by a valid plain strict read SHALL compare
equal to the producer's term under the unchanged FR-035-AC-22 obligation,
while their view accessors still report different provenance. This semantic
equality SHALL apply to nested application operations as well as body roots;
it SHALL NOT make `NotAdmitted` equivalent to admitted absence, install views,
or change wire members, canonical bytes.

The strict reader SHALL offer an explicit authenticated source handoff accepting
the reader-admitted V2 source package as an input. It SHALL first apply the
same strict V1 wire, correspondence, identity, dependency and limit checks,
without bypassing or reordering them. The authenticated handoff SHALL compare
the V1 wire's source package identity and lock with that source and, for every
represented node, its full node key, kind, semantic type, dependencies and body
with the admitted source node before installing views. Only the corresponding
source term's validated views may be installed. For a missing or mismatched
source or represented node, the authenticated handoff SHALL return a typed
handoff error and no partially authenticated package;
that error SHALL NOT be a new FR-330 plain-reader refusal code. A directly
deserialized operation with no validated views remains distinct from admitted
absence by FR-038's typed `NotAdmitted` accessor refusal. A consumer requiring
operation facts to generate behavior SHALL use the authenticated handoff or
propagate `NotAdmitted` rather than silently skip that behavior.

The admitted package SHALL retain each represented local node's exact
`CheckedNodeKind`, aligned with that node after canonical graph ordering,
including recursion-group member ordering. Lowering SHALL carry the kind from
the admitted checked source; strict reading SHALL retain the kind validated
from the canonical wire. A query by represented local typed key SHALL return
that kind, and each root and reached node in the typed closure SHALL expose it.
An absent or unrepresented local key SHALL have no kind. A selected
dependency's kind SHALL NOT be presented as the kind of a local node.

The admitted package SHALL also retain the exact typed `CheckedNodeTag` for
each `unsupported` feature-accounting record at its request position, including
when its requested node is absent from `semantic_graph`. The position is the
zero-based index in FR-330 `feature_accounting`, independent of a consumer's
request or envelope index. Lowering SHALL carry
the typed tag from its disposition; strict reading SHALL retain the tag
validated from that record. A query by feature-accounting index SHALL return
that tag; an out-of-range index or a record with another
disposition SHALL have no unsupported tag. These read-only kinds and tags SHALL
add no FR-330 wire member and SHALL NOT change canonical bytes.

For each `unsupported` record, the admitted package SHALL expose the requested
root's exact typed `CheckedNodeKind` at that record's zero-based
`feature_accounting` index. This is the kind of the record's `node_id` in the
reader-admitted V2 source, even when that node is absent from the represented
graph or the first unsupported reachable `unsupported_node_id` names a different
node or form. The producer SHALL retain this kind from its admitted source.
`read_with_source` SHALL retain it only after authenticating the caller-supplied
V2 source against the V1 source package identity and lock and validating the
represented nodes as it already does. When an unsupported requested root is
absent from that source, Contract IR SHALL return a typed handoff error and
no partially authenticated package.
The authenticated handoff SHALL NOT infer the requested root's kind from the
first unsupported reachable node's `node_tag` or from a same-tag represented
node. For an `unsupported` record admitted by plain strict `read`,
`ContractPackageV1` SHALL return typed `NotAdmitted` for the requested-root
kind, even if the requested key happens to be represented through another
root. An out-of-range index or a record with another disposition SHALL return
typed absence, distinguishable from `NotAdmitted`. The view SHALL NOT alter
the seven-member accounting vocabulary, FR-330's closed `unsupported` payload,
any of the five V1 wire members or canonical bytes. The existing
`unsupported_node_id` and `node_tag` SHALL retain their first-reachable-node
meaning; this requested-root kind describes a different, exact locus.

When reading a canonical `ContractPackageWireV1`, Contract IR SHALL refuse an
unrecognized string `contract_version` with `unknown_contract_version` located
at `/contract_version`, retaining the supplied version in the diagnostic.
When an otherwise valid canonical `ContractPackageWireV1` omits any of its five
required top-level members (`contract_version`, `package`, `semantic_graph`,
`source_map` or `feature_accounting`), Contract IR SHALL refuse with
`invalid_package` at the root path `""` and cause `missing_member`.
When the reader receives otherwise valid `ContractPackageWireV1` bytes with
leading whitespace before the canonical JSON value, Contract IR SHALL refuse with
`noncanonical_wire`, absent path, and absent cause.
When an otherwise valid canonical `ContractPackageWireV1` has a first
`feature_accounting` record naming a requested node other than the first
`source_map.requested_items` entry, Contract IR SHALL refuse with
`invalid_package` at `/feature_accounting` and cause `invalid_value`.
When a caller-selected read limit is exhausted, Contract IR SHALL return
`Incomplete` with the exhausted limit kind, ceiling, consumed count and the
pointer of the failed charge where one exists.
The V1 reader SHALL apply `CheckedPackageReadLimits.work` once to the total
semantic-term validation visits across value and reference validation.
The V1 reader SHALL carry the work counter into reference validation without
reset and report the cumulative count and failed-charge pointer at a work stop.
The V1 reader SHALL charge one unit for each source-map occurrence before
charging one unit for each of its regions in array order, visiting occurrences
in array order within `nodes`, then `requested_items`, then `refusals`.
The V1 reader SHALL report the pointer of the occurrence or region whose charge
crosses the occurrence limit and the count including that failed charge.
The V1 reader SHALL return `Incomplete` when the combined value and reference
validation charges exceed `limits.work`, even if each pass fits separately.
The V1 reader SHALL preserve canonical package bytes and admitted package values.
When the reader returns `Refused` or `Incomplete`, Contract IR SHALL
admit no `ContractPackageV1`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-035-AC-1 | Each V1-BACK-001 through V1-BACK-004 and V1-BACK-014 requested item has exact represented meaning under its selected profile or one located record from the seven-member disposition vocabulary; no refusal creates substitute meaning. | Test |
| FR-035-AC-2 | A mutation that makes a source selection missing, a type or anchor join invalid, a bound ill-typed, a dependency absent or wrong-named, or a contract version unsupported refuses under its applicable existing rule before a backend artifact. An identically keyed local node cannot satisfy a selected external dependency. A correctly renamed or validly edited selected package may admit under the same rules; changed meaning alone is not a content-staleness refusal. | Test |
| FR-035-AC-3 | Mixed supported and refused requests retain one disposition each and independent sibling results, except the package byte ceiling fails every record under FR-038-AC-95. | Test |
| FR-035-AC-4 | Every local reference resolves by typed key to a compatible local node with exact source correspondence; a dependency reference resolves only through its selected admitted dependency; explicit same-group recursion admits and an undeclared cycle refuses. | Test |
| FR-035-AC-5 | Withdrawn by the IR-1141 no-hash decision under QSpec #517/#518: the six-member V1 package and its canonical semantic digest are removed. This id is retained and never reused. | Withdrawn |
| FR-035-AC-6 | For an admitted lowered root, the typed view returns that exact root and each local node reached through `semantic_type`, declared `dependencies` or validated body references once in ascending typed-key order; independently walking those three edge sources yields the same key set, including a shared or cyclic reference, and every returned node retains its exact source correspondence. | Test |
| FR-035-AC-7 | A selected dependency reference in the view retains its selected dependency identity and target key and cannot be satisfied by an identically keyed local node, an absent package or a differently named selected package; every lowered-root view exposes both bound and claim typed-key subsets, possibly empty, exactly matching Contract IR's existing lowering classification in typed-key order. | Test |
| FR-035-AC-8 | Querying a non-lowered requested key returns its exact typed disposition, while an unrequested or absent key returns typed absence even when another root reaches the node. Neither fabricates a closure nor changes any of FR-330’s five wire members or canonical bytes. | Test |
| FR-035-AC-9 | The typed profile exposes `required_source_selections: Vec<CheckedSelection>` and the IR adapter conforms to QSpec FR-330-AC-10 and TC-935: input refusals use `InvalidInput { node_id }`/`invalid_input`, work stops use existing `Failed`/`Work` with unchanged key/limit/consumed counters, and successful checks continue the existing FR-038 meter without reset. The adapter preserves exact source provenance and emits only `profile.reference`, with no duplicated requirement wire field or new refusal payload. | Test |
| FR-035-AC-10 | Given an otherwise valid canonical `ContractPackageWireV1` whose string `contract_version` differs from `quire.contract-package/v1`, `ContractPackageWireV1::read` returns `Refused` with code `unknown_contract_version`, path `/contract_version`, and `contract_version` equal to the supplied string; it returns no admitted package. | Test |
| FR-035-AC-11 | Given otherwise valid canonical package bytes and a caller-selected byte, node, work, edge or source-map occurrence limit exhausted by the reader, `ContractPackageWireV1::read` returns `Incomplete(CheckedPackageIncomplete)` rather than `Admitted`, carrying the exhausted `limit_kind`, selected `limit`, and `consumed` amount at the failed charge. `path` is absent for bytes, `/semantic_graph/nodes` for nodes, `/semantic_graph/nodes/<i>/dependencies` for edges, and the failing source-map value for occurrences. For work it is `/semantic_graph/nodes/<i>/body` when value validation stops, or the failed-charge pointer reported by reference validation, which may name another node member. No `ContractPackageV1` is admitted. The work count and occurrence pointer follow AC-15 and AC-16. | Test |
| FR-035-AC-12 | Omitting any of the five required top-level V1 members (contract_version, package, semantic_graph, source_map, feature_accounting) from otherwise valid canonical bytes makes ContractPackageWireV1::read return Refused/invalid_package/missing_member at the root with no admitted package. An added identity member refuses as unknown_member. | Test |
| FR-035-AC-13 | Given otherwise valid `ContractPackageWireV1` bytes with leading whitespace before the canonical JSON value, `ContractPackageWireV1::read` returns `Refused` with code `noncanonical_wire`, absent path, and absent cause; it returns no admitted package. | Test |
| FR-035-AC-14 | Given an otherwise valid canonical `ContractPackageWireV1` except that its first `feature_accounting` record names a requested node other than the first `source_map.requested_items` entry, `ContractPackageWireV1::read` returns `Refused` with code `invalid_package`, path `/feature_accounting`, and cause `invalid_value`; it returns no admitted package. | Test |
| FR-035-AC-15 | For an otherwise valid canonical package on which both value and reference validation charge work, the reader admits when `limits.work` equals the independently counted total visits across both passes and returns `Incomplete` when it is one less, with `limit_kind = Work`, the selected limit, `consumed` equal to the cumulative count at the first failed charge, and `path` naming that charge in either pass. A limit that fits each pass separately but is below their combined count also returns `Incomplete`, never `Admitted`; no second meter restarts at zero. | Test |
| FR-035-AC-16 | For an otherwise valid canonical package with source-map occurrences and regions, a limit one below the independently counted combined occurrence and region charges returns `Incomplete` with `limit_kind = Occurrences`, the selected limit, `consumed` equal to the count at the first failed charge, and `path` naming the particular occurrence value under `/source_map/nodes/<i>/occurrences/<j>`, `/source_map/requested_items/<i>/occurrences/<j>` or `/source_map/refusals/<i>/occurrences/<j>`, or its `/regions/<k>` member, whose charge crossed the limit. A limit equal to that total admits; a stop does not report only `/source_map`. | Test |
| FR-035-AC-17 | Given an otherwise valid canonical `ContractPackageWireV1` with an `invalid_body.refusal` whose code is `unknown_contract_version` or `unknown_required_capability`, the reader admits that nested record when `cause` is absent and refuses a present `cause` as `invalid_package`/`invalid-value` at the nested `/cause` pointer, including `unsupported-wire` and `unsupported-feature` respectively, as QSpec FR-322-AC-78 and FR-330 require. | Test (TC-047) |
| FR-035-AC-18 | PLANNED/UNRUN (IR-1141). If two dependency paths select one name but supply unequal complete JSON packages, ContractPackageWireV1::read refuses invalid_package/conflicting-definition before graph references. Its two ordered refusal `loci` name the exact importer-rooted selection paths in declaration order. Equal complete values through a diamond admit. | Test (TC-047) |
| FR-035-AC-19 | For a lowered root whose represented source nodes are reordered by canonical typed key, including a recursion-group block sorted by its least typed key while its members retain admitted graph order, producer admission and strict reading of its canonical bytes return the same exact `CheckedNodeKind` for each represented local key, closure root and reached node. An absent or unrepresented local key returns no kind, and a selected dependency's kind is never substituted for a local kind. | Test |
| FR-035-AC-20 | For mixed accounting with an `unsupported` request whose node is absent from the represented graph, producer admission and strict reading return its exact typed `CheckedNodeTag` at its zero-based FR-330 `feature_accounting` index. Querying an out-of-range index or any non-`unsupported` record returns no tag, even when its requested node is represented; interposed consumer context or excluded requests do not shift this accounting index. | Test |
| FR-035-AC-21 | Retaining kinds and unsupported tags changes none of the five FR-330 wire members or canonical bytes; malformed node kind and unsupported tag preserve their strict reader refusal and exhausted limits return Incomplete without an admitted typed view. | Test |
| FR-035-AC-22 | For every represented local key, the producer and strict reader return the exact node and same optional CheckedTerm admitted from its source body through keyed lookups, independent of graph ordinal. An absent key returns neither; retained lookups change no wire member or canonical byte. | Test (TC-047) |
| FR-035-AC-23 | PLANNED (IR-1087). For a V2 reader-admitted operation with a populated mode, ordered law definitions and member declaration, the producer's lowered `ContractPackageV1::term` at its exact key has the source term's validated views and returns the same typed values; the source and lowered `CheckedTerm` compare equal as represented typed bodies. Reordering represented graph nodes does not misassociate views, and lowering does not deserialize emitted operation JSON to construct them. | Test (TC-047) |
| FR-035-AC-24 | PLANNED (IR-1087). For a V2 reader-admitted operation legitimately lacking mode, laws and member declaration, source and lowered terms compare equal and return admitted absence or an empty law view. View retention changes none of the five FR-330 wire members or canonical bytes. | Test (TC-047) |
| FR-035-AC-25 | PLANNED (IR-1087). A never-admitted direct-deserialized operation returns typed `NotAdmitted` for mode, law definitions and member declaration, distinctly from AC-24's admitted absence. Plain strict V1 reading admits an otherwise valid canonical package with represented application operations, including at the exact work limit, and returns each represented term through keyed lookup; its operation views return typed `NotAdmitted`, while its terms compare equal to the producer's terms under AC-22. A consumer must use authenticated read or propagate `NotAdmitted`; it cannot silently omit operation-dependent behavior or parse JSON to recover the facts. | Test (TC-047) |
| FR-035-AC-26 | PLANNED (IR-1087). With the exact reader-admitted V2 source supplied through an explicit authenticated handoff, strict reading the lowerer's canonical V1 bytes returns a `ContractPackageV1` whose every represented term equals the producer's and source's term as a represented typed body and whose operation views separately match the source's populated and legitimately absent admitted values. A mismatched source identity or lock, absent represented source node, or changed represented operation or non-operation node produces a typed handoff error and no partially authenticated package, even when the V1 wire is internally canonical; this is not a new plain-reader refusal code. The handoff retains the plain reader's wire, dependency and limit decisions. The CG consumer's routed quantity-conversion and checked-graph association controls pass using only this admitted V1 value, without a JSON bridge. | Test (TC-047) |
| FR-035-AC-27 | PLANNED (IR-1104). For producer-admitted mixed accounting, the requested-root kind at each `unsupported` accounting index equals the exact `CheckedNodeKind` of that record's `node_id` in the admitted V2 source, including an unrepresented `Temporal(TemporalClause)` root whose first unsupported reachable node is a distinct `Temporal(Fairness)` node referenced in its fairness argument, and separately direct `Temporal(Fairness)` and `Temporal(Formula)` requests. Interposing a healthy sibling or another request does not shift the index, change the sibling disposition, or substitute the first unsupported node's kind. | Test (TC-047) |
| FR-035-AC-28 | PLANNED (IR-1104). `read_with_source` of the producer's canonical V1 bytes and exact reader-admitted V2 source returns the same requested-root kind for every `unsupported` accounting index as the producer. A mismatched source identity or lock, or a source missing an `unsupported` requested root, returns a typed handoff error and no authenticated package; ordinary V1 reader refusal or limit outcomes retain their existing precedence. No requested-root kind is inferred from `node_tag`, `unsupported_node_id`, or a same-tag represented node. | Test (TC-047) |
| FR-035-AC-29 | PLANNED (IR-1104). For a canonical V1 package admitted by plain `read`, querying an `unsupported` accounting index returns typed `NotAdmitted` for the requested-root kind, including when the requested key is represented through another root. A non-`unsupported` or out-of-range index returns typed absence, distinct from `NotAdmitted`. Neither query changes the existing `unsupported` record's first-reachable `unsupported_node_id`/`node_tag` or an independent sibling's result. | Test (TC-047) |
| FR-035-AC-30 | PLANNED (IR-1104). Producer and authenticated reader requested-root-kind queries change none of the five FR-330 wire members, the closed unsupported payload or canonical bytes. Fairness, formula and temporal-clause requests remain distinguishable through the typed view. | Test (TC-047) |

FR-035-AC-12 through FR-035-AC-14 are backed by tagged TC-047 reader tests in
`tests/it/complete_v1_contract_package.rs`; the specified workspace test gate, lint, and grammar validation pass.

## Dependencies

[FR-028](../../model/functional/FR-028-separate-cycle-free-contract-model.md)
owns the cycle-free model boundary. QSpec FR-330 and I12 own the one published
ContractPackage contract. QSpec FR-322 owns the checked graph's node forms,
keys, recursion and selected dependency-reference rules; FR-195 owns
complete-V1 lowering semantics. This FR applies those contracts at the IR
lowering boundary. Contract IR implements the FR-330 package emitter and strict
reader under IR-743; backend adoption remains planned in IR-744.
