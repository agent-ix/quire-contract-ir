---
id: FR-025
title: "Project checked native predicates into TL Boolean propositions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-019
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-048
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-095
    type: references
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/issues/52
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/64
    type: references
---
# FR-025: Project checked native predicates into TL Boolean propositions

## Description

When a checked native Quire predicate is selected for temporal lowering, the
Contract IR bridge shall bind its exact definition and evaluation environment
to one deterministic TL Boolean signal/proposition correspondence without
parsing source text, evaluating either language, or coercing a non-Boolean or
non-final value.

The selected bridge profile is
`quire.contract.native-predicate-tl-projection/v1`. Native Quire remains the
sole editable formal-clause language. The TL catalog, proposition binding and
valuation are derived internal inputs to temporal evaluation, never an
alternative predicate authoring surface.

## Inputs

- An exact accepted native-language/profile definition identity, revision and
  digest, plus the source-document identity, revision, digest, clause span and
  predicate-expression span.
- The selected native contract's authority-verified source-bound checked-leaf
  record and exact bytes. Its public strict reader must bind a
  `source_leaf_ref` and parent `source_subject_ref` to the FR-012 `ClauseRef`,
  source/clause/expression spans, binding-requirements identity,
  model/declaration-closure identity, canonical declaration identity,
  canonical typed-expression identity, Boolean expected type and native
  evaluation/definedness profile identities. This record represents either an
  inline native temporal `holds(expr)` leaf or an admitted whole-clause Boolean
  leaf without manufacturing a standalone `BoundClause` for inline syntax.
- An explicit finite population of distinct checked predicate definitions
  selected by the caller. This formula-independent bridge does not claim that
  the set covers every atom in a temporal formula.
- For each valuation request: the expected producer/profile identity and exact
  source-result contract and mapping `ContractSelection` values,
  observation revision, anchor/snapshot/invocation identity, immutable capture-
  environment identity, model/population identities, an exact accepted
  result-availability authority contract and assertion identity/revision/digest,
  the exact assertion bytes supplied as a sibling input,
  its `observation_state` equal to `open`, `closed-complete` or
  `closed-incomplete`, and `result_availability`
  equal to `available`, `not-yet-observed`, `producer-unavailable` or
  `contract-unavailable`.
- When a producer result is available: its exact bytes, producer-owned identity,
  revision and digest; a derived result projection
  carrying independent predicate-execution, native-truth and completeness axes;
  typed value; declared deciding-fact identities; the complete bounded
  observation-fact population with each fact's canonical identity and
  `available`, `missing`, `incomplete` or `contradicted` status; and an
  optional complete immutable prior source result plus its projection.
- Target contracts `tl-syntax.signal-catalog/v1` and
  `tl-syntax.proposition-map/v1`, their accepted release identities, limits and
  public strict readers.

## Outputs

- A `PredicateProjectionSetDecision` with kind `admitted`, `unavailable`,
  `unsupported`, `refused` or `conflict`.
- An admitted decision contains one immutable `PredicateCorrespondence` per
  distinct predicate, the exact catalog/map references and `projection_set_ref`.
  The operation returns the complete derived Boolean signal-catalog and
  proposition-map documents as separately bounded sibling artifacts; every
  other kind returns neither artifact and retains an ordered nonempty cause set.
- A `PredicateValuationDecision` for one correspondence and the three supplied
  contract selections, with kind `valued`, `incomplete`, `unavailable`,
  `unsupported`, `failed`, `refused` or `conflict`. A contract-admission failure
  stops before observation binding; every post-contract-admission decision binds
  one exact observation. Only `valued` contains exactly one Boolean value; it
  also contains the ordered completeness gaps outside its deciding-fact set.
- Every non-admitted or non-valued decision contains a nonempty ordered set of
  typed causes. Each cause retains the exact rejected identity when one exists
  and the bounded raw discriminator when an unknown profile or construct caused
  the decision.
  It contains no usable partial catalog, binding, or Boolean value.

## Public v1 records

Each public `format` field shall equal the profile literal that introduces its
record below.

The derived producer-input profile shall be
`quire.contract.native-predicate-result-projection/v1`. Its strict record
contains exactly `format`, `source_result_contract`, `source_result_ref`,
`source_result_mapping`, `source_result_revision`, `source_result_digest`, `predicate_ref`,
`producer_profile`, `observation_revision`, `predicate_execution`,
`native_truth`, `value_kind`, `value_ref`, `boolean_value`, `anchor_ref`,
`snapshot_ref`, `invocation_ref`, `capture_ref`, `model_ref`,
`population_ref`, `completeness`, `deciding_fact_refs`,
`observation_facts`, and `prior_result_ref`. `source_result_contract` and
`source_result_mapping` are `ContractSelection` values;
`source_result_ref` remains producer-owned and
`source_result_digest` is lowercase SHA-256 over the exact source bytes.
`source_result_revision` and `observation_revision` are positive u64. `value_kind` is
one of `boolean`, `integer`, `rational`, `text`, `enum`, `record`, `option`,
`collection`, `error`, or `unavailable`; `boolean_value` is a JSON Boolean only
for `boolean` and is the empty string for every other kind. `value_ref` is the
exact lowercase 64-hex producer value identity except that it is empty for
`unavailable`. `prior_result_ref` is either
empty or the lowercase 64-hex identity of the supplied immutable predecessor.

The local `result_projection_ref` shall be lowercase SHA-256 over the exact
bytes `quire-contract-ir`, one zero byte, the derived producer-input profile,
one zero byte, and the canonical projection-record bytes. Canonical projection
records use compact
UTF-8 JSON, Unicode-scalar object-key order, FR-016 scalar rules, sorted distinct
fact-reference arrays, and the exact observation-fact/completeness shapes below.
The local reference is supplied outside the hashed record and shall be
recomputed before any valuation decision. It shall not replace, restamp or
claim authority for `source_result_ref`.

The public projection-set decision profile shall be
`quire.contract.native-predicate-projection-decision/v1`. Its admitted tagged
shape contains exactly `format`, `kind`, `bridge_profile`, `native_contract`,
`target_contracts`, `correspondences`, `signal_catalog_ref`,
`proposition_map_ref`, `projection_set_ref`, and `causes`; `kind` is `admitted`
and `causes` is empty. `target_contracts` contains exactly `proposition_map` and
`signal_catalog`, each a `ContractSelection`.
Its other tagged shape contains exactly `format`, `kind`, and nonempty `causes`.
It contains none of the admitted-only fields.

The public valuation decision profile shall be
`quire.contract.native-predicate-valuation-decision/v1`. Every tagged shape
starts with exactly the base fields `format`, `kind`, `predicate_ref`, `projection_set_ref`,
`proposition_id`, `signal_id`, `source_result_contract`,
`source_result_mapping`, `producer_profile`, `availability_contract`, and
`causes`.

A contract-admission-failure shape has kind `unsupported` or `unavailable`, a
nonempty cause set containing only `availability-contract`,
`source-result-contract` or `source-result-mapping` causes, and exactly the base
fields. It omits assertion, observation and result-derived fields because no
unadmitted reader or mapping can authenticate them.

Every post-contract-admission shape additionally contains exactly
`observation_revision`,
`anchor_ref`, `snapshot_ref`, `invocation_ref`, `capture_ref`, `model_ref`,
`population_ref`, `availability_assertion_ref`,
`availability_assertion_revision`, `availability_assertion_digest`,
`observation_state`, and `result_availability`.

`availability_contract` shall be a `ContractSelection`; assertion revision is
positive u64 and the assertion digest is lowercase 64-hex. The selected
authority's public strict reader shall validate the exact assertion bytes and
verify its identity, revision, digest, observation state and result-availability
classification before the bridge constructs any post-contract-admission
valuation decision. This prerequisite does not apply to the exact
contract-admission-failure shape.

The `unavailable` shape with `producer-unavailable` or `contract-unavailable`,
and the `incomplete` shape with `not-yet-observed`, shall contain only the
post-contract-admission fields and a nonempty cause set because no producer
result exists. Every post-contract-admission shape with `available` shall
additionally contain exactly `source_result_ref`,
`result_projection_ref`,
`predicate_execution`, `native_truth`, `completeness`, `deciding_fact_refs`,
`completeness_gaps`, and `prior_result_ref`. The `valued` shape shall also
contain exactly one `value` field whose value is JSON `true` or `false`; every
other shape shall omit `value`. An absent direct predecessor shall be encoded
as an empty `prior_result_ref` string, not JSON null. A result-bearing shape
shall never fabricate an absent producer result, and an unavailable shape
shall never contain result-derived fields.

Each `PredicateCorrespondence` shall contain exactly `predicate_ref`,
`proposition_id`, `proposition_name`, `signal_id`, and `signal_name`. Each cause
shall contain exactly `dimension`, `code`, and `rejected_ref`, plus
`raw_discriminator` only for an unknown well-formed input value. Identity,
profile and present rejected-reference strings contain 1 through 1,024 UTF-8
bytes; absence of a rejected identity is the empty `rejected_ref` string.
`raw_discriminator` contains 1 through 4,096 UTF-8 bytes. Every identity string
except the explicitly empty `prior_result_ref`, `rejected_ref` or unavailable
`value_ref` contains 1
through 1,024 UTF-8 bytes. Cause arrays contain at most 1,024 entries. Correspondence, deciding-fact,
observation-fact and completeness-gap arrays contain at most 10,000 entries.
Each input or output document contains at most 67,108,864 bytes and semantic
depth 256.

Each observation-fact record shall contain exactly `fact_ref` and `status`;
`fact_ref` is a lowercase 64-hex canonical producer identity and `status` is
`available`, `missing`, `incomplete` or `contradicted`. The complete population
shall sort by `fact_ref` and contain no duplicate. The completeness record shall
contain exactly `assertion_ref`, `boundary_ref`, `population_ref`, and `state`;
the three references are lowercase 64-hex and `state` is `complete`,
`incomplete` or `contradicted`. `deciding_fact_refs` shall be a sorted distinct
subset of the observation-fact population. `completeness_gaps` shall contain
the sorted observation-fact records whose status is not `available` and whose
identity is outside `deciding_fact_refs`.

The bridge shall recompute `population_ref` as lowercase SHA-256 over the bytes
`quire-contract-ir`, one zero byte,
`quire.contract.native-predicate-observation-population/v1`, one zero byte, and
the canonical JSON observation-fact array. It shall reject a supplied mismatch.

The bridge shall require completeness `complete` exactly when every observation
fact is `available`, `incomplete` exactly when at least one fact is `missing` or
`incomplete` and none is `contradicted`, and `contradicted` exactly when at
least one fact is `contradicted`. This population-level state does not by itself
erase truth settled by a disjoint exact deciding-fact set.

The bridge shall serialize every public result and sibling artifact as compact
UTF-8 JSON with Unicode-scalar object-key order, FR-016 scalar rules and the
exact tagged shapes above. Decoding and re-encoding shall preserve exact bytes.

The bridge shall define `signal_catalog_ref` as lowercase SHA-256 over
`quire-contract-ir`, one zero byte,
`quire.contract.tl-signal-catalog-artifact/v1`, one zero byte, and the exact
catalog bytes.

The bridge shall define `proposition_map_ref` by the same construction with
domain `quire.contract.tl-proposition-map-artifact/v1` and the exact map bytes.

The bridge shall reject duplicate or unknown object members, invalid UTF-8,
trailing JSON, invalid enum values, absent required fields, forbidden variant
fields, and any byte/depth/count/string overrun. It shall return one
`invalid_native_predicate_projection` diagnostic at the narrowest public field
path and no partial decision.

If projection-set or valuation allocation/reservation fails after structural
admission, then the bridge shall return respectively
`predicate_projection_resource_exhausted` or
`predicate_valuation_resource_exhausted` with no decision or partial artifact.

## Behavior

The bridge shall accept only an authority-verified source-bound checked-leaf
record under the selected native contract. The selected contract's public
strict reader shall verify its exact bytes, `source_leaf_ref`, parent
`source_subject_ref`, clause and expression locations, binding requirements,
typed-expression/declaration/model identities, Boolean expected type and
discharged FR-015-equivalent definedness obligations under the exact native
profiles. An inline `holds(expr)` remains a leaf of its authoritative native
temporal subject; the bridge shall not turn it into or require a fabricated
standalone FR-023 `BoundClause`. A Boolean-looking text fragment, unlinked
syntax tree, display name or self-asserted `total` flag is invalid input, and
the bridge shall refuse it.

The canonical `PredicateRef` tuple shall contain exactly these members:

| Member | Exact value shape |
|---|---|
| `binding_requirements_ref` | lowercase 64-hex identity of the native subject's checked current-input/capture/environment requirements |
| `bridge_profile` | `quire.contract.native-predicate-tl-projection/v1` |
| `clause_ref` | object `{"clause":<string>,"requirement":{"package":<string>,"requirement":<string>,"revision":<positive-u64>}}` |
| `clause_span` | exact source-span object defined below |
| `declaration_closure_ref` | lowercase 64-hex declaration-closure digest |
| `declaration_ref` | lowercase 64-hex FR-016 declaration digest |
| `definedness_profile` | exact profile identity string |
| `evaluation_profile` | exact profile identity string |
| `expected_type` | exact string `boolean` |
| `expression_ref` | lowercase 64-hex FR-016 typed-expression digest |
| `expression_span` | exact source-span object defined below |
| `model_ref` | lowercase 64-hex model-closure digest |
| `native_definition` | object with exactly string members `digest`, `identity`, `revision`; `digest` is lowercase 64-hex |
| `source` | object with exactly `digest`, `document`, `revision`; `digest` is lowercase 64-hex and `revision` is a positive JSON integer |
| `source_leaf_ref` | lowercase 64-hex identity verified by the selected native checked-leaf contract |
| `source_subject_ref` | lowercase 64-hex identity of the authoritative parent native temporal or whole-clause subject |

Each exact source-span object shall contain `end` and `start`.

Each endpoint shall contain exactly `byte_offset`, `column`, `line`, and
`source`; `source` contains exactly `document` and `revision`.

Both endpoints shall name the same source identity in nondecreasing position
and byte order; byte offsets are unsigned u64, line/column are positive u32,
and revision is positive u64.

The bridge shall encode that object as compact UTF-8 JSON under profile
`quire.contract.native-predicate-ref/v1`: every object, including the referenced
public wire objects, uses Unicode-scalar member-name order; arrays preserve
declared semantic order; strings and integers use the FR-016 escaping and
minimal-integer rules; floating-point and null values are absent.

The bridge shall define `PredicateRef` as lowercase SHA-256 over the exact bytes
`quire-contract-ir`, one zero byte, the profile identity, one zero byte, and the
canonical tuple bytes.

A caller-supplied digest or display name shall not replace this computation.

When any tuple member changes, the bridge shall
produce a different reference even if display text and observed truth are
equal.

The selected predicate population shall contain 1 through 10,000 distinct
`PredicateRef` values, matching Contract IR's FR-019 semantic-collection
limit while remaining within the target catalog limit. The bridge shall sort
the population by `PredicateRef` bytes and assign zero-based contiguous
`PropositionId` and `SignalId` values in that order. The bridge shall name each
generated signal `quire-predicate/` followed by the lowercase 64-hex
`PredicateRef`. The bridge shall assign the Boolean domain to every generated
signal. The bridge shall bind each proposition to the signal with the same
assigned ordinal. It shall give the corresponding proposition-map entry the
same `quire-predicate/<PredicateRef>` name. When population input order changes
without changing its members, the bridge shall preserve every assigned
identity and output byte.

The bridge shall emit one `PropositionEntry` per correspondence, ordered by
strictly increasing `PropositionId`, with the same generated name as its signal.

The signal catalog shall contain exactly the corresponding ordered Boolean
signal declarations and proposition-to-signal bindings.

Contract IR shall check
the cross-document join: the proposition-map and catalog proposition
populations, IDs and generated names are equal, and each binding targets the
same-ordinal signal.

Each individual artifact shall pass its real public tl-syntax strict reader.

The bridge shall serialize both artifacts
as compact UTF-8 JSON using their public v1 field names, Unicode-scalar object-
key order and the FR-016 scalar rules; those exact bytes are the bytes hashed
and returned.

This projection set is complete only for the exact caller-selected predicate
population. Issue #64 shall supply a generated TL formula to the public
`bind_formula` operation.

Issue #64 shall refuse every missing proposition occurrence. Neither an
admitted set nor a changed `projection_set_ref` proves
that the caller selected every atom from a native temporal subject.

Each `ContractSelection` shall contain exactly the string
members `contract`, `package_version`, `repository`, `revision`, and
`schema_digest`. `revision` shall be an immutable repository commit OID and
`schema_digest` shall be lowercase 64-hex; neither a branch name nor a moving
version constraint is valid.

The canonical `projection_set_ref` tuple shall contain exactly
`bridge_profile`, `correspondences`, `native_contract`,
`proposition_map_ref`, `proposition_map_target`, `signal_catalog_ref`,
and `signal_catalog_target`. The artifact-reference members shall be lowercase
64-hex.
`native_contract` and both target members shall be `ContractSelection` objects.
`correspondences` shall be the ordered array of the exact five-member
`PredicateCorrespondence` objects defined above.

The bridge shall encode the projection-set tuple with the same canonical JSON
rules and digest construction as `PredicateRef`, replacing the profile with
`quire.contract.native-predicate-projection-set/v1`. A caller-supplied
identifier shall not replace this content computation.

The normative synthetic `PredicateRef` preimage below contains 1,467 JSON bytes
and has digest
`7665979d03dbfe1fa8a3b6243d1b7cefc805bc56199bf40aff0dddb618647db8`:

```json
{"binding_requirements_ref":"0000000000000000000000000000000000000000000000000000000000000000","bridge_profile":"quire.contract.native-predicate-tl-projection/v1","clause_ref":{"clause":"c","requirement":{"package":"p","requirement":"r","revision":1}},"clause_span":{"end":{"byte_offset":1,"column":2,"line":1,"source":{"document":"s","revision":1}},"start":{"byte_offset":0,"column":1,"line":1,"source":{"document":"s","revision":1}}},"declaration_closure_ref":"1111111111111111111111111111111111111111111111111111111111111111","declaration_ref":"2222222222222222222222222222222222222222222222222222222222222222","definedness_profile":"d/v1","evaluation_profile":"e/v1","expected_type":"boolean","expression_ref":"3333333333333333333333333333333333333333333333333333333333333333","expression_span":{"end":{"byte_offset":1,"column":2,"line":1,"source":{"document":"s","revision":1}},"start":{"byte_offset":0,"column":1,"line":1,"source":{"document":"s","revision":1}}},"model_ref":"4444444444444444444444444444444444444444444444444444444444444444","native_definition":{"digest":"5555555555555555555555555555555555555555555555555555555555555555","identity":"n/v1","revision":"1"},"source":{"digest":"6666666666666666666666666666666666666666666666666666666666666666","document":"s","revision":1},"source_leaf_ref":"7777777777777777777777777777777777777777777777777777777777777777","source_subject_ref":"8888888888888888888888888888888888888888888888888888888888888888"}
```

The normative synthetic projection-set preimage below contains 1,362 JSON
bytes and pins canonicalization only; it does not claim that the candidate
contracts are accepted. It has digest
`61a882228690bde7dadc335aa40e718941d837e9d6a408b1323f8ef449e15b21`:

```json
{"bridge_profile":"quire.contract.native-predicate-tl-projection/v1","correspondences":[{"predicate_ref":"7665979d03dbfe1fa8a3b6243d1b7cefc805bc56199bf40aff0dddb618647db8","proposition_id":0,"proposition_name":"quire-predicate/7665979d03dbfe1fa8a3b6243d1b7cefc805bc56199bf40aff0dddb618647db8","signal_id":0,"signal_name":"quire-predicate/7665979d03dbfe1fa8a3b6243d1b7cefc805bc56199bf40aff0dddb618647db8"}],"native_contract":{"contract":"n/v1","package_version":"1-draft.2","repository":"agent-ix/quire-specification","revision":"19bbf7a2036168a5f14dddd946f696707dc45fde","schema_digest":"1111111111111111111111111111111111111111111111111111111111111111"},"proposition_map_ref":"7777777777777777777777777777777777777777777777777777777777777777","proposition_map_target":{"contract":"tl-syntax.proposition-map/v1","package_version":"0.1.0","repository":"agent-ix/tl-syntax","revision":"7fb0fe32f6ba4ef2606d120ddbe5858965d4fb5a","schema_digest":"3333333333333333333333333333333333333333333333333333333333333333"},"signal_catalog_ref":"8888888888888888888888888888888888888888888888888888888888888888","signal_catalog_target":{"contract":"tl-syntax.signal-catalog/v1","package_version":"0.1.0","repository":"agent-ix/tl-syntax","revision":"7fb0fe32f6ba4ef2606d120ddbe5858965d4fb5a","schema_digest":"4444444444444444444444444444444444444444444444444444444444444444"}}
```

The normative synthetic result-projection preimage below contains 2,177 JSON
bytes and likewise pins canonicalization without claiming contract acceptance.
Its digest is
`d874712fa02dd23714b12a6c0de74d284033e46456c5264d350cc771ec9f00e2`:

```json
{"anchor_ref":"1111111111111111111111111111111111111111111111111111111111111111","boolean_value":true,"capture_ref":"2222222222222222222222222222222222222222222222222222222222222222","completeness":{"assertion_ref":"3333333333333333333333333333333333333333333333333333333333333333","boundary_ref":"4444444444444444444444444444444444444444444444444444444444444444","population_ref":"80021974474700b6540898d07c66c3bb2213904a277c03c4088951d4366d0998","state":"complete"},"deciding_fact_refs":["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],"format":"quire.contract.native-predicate-result-projection/v1","invocation_ref":"5555555555555555555555555555555555555555555555555555555555555555","model_ref":"6666666666666666666666666666666666666666666666666666666666666666","native_truth":"true","observation_facts":[{"fact_ref":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":"available"}],"observation_revision":1,"population_ref":"80021974474700b6540898d07c66c3bb2213904a277c03c4088951d4366d0998","predicate_execution":"completed","predicate_ref":"7665979d03dbfe1fa8a3b6243d1b7cefc805bc56199bf40aff0dddb618647db8","prior_result_ref":"","producer_profile":"producer/v1","snapshot_ref":"7777777777777777777777777777777777777777777777777777777777777777","source_result_contract":{"contract":"quire.protocol.result/v1-draft.1","package_version":"1-draft.1","repository":"agent-ix/quire-specification","revision":"19bbf7a2036168a5f14dddd946f696707dc45fde","schema_digest":"717ee2a2c3615026af3c1789dd1c28fc62d90189ce0343d2d561776b9543a6e3"},"source_result_digest":"9999999999999999999999999999999999999999999999999999999999999999","source_result_mapping":{"contract":"quire.contract.native-predicate-result-mapping/v1","package_version":"0.1.0","repository":"agent-ix/quire-contract-ir","revision":"69dc1a3e85adcfabaaed35875887ea5f508f58a0","schema_digest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},"source_result_ref":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","source_result_revision":1,"value_kind":"boolean","value_ref":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}
```

The closed projection-set cause dimensions, in required output order, shall be
`native-contract`, `target-contract`, `source`, `checked-leaf`, `model`,
`predicate`, `population`, `catalog`, and `proposition-map`.

The closed valuation cause dimensions, in required output order, shall be
`correspondence`, `availability-contract`, `source-result-contract`,
`source-result-mapping`, `producer`, `observation`, `predicate-execution`,
`anchor`, `capture`, `model`, `population`, `completeness`, `deciding-facts`,
`result`, and `supersession`.

Causes within one dimension shall sort by stable diagnostic code, rejected
identity, presence of `raw_discriminator`, and raw UTF-8 discriminator bytes.

A projection-set decision shall not contain a valuation-only cause.

A valuation decision shall not alter or reclassify its admitted projection set.

The bridge shall use the following closed STD-001 code allocation:

| Code | Dimension | Decision kind |
|---|---|---|
| `predicate_native_contract_unavailable` | `native-contract` | `unavailable` |
| `predicate_target_contract_unavailable` | `target-contract` | `unavailable` |
| `predicate_native_profile_unsupported` | `native-contract` | `unsupported` |
| `predicate_target_profile_unsupported` | `target-contract` | `unsupported` |
| `predicate_producer_profile_unsupported` | `producer` | `unsupported` |
| `predicate_evaluation_profile_unsupported` | `result` | `unsupported` |
| `predicate_construct_unsupported` | `predicate` | `unsupported` |
| `predicate_source_mismatch` | `source` | `refused` |
| `predicate_checked_leaf_mismatch` | `checked-leaf` | `refused` |
| `predicate_model_mismatch` | `model` | `refused` |
| `predicate_population_invalid` | `population` | `refused` |
| `predicate_native_contract_conflict` | `native-contract` | `conflict` |
| `predicate_target_contract_conflict` | `target-contract` | `conflict` |
| `predicate_identity_conflict` | `predicate` | `conflict` |
| `predicate_catalog_rejected` | `catalog` | `refused` |
| `predicate_map_rejected` | `proposition-map` | `refused` |
| `predicate_catalog_map_mismatch` | `proposition-map` | `refused` |
| `predicate_availability_contract_unsupported` | `availability-contract` | `unsupported` |
| `predicate_availability_contract_unavailable` | `availability-contract` | `unavailable` |
| `predicate_source_result_contract_unsupported` | `source-result-contract` | `unsupported` |
| `predicate_source_result_contract_unavailable` | `source-result-contract` | `unavailable` |
| `predicate_source_result_mapping_unsupported` | `source-result-mapping` | `unsupported` |
| `predicate_source_result_mapping_unavailable` | `source-result-mapping` | `unavailable` |
| `predicate_producer_unavailable` | `producer` | `unavailable` |
| `predicate_result_not_yet_observed` | `observation` | `incomplete` |
| `predicate_execution_unsupported` | `predicate-execution` | `unsupported` |
| `predicate_execution_refused` | `predicate-execution` | `refused` |
| `predicate_execution_failed` | `predicate-execution` | `failed` |
| `predicate_execution_incomplete` | `predicate-execution` | `incomplete` |
| `predicate_observation_mismatch` | `observation` | `refused` |
| `predicate_anchor_mismatch` | `anchor` | `refused` |
| `predicate_capture_mismatch` | `capture` | `refused` |
| `predicate_valuation_population_mismatch` | `population` | `refused` |
| `predicate_completeness_incomplete` | `deciding-facts` | `incomplete` |
| `predicate_completeness_conflict` | `deciding-facts` | `conflict` |
| `predicate_result_type_mismatch` | `result` | `refused` |
| `predicate_result_stale` | `result` | `refused` |
| `predicate_supersession_invalid` | `supersession` | `refused` |

`invalid_native_predicate_projection`, `predicate_projection_resource_exhausted`
and `predicate_valuation_resource_exhausted` shall remain operation diagnostics
rather than decision causes because no complete decision exists.

The projection-set decision shall classify a valid unknown native/target
profile or construct as `unsupported`, an absent accepted native/target
contract as `unavailable`, stale/mismatched semantic input or target-reader
rejection as `refused`, and
unequal content claiming one identity as `conflict`.

The valuation decision shall classify an unknown well-formed availability,
source-result or mapping contract, producer/evaluation profile, or source
execution `unsupported` as `unsupported`; an accepted but unreachable
availability, source-result or mapping contract or absent producer as
`unavailable`; stale/mismatched semantic input or refused source execution as
`refused`; a contradicted deciding fact as `conflict`; incomplete deciding
facts or resource-incomplete execution as `incomplete`; and failed source
execution as `failed`.

When causes from multiple decision kinds coexist, a projection-set decision shall select one kind by the precedence `conflict`, `refused`, `unsupported`,
`unavailable`, `admitted`.

When causes from multiple decision kinds coexist, a valuation decision shall
select one kind by the precedence `conflict`, `refused`, `unsupported`,
`failed`, `unavailable`, `incomplete`, `valued`.

Each decision shall retain every cause of equal or lower precedence.

The bridge shall refuse the entire projection set if the target public strict
reader rejects either generated artifact or any generated signal/binding is
non-Boolean or non-bijective.
It shall release no catalog, map, digest, or correspondence from that attempt.

Every `PredicateValuationDecision` shall bind the `PredicateRef`,
`projection_set_ref`, proposition/signal identities, availability contract,
source-result contract, source-result mapping and producer profile.

Every post-contract-admission decision shall additionally bind the verified
availability assertion, observation revision/state/classification,
anchor/snapshot/invocation, capture environment and model/population identities.

Every result-bearing post-contract-admission decision shall additionally bind
source-result and local-projection identities, predicate execution, native
truth, completeness and deciding-fact identities. It shall preserve predicate
execution, native truth and completeness as independent axes instead of folding
one axis into another.

The closed predicate-execution values shall be `completed`, `unsupported`,
`refused`, `failed`, and `resource-incomplete`.

The closed native-truth values shall be `true`, `false`, and `unavailable`.

Completeness shall retain the source producer's complete/incomplete/
contradicted record rather than become a Boolean or execution-state alias.

When a producer or evaluation profile identity is unknown but well formed, the
bridge shall preserve it as an `unsupported` raw discriminator.

For an available current result, the selected source-result contract's public
strict reader shall validate the exact source bytes and verify
`source_result_ref`, revision and digest.

The selected source-result mapping shall define a deterministic total
extraction from that validated source result to every field in the derived
projection record.

The bridge shall execute that selected mapping and compare every derived field
before computing `result_projection_ref`; caller-supplied projected axes cannot
replace or override the extracted values.

When structural validation and admitted-correspondence lookup succeed, the
bridge shall admit the three selected contracts and their public readers before
parsing an availability assertion or source result.

When any selection is unknown but well formed or any accepted selection/reader
is unreachable, the bridge shall return the exact contract-admission-failure
shape. The bridge shall select `unsupported` for an unknown-well-formed
selection and otherwise select `unavailable`. The decision shall retain the
independently applicable contract-specific causes in closed dimension order.
Caller-claimed assertion or result fields cannot alter this stage and shall not
be copied into its output.

When all three contracts are admitted, the bridge shall apply the following
total post-contract-admission valuation mapping after assertion, identity and
bound validation, from the first matching row downward:

| Input condition | Required decision |
|---|---|
| a deciding fact is `contradicted` | `conflict`, no value |
| stale/wrong semantic identity, impossible axis combination, or typed result not exact Boolean/equal truth | `refused`, no value |
| predicate execution `unsupported` or evaluation profile is unknown but well formed | `unsupported`, no value |
| predicate execution `refused` | `refused`, no value |
| predicate execution `failed` | `failed`, no value |
| `result_availability` is `producer-unavailable` or `contract-unavailable` under the verified authority assertion | `unavailable`, no value |
| `result_availability` is `not-yet-observed` under the verified authority assertion | `incomplete`, no value regardless of observation state; `closed-complete` means observation input is complete while the required assessment result is missing |
| predicate execution `resource-incomplete` | `incomplete`, no value |
| execution is `completed`, truth is `unavailable`, and at least one deciding fact is missing/incomplete | `incomplete`, no value |
| execution is `completed`, truth is `unavailable`, and every deciding fact is available | `refused`, no value; the combination has no incompleteness premise |
| execution is `completed`, truth is `true` or `false`, typed Boolean equals truth, and every deciding fact is available | `valued`, exact Boolean; retain every missing/incomplete/contradicted fact outside the deciding set as a completeness gap |

A valued decision shall require the final row and no higher-precedence cause.

The exact deciding-fact set shall contain at most 10,000 distinct canonical
fact identities sorted by identity bytes. The completeness record shall name
the complete subject/source population, inclusive observation boundary, and
authority assertion identity. A correction input shall supply the complete
immutable prior source result bytes and their derived projection, not only a
claimed identity.

The public strict reader for the selected source-result contract shall verify
the prior producer identity/digest.

The bridge shall recompute the
prior local projection reference, require a distinct successor revision, and
bind the successor to the same predicate subject. It shall refuse a missing,
self-referential, same-revision, wrong-subject or digest-inconsistent direct
predecessor. Global dangling-edge, branching and cycle detection remains with
the producer/result-store contract because this pure bridge has no ambient
result graph.

A corrected observation shall use a new observation/source-result revision.

The corrected observation shall name the verified exact prior source result as
superseded without rewriting its bytes.

Repeated equal inputs shall produce structurally equal decisions. A
changed source, model, catalog, predicate, anchor, capture, observation,
completeness, deciding-fact, producer or profile identity shall invalidate reuse
even when the Boolean value is unchanged.

The bridge shall operate only on supplied validated immutable values. It shall
not call the native or TL evaluator, read ambient runtime state, resolve names,
invoke a callback/plugin, access the network, or create a second expression
parser. Native evaluation remains owned by the selected native profile and its
producer; TL evaluation begins only after a `valued` decision is joined to the
exact derived proposition.

## Conformance Vectors

TC-038 shall include the following classes and require every listed mutation,
not one representative per row:

| Class | Positive control | Required discriminators |
|---|---|---|
| Boolean admission | completed checked literal/comparison/guarded optional predicate yields exact `true` or `false` | direct integer/rational/text/enum/record/option/collection value; nullable/error/unavailable truth; unsupported/refused/failed/resource-incomplete execution; impossible cross-axis combinations; self-asserted total flag |
| Definition identity | one accepted native checked-leaf/parent-subject/source/profile/model/typed-expression tuple | mutate every `PredicateRef` member independently; equal display text in another model, clause or inline `holds` leaf |
| Population mapping | reordered selected population produces identical contiguous IDs, names, catalog/map bytes and `projection_set_ref` | zero, duplicate-equal, duplicate-conflicting, smaller valid selection, added member, 10,000 and 10,001 predicates; unequal proposition/signal populations, IDs or names; non-Boolean signal or non-bijective binding; issue #64 formula occurrence missing from the selected set |
| Evaluation environment | exact anchor, invocation, immutable capture and observation produces a value | wrong or stale anchor/invocation/capture/model/population/observation identity; mutable capture substitution |
| Completeness/support | missing, incomplete or contradicted fact outside completed support preserves value plus typed gap | the same mutation inside support removes the value; pair caller-claimed `available` independently with unknown-well-formed and accepted-unreachable availability, source-result and mapping selections and require the base-only contract-admission shape; unavailable producer, unsupported/refused/failed/resource-incomplete execution and absent contract each retain their distinct non-value kind |
| Revision/conflict | corrected result supplies an exact immutable predecessor, uses a successor revision and retains prior bytes | absent or digest-mismatched predecessor, self-reference, same-revision predecessor, or result replay against a changed subject are refused; a contradicted deciding fact conflicts; no claim of ambient graph validation |
| Boundary purity | public strict TL readers accept the emitted catalog and map | private wire import, source parsing, evaluator callback, ambient lookup or unknown target/profile version |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-025-AC-1 | Every admitted predicate maps bijectively and deterministically to one Boolean signal and proposition in both target documents; input permutation preserves IDs, names, exact bytes and `projection_set_ref`, while omission/addition changes the selected-population identity. | Test (TC-038) |
| FR-025-AC-2 | `PredicateRef` binds the authority-verified checked leaf and parent subject plus exact source/profile, clause/span, binding-requirements and model/declaration/expression identities; every valuation binds its admitted correspondence and three supplied contract selections; every post-contract-admission decision additionally binds exact availability/observation/anchor/invocation/capture/model/population identities; every result-bearing decision also binds exact result/execution/truth/completeness/deciding-fact identities; each applicable one-axis mutation prevents reuse. | Test (TC-038) |
| FR-025-AC-3 | Only completed predicate execution with exact native truth and an equal Boolean typed value produces `valued(true)` or `valued(false)`; every non-Boolean kind, unavailable truth, non-completed execution, malformed or wrong-subject result exposes no Boolean and is never coerced. | Test (TC-038) |
| FR-025-AC-4 | Resource-incomplete execution, not-yet-observed results or missing deciding facts yield `incomplete`; an accepted-unreachable availability/source-result/mapping contract or absent producer yields `unavailable`; an unknown well-formed selection/profile/construct yields `unsupported`; failed execution yields `failed`; refused, invalid or stale bindings yield `refused`; a contradicted deciding fact yields `conflict`; every contract selection is mutated independently and no non-valued kind exposes a TL valuation. | Test (TC-038) |
| FR-025-AC-5 | A missing, incomplete or contradicted fact outside an exact deciding-fact set preserves the settled Boolean plus its typed completeness gap, while the same mutation inside the set removes the Boolean and produces `incomplete` or `conflict` according to status. | Test (TC-038) |
| FR-025-AC-6 | Valid predicate and deciding-fact populations at 10,000 and every constructible valid document at or below 67,108,864 bytes are admitted; 10,001 items or 67,108,865 input bytes, duplicate/conflicting identities, invalid supersession, non-Boolean signals, non-bijective bindings, allocation failure and target-reader rejection expose no partial catalog or map. | Test (TC-038) |
| FR-025-AC-7 | Repeated equal inputs are structurally equal; a correction requires an exact immutable direct predecessor plus a distinct successor observation/result revision; self-reference, wrong-subject or same-revision predecessor is `refused`, a contradicted deciding fact is `conflict`, and every prior byte remains unchanged. | Test (TC-038) |
| FR-025-AC-8 | Real public tl-syntax strict readers accept both positive documents, and the Contract IR join rejects population, ID, name, domain or binding disagreement without private wire imports, evaluator invocation or an authored TL/FRETish predicate surface; issue #64 separately proves formula-occurrence completeness. | Test (TC-038) |

## Dependencies

- [FR-012](./FR-012-anchors-clauses-dependencies.md),
  [FR-014](./FR-014-expression-semantics.md),
  [FR-015](./FR-015-definedness.md),
  [FR-016](./FR-016-canonicalization-digests.md) supply the accepted local
  identity, canonical-byte rules, typing and definedness foundations. FR-023
  remains the whole-clause executable-projection contract; it is not an
  authority for an inline native temporal predicate leaf.
- `agent-ix/quire-specification#15` must merge exact reviewed FR-048 native
  temporal binding and FR-095 correspondence definitions plus a public
  source-bound checked-leaf contract and strict reader. That contract must
  preserve each inline `holds(expr)` leaf under its parent native subject and
  expose every checked-leaf field consumed above. The current stacked candidate
  describes the leaf semantics but does not yet publish this reader contract;
  it is not a released dependency and cannot satisfy implementation admission.
- An observation authority must publish an accepted result-availability
  assertion contract and strict reader, and the native result owner must publish
  an accepted source-result contract plus deterministic projection mapping.
  No current released dependency satisfies either interface; implementation is
  blocked until their exact `ContractSelection` values and sibling-byte inputs
  are reviewed and selected. This requirement does not assign those producer
  authorities to Contract IR.
- `agent-ix/tl-syntax#15` supplies the target signal-catalog/source-context
  contract. Because tl-syntax is currently unpublished, an accepted target
  release means an immutable reviewed repository commit plus the exact public
  schema digests and Cargo package version; a branch name or `current` alias is
  insufficient. In particular, target admission is blocked until
  `tl-syntax.signal-catalog/v1` has a canonical published schema artifact whose
  exact bytes ground `ContractSelection.schema_digest`; an implementation-only
  Rust/Serde shape is not that artifact. Issue #64 consumes this requirement, binds a generated formula
  to the selected proposition population, and refuses any missing occurrence.
  Issue #57 remains output-only and is not a prerequisite.
- This requirement specifies the pure bridge boundary and conformance vectors.
  It does not prove that the native producer, temporal bridge, TL evaluator or
  downstream monitor is implemented or qualified. Any first-party reader,
  mapping or bridge implementation added under this requirement shall be Rust;
  no Java, Node, Electron or new Python semantic path is admitted.
