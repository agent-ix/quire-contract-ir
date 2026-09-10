---
id: STD-001
title: "Contract IR v0.1 diagnostic code registry"
type: Standard
code: contract-ir-diagnostics-v0.1
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: references
---
# STD-001: Contract IR v0.1 diagnostic code registry

## Description

This registry owns the stable machine-readable diagnostic codes introduced by
issues #6 through #10, #63 and #64. Implementations may add human context but shall not parse
or synthesize codes from messages. Codes are lowercase ASCII snake case.

## Issue 6 Codes

| Code | Condition | Required location |
|---|---|---|
| `invalid_package_namespace` | Empty or malformed package namespace | package identity path |
| `invalid_wire_format` | JSON syntax or closed wire shape prevents decoding, or the 576-level wire-nesting limit is exceeded | document path; `document.nesting` identifies pre-decode depth refusal |
| `invalid_schema_version` | Zero schema major; schema minor zero is valid | schema-version path |
| `invalid_identifier` | Empty or malformed source-document, requirement, clause, anchor, or dependency-path-segment identifier | offending identity path |
| `invalid_requirement_revision` | Zero or non-increasing requirement revision | requirement revision path |
| `invalid_source_revision` | Zero source-document revision | source revision path |
| `duplicate_requirement` | Two current requirements share one ID | later requirement path and earlier related identity |
| `duplicate_clause` | Two clauses in one requirement share one ID | later clause span and earlier related identity |
| `cross_package_reference` | A reference names a different package | reference span/path |
| `invalid_source_span` | Source endpoints are zero-based-invalid, reversed, or name a different source | source span |
| `floating_executable_clause` | Executable clause has no anchor | clause span |
| `informational_clause_anchored` | Informational clause has an anchor | clause span |
| `incompatible_clause_anchor` | Clause and anchor kinds violate the closed compatibility table | clause span |
| `malformed_reference` | Reference identity is structurally malformed | reference span/path |
| `stale_requirement_revision` | Requirement exists but the exact revision differs | reference span/path and current related identity |
| `orphaned_requirement_reference` | Referenced requirement ID is absent | reference span/path |
| `orphaned_clause_reference` | Requirement revision resolves but its clause ID is absent | reference span/path |

## Issue 8 Codes

| Code | Condition | Required location |
|---|---|---|
| `duplicate_type_declaration` | Two enum/record declarations share one type name | later declaration span |
| `duplicate_value_declaration` | Two input/state declarations share one value name | later declaration span |
| `duplicate_function_declaration` | Two pure functions share one name | later declaration span |
| `duplicate_field` | Two record fields share one name | later field span |
| `duplicate_variant` | Two enum variants share one name | later variant span |
| `duplicate_parameter` | Two function parameters share one name | later parameter span |
| `empty_enum` | Enum declaration has no variants | enum declaration span |
| `invalid_numeric_bounds` | Integer bounds/domain, rational denominator, or collection maximum exceeds its closed numeric range | type/literal span or path |
| `text_bound_exceeded` | Text literal contains more than 1048576 Unicode scalar values | text literal span |
| `unbounded_collection` | Collection maximum is zero or absent | type span/path |
| `collection_bound_exceeded` | Collection literal contains more items than its declared maximum | collection literal span |
| `orphaned_type_reference` | Named enum/record type does not resolve | type span/path |
| `recursive_type` | Record containment graph has a direct or indirect cycle | participating field span |
| `orphaned_value_reference` | Input/state reference does not resolve | expression span |
| `orphaned_function_reference` | Pure-function reference does not resolve | call span |
| `invalid_state_observation` | Input/state observation is not permitted by the FR-014 execution-point table | reference span |
| `invalid_scope` | Local name is absent, duplicated, or escapes its quantifier | local/quantifier span |
| `arity_mismatch` | Pure-function argument count differs from its declaration | call span |
| `ill_typed_expression` | Operand, access, argument, field, variant, or quantifier-domain type is invalid | narrowest expression span |
| `result_type_mismatch` | Checked expression type differs from the expected type | root expression span |
| `non_boolean_clause_root` | Executable clause body does not have Boolean type | clause-root span |
| `potentially_undefined` | A partial-operation obligation is not statically discharged; diagnostic includes mandatory `obligation_kind` (`option_presence`, `non_zero_divisor`, `index_in_bounds`, or `checked_range`) | partial-operation span |
| `expression_too_large` | Expression exceeds 10000 nodes or depth 256 | first node crossing the limit |

## Issue 9 Codes

| Code | Condition | Required location |
|---|---|---|
| `unsupported_schema_version` | Wire preflight reads a nonzero schema major other than 1 | `schema_version.major` path; no semantic span |
| `unregistered_migration` | Major 1 minor is not 0/1, or a requested migration edge is not the registered 1.0-to-1.1 edge | `schema_version` or migration request path |
| `canonicalization_resource_exhausted` | Canonical byte allocation cannot be reserved without exceeding host resources | canonicalized object path; source span when the object has one |
| `duplicate_artifact_trace` | A later artifact trace repeats an artifact ID in one classification input | later trace span |
| `stale_trace_digest` | A deep trace's requirement digest differs from the resolved current requirement digest | digest-token span |

## Issue 10 Codes

| Code | Condition | Required location |
|---|---|---|
| `semantic_input_too_large` | A complete operation exceeds 25000 semantic nodes, recursive semantic depth 256, or 10000 entries in any semantic collection | first node, depth, or collection path crossing the limit; source span when present |

## Issue 63 Codes

These codes are also the closed `code` vocabulary for FR-025 decision causes.
Each allowed `(code, dimension, kind)` tuple is fixed by FR-025; a code cannot
be used outside those exact allocations.

| Code | Condition | Required location |
|---|---|---|
| `invalid_native_predicate_projection` | A v1 request/result/decision record violates its strict tagged shape, UTF-8, enum, byte, depth, count, or string bound | narrowest rejected public field path |
| `predicate_native_contract_unavailable` | The exact accepted native contract is absent | native contract selection |
| `predicate_target_contract_unavailable` | An exact required tl-syntax contract is absent | target contract selection |
| `predicate_native_profile_unsupported` | A well-formed native profile is not in the admitted v1 domain | native profile field and raw discriminator |
| `predicate_target_profile_unsupported` | A well-formed target profile is not in the admitted v1 domain | target profile field and raw discriminator |
| `predicate_producer_profile_unsupported` | A well-formed producer profile is not in the admitted v1 domain | producer profile field and raw discriminator |
| `predicate_evaluation_profile_unsupported` | A well-formed evaluation profile is not in the admitted v1 domain | evaluation profile field and raw discriminator |
| `predicate_construct_unsupported` | A well-formed checked predicate construct is outside the admitted bridge domain | predicate reference |
| `predicate_source_mismatch` | Source identity/revision/digest/span does not match the checked predicate | source or span field |
| `predicate_checked_leaf_mismatch` | Checked-leaf/parent-subject/clause/binding/declaration/expression identity is stale or unequal | first unequal checked-leaf field |
| `predicate_model_mismatch` | Model/declaration-closure identity is stale or unequal | model field |
| `predicate_population_invalid` | Selected predicate population is empty, duplicate, non-distinct, internally inconsistent with its declared members, or over the Contract IR limit; it does not mean formula-atom incompleteness | predicate population |
| `predicate_native_contract_conflict` | Unequal native contract definitions claim one current contract identity | native contract identity and related candidate |
| `predicate_target_contract_conflict` | Unequal target definitions claim one current contract identity | target contract identity and related candidate |
| `predicate_identity_conflict` | Unequal predicate content claims one `PredicateRef` | conflicting predicate identity and related candidate |
| `predicate_catalog_rejected` | The real signal-catalog strict reader rejects the generated document | catalog field/path |
| `predicate_map_rejected` | The real proposition-map strict reader rejects the generated document | proposition-map field/path |
| `predicate_catalog_map_mismatch` | Catalog/map population, proposition ID/name, signal domain, ordinal, or binding join differs | first unequal joined field |
| `predicate_projection_resource_exhausted` | Projection allocation/reservation fails without a partial artifact | projection resource path |
| `predicate_availability_contract_unsupported` | A well-formed availability contract selection is not in the admitted v1 domain | availability-contract field and raw discriminator |
| `predicate_availability_contract_unavailable` | The selected accepted availability contract or its public strict reader cannot be reached | availability-contract field |
| `predicate_source_result_contract_unsupported` | A well-formed source-result contract selection is not in the admitted v1 domain | source-result-contract field and raw discriminator |
| `predicate_source_result_contract_unavailable` | The selected accepted source-result contract or its public strict reader cannot be reached | source-result-contract field |
| `predicate_source_result_mapping_unsupported` | A well-formed source-result mapping selection is not in the admitted v1 domain | source-result-mapping field and raw discriminator |
| `predicate_source_result_mapping_unavailable` | The selected accepted source-result mapping or its public strict reader cannot be reached | source-result-mapping field |
| `predicate_producer_unavailable` | The verified availability assertion classifies the exact producer or required result contract as unavailable | producer/result-availability field |
| `predicate_result_not_yet_observed` | Authority-verified availability assertion says the required producer result is not yet observed, regardless of observation state | result-availability field |
| `predicate_execution_unsupported` | Source predicate execution is `unsupported` | predicate-execution field |
| `predicate_execution_refused` | Source predicate execution is `refused` | predicate-execution field |
| `predicate_execution_failed` | Source predicate execution is `failed` | predicate-execution field |
| `predicate_execution_incomplete` | Source predicate execution is `resource-incomplete` | predicate-execution field |
| `predicate_observation_mismatch` | Observation/result identity or revision is stale or unequal | observation field |
| `predicate_anchor_mismatch` | Anchor/snapshot/invocation identity is stale or unequal | first unequal anchor field |
| `predicate_capture_mismatch` | Capture-environment identity is stale or unequal | capture field |
| `predicate_valuation_population_mismatch` | Observation population identity does not equal the canonical fact population | population field |
| `predicate_completeness_incomplete` | A deciding fact is missing/incomplete or native truth is unavailable on that premise | completeness or deciding-fact field |
| `predicate_completeness_conflict` | A deciding fact is contradicted; an outside-set contradiction remains a typed gap | contradicted deciding-fact field |
| `predicate_result_type_mismatch` | Value kind/payload is non-Boolean or disagrees with native truth | value-kind/value field |
| `predicate_result_stale` | Source result/profile/subject identity is stale or unequal | first stale result field |
| `predicate_supersession_invalid` | Direct predecessor is absent, self-referential, same-revision, wrong-subject, or digest-inconsistent | prior-result field |
| `predicate_valuation_resource_exhausted` | Valuation allocation/reservation fails without a partial decision | valuation resource path |

## Issue 64 Codes

These codes are the closed `code` vocabulary for FR-026 projection and result
join causes. FR-026 fixes every allowed `(code, dimension, kind)` tuple; a code
cannot be used outside those allocations.

| Code | Condition | Required location |
|---|---|---|
| `invalid_native_temporal_bridge` | A selected input strict reader rejects a v1 input, or a projection/result request or decision violates its strict tagged shape, UTF-8, enum, byte, depth, count or string bound | narrowest rejected public field path |
| `temporal_native_contract_unsupported` | A well-formed native temporal contract selection is outside the admitted v1 domain | native-contract field and raw discriminator |
| `temporal_native_contract_unavailable` | The selected accepted native temporal contract or public strict reader cannot be reached | native-contract field |
| `temporal_native_contract_conflict` | Unequal native temporal contract definitions claim one current identity | native-contract identity and related candidate |
| `temporal_subject_mismatch` | Native subject identity, revision, digest, model, source span or tree identity is stale or unequal | first unequal native-subject field |
| `temporal_predicate_projection_incomplete` | A required checked leaf has no admitted FR-025 correspondence yet | exact native `holds` leaf |
| `temporal_predicate_projection_contract_unsupported` | A well-formed FR-025 projection contract selection is outside the admitted v1 domain | predicate-projection-contract field and raw discriminator |
| `temporal_predicate_projection_contract_unavailable` | The selected accepted FR-025 projection contract or public strict reader cannot be reached | predicate-projection-contract field |
| `temporal_predicate_projection_contract_conflict` | Unequal FR-025 projection contract definitions claim one selected identity | predicate-projection-contract identity and related candidate |
| `temporal_predicate_projection_unavailable` | The exact admitted FR-025 projection or required valuation decision cannot be reached | predicate-projection field |
| `temporal_predicate_projection_unsupported` | A required FR-025 valuation decision reports unsupported | predicate-projection field |
| `temporal_predicate_projection_failed` | A required FR-025 valuation decision reports failed execution | predicate-projection field |
| `temporal_predicate_projection_refused` | A required FR-025 valuation decision reports refused | predicate-projection field |
| `temporal_predicate_projection_conflict` | A required FR-025 valuation decision reports conflict | predicate-projection field |
| `temporal_predicate_projection_mismatch` | Predicate/proposition population, identity or parent-subject binding is stale or unequal | first unequal predicate-projection field |
| `temporal_profile_unsupported` | A well-formed native temporal profile is outside the closed v1 support table | temporal-profile field and raw discriminator |
| `temporal_operator_unsupported` | A well-formed native temporal operator is outside the closed future-core mapping | operator span and raw discriminator |
| `temporal_interval_invalid` | A temporal interval is negative, fractional, inverted, unbounded or exceeds `u32::MAX` | interval span |
| `temporal_clock_incomplete` | A required fixed-sample clock component or sample is not yet available | clock/sample field |
| `temporal_clock_contract_unsupported` | A well-formed clock contract selection is outside the admitted v1 domain | clock-contract field and raw discriminator |
| `temporal_clock_contract_unavailable` | The selected accepted clock contract or public strict reader cannot be reached | clock-contract field |
| `temporal_clock_contract_conflict` | Unequal clock contract definitions claim one selected identity | clock-contract identity and related candidate |
| `temporal_clock_unavailable` | The authority-verified clock binding cannot be reached | clock field |
| `temporal_clock_mismatch` | Clock identity, epoch, period, unit or sample-position mapping is stale or unequal | first unequal clock field |
| `temporal_observation_incomplete` | The observation is closed-incomplete or required observation/history is not yet complete | observation field |
| `temporal_observation_contract_unsupported` | A well-formed observation contract selection is outside the admitted v1 domain | observation-contract field and raw discriminator |
| `temporal_observation_contract_unavailable` | The selected accepted observation contract or public strict reader cannot be reached | observation-contract field |
| `temporal_observation_contract_conflict` | Unequal observation contract definitions claim one selected identity | observation-contract identity and related candidate |
| `temporal_observation_mismatch` | Observation identity, revision, state or immutable content is stale or unequal | first unequal observation field |
| `temporal_capture_contract_unsupported` | A well-formed capture contract selection is outside the admitted v1 domain | capture-contract field and raw discriminator |
| `temporal_capture_contract_unavailable` | The selected accepted capture contract or public strict reader cannot be reached | capture-contract field |
| `temporal_capture_contract_conflict` | Unequal capture contract definitions claim one selected identity | capture-contract identity and related candidate |
| `temporal_activation_incomplete` | The temporal activation state is not yet known | activation field |
| `temporal_activation_inactive` | A caller requests evaluation for an authoritatively inactive temporal scope | activation field |
| `temporal_activation_mismatch` | Activation identity is stale or unequal | activation field |
| `temporal_capture_incomplete` | A required immutable capture is not yet available | capture field |
| `temporal_capture_mismatch` | Capture binding is stale or foreign by identity, revision or expected digest; unequal content claiming the same identity is instead `temporal_identity_conflict` | first stale or foreign capture field |
| `temporal_formula_contract_unsupported` | A well-formed TL formula contract selection is outside the admitted v1 domain | formula-contract field and raw discriminator |
| `temporal_formula_contract_unavailable` | The selected accepted TL formula contract or public strict reader cannot be reached | formula-contract field |
| `temporal_formula_contract_conflict` | Unequal TL formula contract definitions claim one selected identity | formula-contract identity and related candidate |
| `temporal_semantic_contract_unsupported` | A well-formed TL semantic contract selection is outside the closed v1 support table | semantic-contract field and raw discriminator |
| `temporal_semantic_contract_unavailable` | The selected accepted TL semantic contract or public strict reader cannot be reached | semantic-contract field |
| `temporal_semantic_contract_conflict` | Unequal TL semantic contract definitions claim one selected identity | semantic-contract identity and related candidate |
| `temporal_evaluator_contract_unsupported` | A well-formed TL evaluator contract selection is outside the admitted v1 domain | evaluator-contract field and raw discriminator |
| `temporal_evaluator_contract_unavailable` | The selected accepted TL evaluator contract or public strict reader cannot be reached | evaluator-contract field |
| `temporal_evaluator_contract_conflict` | Unequal TL evaluator contract definitions claim one selected identity | evaluator-contract identity and related candidate |
| `temporal_trace_contract_unsupported` | A well-formed TL trace contract selection is outside the admitted v1 domain | trace-contract field and raw discriminator |
| `temporal_trace_contract_unavailable` | The selected accepted TL trace contract or public strict reader cannot be reached | trace-contract field |
| `temporal_trace_contract_conflict` | Unequal TL trace contract definitions claim one selected identity | trace-contract identity and related candidate |
| `temporal_request_contract_unsupported` | A well-formed TL evaluator-request contract selection is outside the admitted v1 domain | request-contract field and raw discriminator |
| `temporal_request_contract_unavailable` | The selected accepted TL evaluator-request contract or public strict reader cannot be reached | request-contract field |
| `temporal_request_contract_conflict` | Unequal TL evaluator-request contract definitions claim one selected identity | request-contract identity and related candidate |
| `temporal_formula_rejected` | The real selected TL formula strict reader rejects the generated formula | narrowest rejected formula field |
| `temporal_trace_rejected` | The real selected TL trace strict reader rejects the generated complete valuation trace | narrowest rejected trace field |
| `temporal_request_rejected` | The real selected TL request strict reader rejects the generated formula/trace request | narrowest rejected request field |
| `temporal_identity_conflict` | Unequal subject, clock, observation, capture, formula, trace or request content claims one identity | conflicting identity and related candidate |
| `temporal_availability_contract_unsupported` | A well-formed result-availability contract selection is outside the admitted v1 domain | availability-contract field and raw discriminator |
| `temporal_availability_contract_unavailable` | The selected accepted availability contract or public strict reader cannot be reached | availability-contract field |
| `temporal_availability_contract_conflict` | Unequal result-availability contract definitions claim one selected identity | availability-contract identity and related candidate |
| `temporal_availability_assertion_mismatch` | The availability assertion identity, revision, digest, observation binding or classifications are stale or unequal | first unequal availability field |
| `temporal_availability_assertion_conflict` | Unequal availability assertion content claims one authority-owned identity | conflicting availability identity and related candidate |
| `temporal_native_result_contract_unsupported` | A well-formed native-result contract selection is outside the admitted v1 domain | native-result-contract field and raw discriminator |
| `temporal_native_result_contract_unavailable` | The selected accepted native-result contract or public strict reader cannot be reached | native-result-contract field |
| `temporal_native_result_contract_conflict` | Unequal native-result contract definitions claim one selected identity | native-result-contract identity and related candidate |
| `temporal_tl_result_contract_unsupported` | A well-formed TL-result contract selection is outside the admitted v1 domain | TL-result-contract field and raw discriminator |
| `temporal_tl_result_contract_unavailable` | The selected accepted TL-result contract or public strict reader cannot be reached | TL-result-contract field |
| `temporal_tl_result_contract_conflict` | Unequal TL-result contract definitions claim one selected identity | TL-result-contract identity and related candidate |
| `temporal_native_result_unavailable` | The verified availability assertion classifies the native producer or result as unavailable | native result availability field |
| `temporal_tl_result_unavailable` | The verified availability assertion classifies the TL producer or result as unavailable | TL result availability field |
| `temporal_native_result_incomplete` | Native assessment execution is resource-incomplete or native truth is unavailable because exact decision support is incomplete | native execution/truth/support field |
| `temporal_tl_result_incomplete` | TL assessment execution is resource-incomplete or TL truth is unavailable because exact decision support is incomplete | TL execution/truth/support field |
| `temporal_native_result_unsupported` | The available native result reports unsupported assessment execution | native assessment-execution field |
| `temporal_tl_result_unsupported` | The available TL result reports unsupported assessment execution | TL assessment-execution field |
| `temporal_native_result_failed` | The available native result reports failed assessment execution | native assessment-execution field |
| `temporal_tl_result_failed` | The available TL result reports failed assessment execution | TL assessment-execution field |
| `temporal_native_result_refused` | The available native result reports refused assessment execution | native assessment-execution field |
| `temporal_tl_result_refused` | The available TL result reports refused assessment execution | TL assessment-execution field |
| `temporal_native_result_contradicted` | The available native result reports contradicted completeness or immutable progress/closure premises | native completeness/progress/closure field |
| `temporal_tl_result_contradicted` | The available TL result reports contradicted completeness or immutable progress/closure premises | TL completeness/progress/closure field |
| `temporal_result_binding_mismatch` | An available result claims a stale or unequal correspondence, formula, trace, request, observation or semantic profile | first unequal binding field |
| `temporal_result_identity_conflict` | Unequal available result content claims one producer-owned result identity | conflicting result identity and related candidate |
| `temporal_progress_contract_unsupported` | An embedded progress contract selection is outside the admitted v1 domain | progress-contract field and raw discriminator |
| `temporal_progress_contract_unavailable` | An embedded accepted progress contract or public strict reader cannot be reached | progress-contract field |
| `temporal_progress_contract_conflict` | Unequal progress contract definitions claim one selected identity | progress-contract identity and related candidate |
| `temporal_progress_mismatch` | Available native and TL progress values cannot agree under the exact join table | progress fields |
| `temporal_closure_mismatch` | Result progress is inconsistent with the immutable observation state | observation-state/progress field |
| `temporal_result_closure_disagreement` | Otherwise valid native and TL views disagree on decision-scope or surrounding-execution closure | first unequal closure field |
| `temporal_truth_mismatch` | Otherwise valid native and TL views disagree on final truth or final-versus-pending state | truth fields |
| `temporal_settlement_mismatch` | Otherwise valid native and TL views disagree on settlement basis | settlement-basis fields |
| `temporal_support_mismatch` | Otherwise valid native and TL views disagree on exact decision support | decision-support fields |
| `temporal_completeness_contract_unsupported` | An embedded completeness contract selection is outside the admitted v1 domain | completeness-contract field and raw discriminator |
| `temporal_completeness_contract_unavailable` | An embedded accepted completeness contract or public strict reader cannot be reached | completeness-contract field |
| `temporal_completeness_contract_conflict` | Unequal completeness contract definitions claim one selected identity | completeness-contract identity and related candidate |
| `temporal_completeness_mismatch` | Otherwise valid native and TL views disagree on completeness contract, identity, digest or state | completeness fields |
| `temporal_supersession_invalid` | Correction relation, direct predecessor, contradicted premise or corrected input is absent, self-referential, same-revision, wrong-subject, wrong-producer or digest-inconsistent | narrowest relation/predecessor/premise/input field |
| `temporal_projection_resource_exhausted` | Formula, valuation, trace, request or correspondence allocation fails without a partial artifact | projection resource path |
| `temporal_result_join_resource_exhausted` | Result-join allocation fails without a partial decision | result-join resource path |

## Application Guidance

Public diagnostics contain a code, closed severity `error`, message, semantic
path, optional source span, related identities, and optional
`obligation_kind`. The obligation field is present if and only if the code is
`potentially_undefined` and uses the four-value closed enum. New failure classes
require a registry row, an owning requirement criterion, and positive or
negative test evidence before implementation claims coverage.

Diagnostic precedence is identity grammar, then source/span structure, then
package ownership, then exact-revision resolution, then target existence, and
finally clause/anchor compatibility. One condition emits its highest-precedence
primary code; additional context may appear only as related identities or
secondary diagnostics. Thus an empty clause ID is `invalid_identifier`, not
`malformed_reference`, and an empty package namespace is
`invalid_package_namespace`, not `cross_package_reference`.

Issue #8 precedence is declaration/identifier grammar and numeric/collection
bounds, then duplicates, named-type resolution, containment cycles, local/value/
function name resolution, call arity, operand/access typing, expected-result
typing, and definedness last. At an executable clause root,
`non_boolean_clause_root` takes precedence over `result_type_mismatch`. Local
lookup precedes value lookup; an absent syntactic local is `invalid_scope`, an
absent declared value is `orphaned_value_reference`, and an absent call target
is `orphaned_function_reference`. Arity precedes argument typing. An ill-typed
partial node is `ill_typed_expression`, never `potentially_undefined`.

Expression diagnostics are emitted in authored pre-order: one primary
diagnostic per node, siblings in stored order. A failed child suppresses its
parent's typing/definedness diagnostic, while independent siblings continue.
Within one node the issue #8 precedence above selects the primary code. State
observation policy follows resolved-value lookup and precedes operand typing.
Declaration-environment diagnostics precede expression diagnostics and follow
stored declaration/field/variant/parameter order.

Issue #9 wire precedence is JSON/top-level structure, schema-version numeric
grammar, unsupported major, unregistered minor/migration edge, then semantic
package interpretation. Canonicalization accepts validated values only and
performs no diagnostic recovery; resource exhaustion produces no partial bytes
or digest. Coverage precedence per trace is duplicate artifact ID,
cross-package target, missing requirement, stale revision, then deep-digest
mismatch. Coverage diagnostics retain authored trace order even though report
rows sort structurally.

## Dependencies

- **Upstream**: PGM-01 evidence and human-decision boundaries.
- **Downstream**: FR-013 through FR-019, FR-023, FR-025, and FR-026 extend or consume this
  semantic registry without renaming issue #6 codes. FR-020 defines separate
  runner operational codes that are neither `DiagnosticCode` values nor
  semantic diagnostic shapes.
