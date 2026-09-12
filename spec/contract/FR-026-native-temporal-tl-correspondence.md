---
id: FR-026
title: "Bind native temporal subjects to exact TL correspondence"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-048
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-090
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-091
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-092
    type: references
  - target: ix://agent-ix/quire-specification/FR-093
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-094
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-095
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-061
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-110
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-112
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-113
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-007
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/issues/52
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/57
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/64
    type: references
  - target: ix://agent-ix/quire-contract-ir/ADR-0053
    type: references
---
# FR-026: Bind native temporal subjects to exact TL correspondence

## Description

When an authority-verified native Quire temporal subject is selected for TL
evaluation, Contract IR shall either construct one exact internal TL formula
with its valuation set, trace, evaluator request and correspondence, or return
a typed fail-closed decision without any usable evaluator artifact.

Native Quire remains the sole editable formal-clause language. TL documents are
derived internal representation and evaluator inputs. This bridge parses no TL
text, defines no second temporal language, and does not make FRETish a source;
issue #57 remains an output-only mapping.

The bridge profile is
`quire.contract.native-temporal-tl-correspondence/v1`.

## Inputs

- An exact `ContractSelection` for an accepted native source-bound temporal
  subject contract and its public strict reader, plus the exact subject bytes,
  identity, revision and digest. The verified subject exposes a bounded typed
  post-parse temporal tree, source/clause/expression spans, temporal profile,
  clock requirements, activation identity, capture requirements and model
  identity. Contract IR does not consume an FR-023 common-expression
  `BoundClause` as a substitute for that temporal tree.
- The exact admitted FR-025 projection decision bytes, its
  `projection_set_ref`, target signal-catalog/proposition-map selections, and
  one `PredicateRef`/`PropositionId` correspondence for every inline
  `holds(expr)` occurrence in the temporal tree. For every admitted observation
  position and every proposition required by the formula, the exact admitted
  FR-025 valued-decision bytes and `result_projection_ref`; any non-valued
  decision remains a typed projection gap rather than an omitted false value.
- Exact `ContractSelection` values for `tl-syntax.formula/v1`, the selected TL
  semantic profile, `tl-mltl.trace/v1`, `tl-mltl.command/v1`, and the public
  tl-mltl evaluator/report contract.
- Exact clock, observation and capture `ContractSelection` values;
  authority-verified clock-binding, observation and capture bytes; clock
  identity, revision and digest; observation identity, revision and
  digest; and an ordered observation-position population whose public reader
  authenticates each unique `position_ref` and its exact zero-based position,
  `anchor_ref`, `snapshot_ref`, and `invocation_ref`;
  activation identity and state `active`, `inactive` or `unknown`; immutable
  capture-environment identity, revision and digest; and observation state
  `open`, `closed-complete` or `closed-incomplete`. `observation_state` is the
  surrounding-execution closure authenticated by the selected observation
  reader: complete execution versus open prefix for the whole observed run.
  It is not a decision-scope closure.
- For result joining, an exact result-availability contract selection and
  authority-verified assertion bytes, identity, revision and digest; exact
  native-result and TL-result contract selections; and, for each result that
  assertion classifies `available`, immutable result bytes, producer-owned
  identity, revision and digest plus the exact trace identity and
  correspondence reference it claims; and, when either result names a direct
  predecessor, that predecessor's immutable bytes, identity, revision and
  digest. The exact authority-owned progress and completeness assertion bytes
  named by each available result, together with their embedded immutable
  contract selections, are also supplied and validated through those selected
  public readers. Each available result's selected public
  view supplies assessment execution, independent decision-scope and
  surrounding-execution progress/closure, truth, settlement basis, exact
  decision support, completeness and direct predecessor identity; none is
  inferred from the Boolean value or file position.

Each `ContractSelection` uses the exact five-string FR-025 shape `contract`,
`package_version`, `repository`, immutable `revision`, and lowercase 64-hex
`schema_digest`. A branch, moving version range, copied schema or private Rust
wire type is not an accepted selection.

Each supplied or generated document contains at most 67,108,864 bytes and
semantic depth 256. Each native temporal tree, node-correspondence array,
observation-position array, valuation-set array and decision-support array
contains at most 10,000 entries. Every interval satisfies
`0 <= lower <= upper <= 4294967295`; the upper bound is the TL `u32::MAX`, not
an unbounded sentinel.

## Outputs

- A `TemporalProjectionDecision` with kind `admitted`, `incomplete`,
  `unavailable`, `unsupported`, `failed`, `refused` or `conflict`.
- An admitted decision contains the exact derived formula document and bytes,
  `formula_ref`, ordered node correspondences, complete valuation-set identity,
  exact TL trace and evaluator-request documents and identities,
  `correspondence_ref`, and no causes. Every other kind contains none of those
  admitted-only fields or a usable evaluator request and has a nonempty ordered
  cause set.
- A `TemporalResultJoinDecision` for an admitted correspondence with kind
  `agreement`, `incomplete`, `unavailable`, `unsupported`, `failed`, `refused`
  or `conflict`.
  Only an `agreement` with comparison `equal-final` contains one Boolean value.
  An `equal-pending` agreement contains no Boolean value and is not a final
  verdict.

The bridge emits no native or TL evaluation result, proof, retained evidence,
qualification claim, accreditation statement or release decision.

## Public v1 records

Every `TemporalProjectionDecision` contains exactly the base fields `format`,
`kind`, `bridge_contract`, `native_contract`,
`predicate_projection_contract`, `clock_contract`, `observation_contract`,
`capture_contract`,
`target_formula_contract`, `target_semantic_contract`,
`target_trace_contract`, `target_request_contract`,
`target_evaluator_contract`, and `causes`.
`format` is `quire.contract.native-temporal-projection-decision/v1`.
`bridge_contract` is the literal bridge profile from the Description; every
other field ending in `_contract` is its exact `ContractSelection` input.

A projection contract-admission-failure shape has kind `unsupported`,
`unavailable` or `conflict`, contains exactly the base fields, and has causes
only in `native-contract`, `predicate-projection-contract`, `clock-contract`,
`observation-contract`, `capture-contract`, `formula-contract`, `semantic-contract`,
`trace-contract`, `request-contract` or `evaluator-contract`. It omits all
subject, observation, valuation and generated fields because an unadmitted
reader cannot authenticate them.

Every post-contract-admission projection additionally contains exactly
`native_subject_ref`, `native_subject_revision`, `native_subject_digest`,
`projection_set_ref`, `clock_ref`, `clock_revision`, `clock_digest`,
`observation_ref`, `observation_revision`,
`observation_digest`, `observation_state`, `activation_ref`, `activation_state`,
`capture_ref`, `capture_revision`, and `capture_digest`. Only `active` can be
admitted; `unknown` is incomplete and a
request to evaluate an authoritatively `inactive` scope is refused without a
formula or trace.

Native-subject, clock, observation and capture revisions are positive u64.
Their digests are lowercase SHA-256 over the exact bytes accepted by their
selected strict readers.

An admitted projection additionally contains exactly `formula_document`,
`formula_ref`, `node_correspondences`, `valuation_set_ref`, `trace_document`,
`trace_ref`, `evaluator_request_document`, `evaluator_request_ref`, and
`correspondence_ref`. Every other post-contract-admission projection omits
those nine members and uses only the projection cause dimensions allocated
below.

Each node correspondence contains exactly `native_node_ref`,
`native_expression_span`, `tl_node_id`, and `predicate_ref`.
`predicate_ref` is nonempty only for a native `holds(expr)` leaf and is empty
for every other node. Native spans retain their source identity and u64 byte
offsets in this correspondence. Generated TL nodes omit their optional u32
diagnostic span; truncating or re-parenting a native span is forbidden.

Every `TemporalResultJoinDecision` contains exactly the base fields `format`,
`kind`, `correspondence_ref`, `formula_ref`, `trace_ref`,
`evaluator_request_ref`, `observation_ref`,
`observation_contract`, `observation_revision`, `observation_digest`,
`observation_state`,
`availability_contract`,
`native_result_contract`, `tl_result_contract`, `comparison`, and `causes`.
`format` is `quire.contract.native-temporal-result-join/v1`.

A contract-admission-failure join has kind `unsupported`, `unavailable` or
`conflict`,
contains exactly the base fields, and has causes only in
`availability-contract`, `native-result-contract` or `tl-result-contract`.
It omits assertion and result-derived fields because no unadmitted reader may
authenticate them. Its comparison is `not-compared`.

Every post-contract-admission join additionally contains exactly
`availability_assertion_ref`, `availability_assertion_revision`,
`availability_assertion_digest`, `native_result_availability`, and
`tl_result_availability`. Each availability is `available` or `unavailable`.
The assertion revision is positive u64, its digest is lowercase SHA-256 over
its exact bytes, and its selected public strict reader shall authenticate the
assertion and both classifications before the bridge constructs this shape.

When either result availability is `unavailable`, the decision kind is
`unavailable`, comparison is `not-compared`, causes are nonempty, and the shape
contains only the base and post-contract-admission fields. It does not
fabricate a result identity, progress value or Boolean.

When both results are available and their selected outer result readers accept
the exact bytes, but any embedded progress or completeness contract is
unsupported, unavailable or conflicting, an embedded-contract-admission-failure
shape additionally contains exactly `<prefix>_result_ref`,
`<prefix>_result_revision`, `<prefix>_result_digest`,
`<prefix>_decision_scope_progress_contract`,
`<prefix>_execution_progress_contract`, and
`<prefix>_completeness_contract` for each prefix `native` and `tl`. These are
the outer-reader-authenticated identity and selection claims. The decision
kind is the precedence-selected embedded contract failure, comparison is
`not-compared`, causes are nonempty only in `progress` or `completeness`, and
the shape omits every progress/completeness assertion identity or state,
closure, truth, settlement, support, correction field and Boolean because its
owning reader was not admitted.

When both results and every embedded contract are available and admitted, the
decision additionally contains exactly the following fields for each prefix
`native` and `tl`:
`<prefix>_result_ref`, `<prefix>_result_revision`,
`<prefix>_result_digest`, `<prefix>_assessment_execution`,
`<prefix>_decision_scope_progress_contract`,
`<prefix>_decision_scope_progress_ref`,
`<prefix>_decision_scope_progress_revision`,
`<prefix>_decision_scope_progress_digest`,
`<prefix>_decision_scope_ref`, `<prefix>_decision_scope_closure`,
`<prefix>_execution_progress_contract`,
`<prefix>_execution_progress_ref`,
`<prefix>_execution_progress_revision`, `<prefix>_execution_progress_digest`,
`<prefix>_execution_scope_ref`, `<prefix>_execution_closure`,
`<prefix>_progress_clock_ref`, `<prefix>_progress_native_subject_ref`,
`<prefix>_progress_interval_ref`, `<prefix>_progress_history_boundary_ref`,
`<prefix>_progress_source_refs`,
`<prefix>_truth`, `<prefix>_settlement_basis`,
`<prefix>_decision_support_refs`, `<prefix>_completeness_contract`,
`<prefix>_completeness_ref`,
`<prefix>_completeness_revision`, `<prefix>_completeness_digest`,
`<prefix>_completeness`, `<prefix>_result_relation`,
`<prefix>_prior_result_ref`, `<prefix>_contradicted_premise_ref`, and
`<prefix>_corrected_input_ref`.

Result references are producer-owned nonempty identities. Progress and
completeness references are nonempty identities owned and authenticated by the
authorities selected in their adjacent embedded contracts; a result producer
shall not mint or restamp them. Revisions are positive u64 and digests are
lowercase SHA-256 over the exact supplied authority or result bytes. The closed
result-relation values are
`original`, `superseding`, and `invalidating`. An original result has empty
prior-result, contradicted-premise, and corrected-input references. A
superseding result has nonempty prior-result and corrected-input references and
an empty contradicted-premise reference. An invalidating result has nonempty
prior-result, contradicted-premise, and corrected-input references. Thus the
contradicted-premise reference is nonempty if and only if the relation is
`invalidating`, and no non-original relation may omit its direct predecessor.
Each field ending in `_progress_contract` or `_completeness_contract` is the
exact embedded `ContractSelection` authenticated by the selected result reader
for the adjacent assertion. The bridge shall admit each embedded selection and
its public strict reader before interpreting its reference, digest, closure or
state.
Decision-support arrays are sorted, distinct, contain at most 10,000 nonempty
identities and name only facts admitted through the correspondence. The
selected public readers shall validate the exact result bytes before the
result-bearing decision is constructed.

Each progress assertion shall bind the exact correspondence `clock_ref`,
`native_subject_ref`, decision-scope identity, surrounding-execution identity,
interval/history boundary, and sorted distinct source set exposed by these
fields. The bridge shall refuse a foreign clock, subject, scope, source or
boundary even when both producers repeat the same foreign assertion.

The closed assessment-execution values are `completed`,
`resource-incomplete`, `unsupported`, `failed`, and `refused`. The closed
closure values are `open`, `closed-complete`, and `closed-incomplete`. The
closed truth values are `true`, `false`, `pending`, and `unavailable`. The
closed settlement bases are `closed-scope`, `decisive-witness`,
`decisive-counterexample`, `unsettled`, and `unavailable`. The closed
completeness values are `complete`, `incomplete`, and `contradicted`. These
axes are independent fields and shall not be derived from each other. The
closed comparison values are `equal-final`, `equal-pending`, `not-compared`,
and `mismatch`.
Only `agreement` with `equal-final` additionally contains exactly one JSON
Boolean `value`. Every other shape omits `value`.

A `completed` assessment with final truth shall carry `closed-scope`,
`decisive-witness` or `decisive-counterexample` under the rules below. A
`completed` assessment with `pending` shall carry `unsettled` on an open
decision scope. Any non-completed assessment shall carry truth `unavailable`
and settlement basis `unavailable`. Contradicted completeness is never a
healthy completed view. Any other execution/truth/settlement/closure
combination is internally invalid and is refused before producer comparison.

Each cause contains exactly `dimension`, `code`, and `rejected_ref`, plus
`raw_discriminator` only for an unknown well-formed selection or profile.
The closed projection dimensions, in output order, are `native-contract`,
`predicate-projection-contract`, `clock-contract`, `observation-contract`,
`capture-contract`, `formula-contract`, `semantic-contract`, `trace-contract`,
`request-contract`, `evaluator-contract`, `native-subject`, `predicate-projection`,
`temporal-profile`, `operator`, `interval`, `clock`, `observation`, `activation`,
`capture`, `formula`, `trace`, and `request`.
The closed result-join dimensions, in output order, are
`availability-contract`, `availability`, `native-result-contract`,
`tl-result-contract`, `native-result`, `tl-result`, `correspondence`, `formula`,
`trace`, `request`, `observation`, `semantic-profile`, `progress`, `closure`,
`truth`, `settlement`, `support`, `completeness`, and `supersession`.
A projection shall not contain a result-join-only cause and a result join shall
not contain a projection-only cause. Causes sort by their applicable closed
dimension order, then code, rejected identity, presence of
`raw_discriminator`, and raw UTF-8 discriminator bytes. Unknown well-formed
semantic domains are
`unsupported`; accepted but unreachable contracts are `unavailable`; missing
runtime observations/history are `incomplete`; stale or mismatched identities
in an otherwise validly shaped document are `refused`; unequal content claiming
one identity or unequal conclusive native/TL results are `conflict`.
In particular, `temporal_capture_mismatch` applies only to a stale or foreign
capture identity, revision or expected digest. Unequal capture content claiming
the same identity uses `temporal_identity_conflict` and kind `conflict`.

Identity, profile and present rejected-reference strings contain 1 through
1,024 UTF-8 bytes; absence of a rejected identity is the empty
`rejected_ref`. `raw_discriminator` contains 1 through 4,096 UTF-8 bytes and is
otherwise absent. Cause arrays contain at most 1,024 entries. The bridge shall
reject duplicate or unknown object members, invalid UTF-8, trailing JSON,
invalid enum values, absent required fields, forbidden variant fields and any
byte/depth/count/string overrun, or rejection by a selected input strict reader,
with one `invalid_native_temporal_bridge`
diagnostic at the narrowest public field path and no partial decision.

The bridge shall use this closed STD-001 cause allocation:

| Code | Dimension | Decision kind |
|---|---|---|
| `temporal_native_contract_unsupported` | `native-contract` | `unsupported` |
| `temporal_native_contract_unavailable` | `native-contract` | `unavailable` |
| `temporal_native_contract_conflict` | `native-contract` | `conflict` |
| `temporal_subject_mismatch` | `native-subject` | `refused` |
| `temporal_predicate_projection_incomplete` | `predicate-projection` | `incomplete` |
| `temporal_predicate_projection_contract_unsupported` | `predicate-projection-contract` | `unsupported` |
| `temporal_predicate_projection_contract_unavailable` | `predicate-projection-contract` | `unavailable` |
| `temporal_predicate_projection_contract_conflict` | `predicate-projection-contract` | `conflict` |
| `temporal_predicate_projection_unavailable` | `predicate-projection` | `unavailable` |
| `temporal_predicate_projection_unsupported` | `predicate-projection` | `unsupported` |
| `temporal_predicate_projection_failed` | `predicate-projection` | `failed` |
| `temporal_predicate_projection_refused` | `predicate-projection` | `refused` |
| `temporal_predicate_projection_conflict` | `predicate-projection` | `conflict` |
| `temporal_predicate_projection_mismatch` | `predicate-projection` | `refused` |
| `temporal_profile_unsupported` | `temporal-profile` | `unsupported` |
| `temporal_operator_unsupported` | `operator` | `unsupported` |
| `temporal_interval_invalid` | `interval` | `refused` |
| `temporal_clock_incomplete` | `clock` | `incomplete` |
| `temporal_clock_contract_unsupported` | `clock-contract` | `unsupported` |
| `temporal_clock_contract_unavailable` | `clock-contract` | `unavailable` |
| `temporal_clock_contract_conflict` | `clock-contract` | `conflict` |
| `temporal_clock_unavailable` | `clock` | `unavailable` |
| `temporal_clock_mismatch` | `clock` | `refused` |
| `temporal_observation_incomplete` | `observation` | `incomplete` |
| `temporal_observation_contract_unsupported` | `observation-contract` | `unsupported` |
| `temporal_observation_contract_unavailable` | `observation-contract` | `unavailable` |
| `temporal_observation_contract_conflict` | `observation-contract` | `conflict` |
| `temporal_observation_mismatch` | `observation` | `refused` |
| `temporal_capture_contract_unsupported` | `capture-contract` | `unsupported` |
| `temporal_capture_contract_unavailable` | `capture-contract` | `unavailable` |
| `temporal_capture_contract_conflict` | `capture-contract` | `conflict` |
| `temporal_activation_incomplete` | `activation` | `incomplete` |
| `temporal_activation_inactive` | `activation` | `refused` |
| `temporal_activation_mismatch` | `activation` | `refused` |
| `temporal_capture_incomplete` | `capture` | `incomplete` |
| `temporal_capture_mismatch` | `capture` | `refused` |
| `temporal_formula_contract_unsupported` | `formula-contract` | `unsupported` |
| `temporal_formula_contract_unavailable` | `formula-contract` | `unavailable` |
| `temporal_formula_contract_conflict` | `formula-contract` | `conflict` |
| `temporal_semantic_contract_unsupported` | `semantic-contract` | `unsupported` |
| `temporal_semantic_contract_unavailable` | `semantic-contract` | `unavailable` |
| `temporal_semantic_contract_conflict` | `semantic-contract` | `conflict` |
| `temporal_evaluator_contract_unsupported` | `evaluator-contract` | `unsupported` |
| `temporal_evaluator_contract_unavailable` | `evaluator-contract` | `unavailable` |
| `temporal_evaluator_contract_conflict` | `evaluator-contract` | `conflict` |
| `temporal_trace_contract_unsupported` | `trace-contract` | `unsupported` |
| `temporal_trace_contract_unavailable` | `trace-contract` | `unavailable` |
| `temporal_trace_contract_conflict` | `trace-contract` | `conflict` |
| `temporal_request_contract_unsupported` | `request-contract` | `unsupported` |
| `temporal_request_contract_unavailable` | `request-contract` | `unavailable` |
| `temporal_request_contract_conflict` | `request-contract` | `conflict` |
| `temporal_formula_rejected` | `formula` | `refused` |
| `temporal_trace_rejected` | `trace` | `refused` |
| `temporal_request_rejected` | `request` | `refused` |
| `temporal_identity_conflict` | `native-subject`, `clock`, `observation`, `capture`, `formula`, `trace` or `request` | `conflict` |
| `temporal_availability_contract_unsupported` | `availability-contract` | `unsupported` |
| `temporal_availability_contract_unavailable` | `availability-contract` | `unavailable` |
| `temporal_availability_contract_conflict` | `availability-contract` | `conflict` |
| `temporal_availability_assertion_mismatch` | `availability` | `refused` |
| `temporal_availability_assertion_conflict` | `availability` | `conflict` |
| `temporal_native_result_contract_unsupported` | `native-result-contract` | `unsupported` |
| `temporal_native_result_contract_unavailable` | `native-result-contract` | `unavailable` |
| `temporal_native_result_contract_conflict` | `native-result-contract` | `conflict` |
| `temporal_tl_result_contract_unsupported` | `tl-result-contract` | `unsupported` |
| `temporal_tl_result_contract_unavailable` | `tl-result-contract` | `unavailable` |
| `temporal_tl_result_contract_conflict` | `tl-result-contract` | `conflict` |
| `temporal_native_result_unavailable` | `native-result` | `unavailable` |
| `temporal_tl_result_unavailable` | `tl-result` | `unavailable` |
| `temporal_native_result_incomplete` | `native-result` | `incomplete` |
| `temporal_tl_result_incomplete` | `tl-result` | `incomplete` |
| `temporal_native_result_unsupported` | `native-result` | `unsupported` |
| `temporal_tl_result_unsupported` | `tl-result` | `unsupported` |
| `temporal_native_result_failed` | `native-result` | `failed` |
| `temporal_tl_result_failed` | `tl-result` | `failed` |
| `temporal_native_result_refused` | `native-result` | `refused` |
| `temporal_tl_result_refused` | `tl-result` | `refused` |
| `temporal_native_result_contradicted` | `native-result` | `conflict` |
| `temporal_tl_result_contradicted` | `tl-result` | `conflict` |
| `temporal_result_binding_mismatch` | `correspondence`, `formula`, `trace`, `request`, `observation` or `semantic-profile` | `refused` |
| `temporal_result_identity_conflict` | `native-result` or `tl-result` | `conflict` |
| `temporal_progress_contract_unsupported` | `progress` | `unsupported` |
| `temporal_progress_contract_unavailable` | `progress` | `unavailable` |
| `temporal_progress_contract_conflict` | `progress` | `conflict` |
| `temporal_progress_mismatch` | `progress` | `conflict` |
| `temporal_closure_mismatch` | `closure` | `refused` |
| `temporal_result_closure_disagreement` | `closure` | `conflict` |
| `temporal_truth_mismatch` | `truth` | `conflict` |
| `temporal_settlement_mismatch` | `settlement` | `conflict` |
| `temporal_support_mismatch` | `support` | `conflict` |
| `temporal_completeness_contract_unsupported` | `completeness` | `unsupported` |
| `temporal_completeness_contract_unavailable` | `completeness` | `unavailable` |
| `temporal_completeness_contract_conflict` | `completeness` | `conflict` |
| `temporal_completeness_mismatch` | `completeness` | `conflict` |
| `temporal_supersession_invalid` | `supersession` | `refused` |

`temporal_projection_resource_exhausted` and
`temporal_result_join_resource_exhausted` are operation diagnostics rather
than causes because allocation failure cannot yield a complete decision.
Contract admission for a result join precedes assertion interpretation; an
unadmitted availability or result reader yields a base-only `unsupported`,
`unavailable` or `conflict` join without trusting assertion/result fields.
Contract conflict means unequal schema content claims the same selected
contract identity; each contract dimension has its own conflict code. After
admission, every post-contract-admission join contains the verified assertion
fields.

Projection kind precedence is `conflict`, `refused`, `failed`, `unsupported`,
`unavailable`, `incomplete`, `admitted`. Result-join kind precedence is
`conflict`, `refused`, `failed`, `unsupported`, `unavailable`, `incomplete`,
`agreement`. Each decision retains every cause at its selected or lower
precedence.

## Formula construction and identity

When all contracts and static identities are admitted, the bridge shall walk
the verified native tree in deterministic left-to-right postorder. It shall
emit exactly one TL node for each native node occurrence, shall not deduplicate
equal subtrees, and shall assign zero-based contiguous `NodeId` values in emit
order. Every operand therefore precedes its consumer and the last emitted node
is the formula root.

The v1 recursive mapping is closed:

| Native node | Exact TL node |
|---|---|
| `true` | `True` |
| `false` | `False` |
| `holds(expr)` | `Proposition` with the FR-025 `PropositionId` for that exact checked leaf |
| `not p` | `Not(visit(p))` |
| `p and q` | `And(visit(p), visit(q))` |
| `p or q` | `Or(visit(p), visit(q))` |
| `p implies q` | `Implies(visit(p), visit(q))` |
| `eventually[a,b] p` | `Future([a,b], visit(p))` |
| `always[a,b] p` | `Globally([a,b], visit(p))` |
| `p until[a,b] q` | `Until([a,b], visit(p), visit(q))` |
| `p release[a,b] q` | `Release([a,b], visit(p), visit(q))` |

The bridge shall refuse a missing leaf correspondence, duplicate native node
identity, non-tree operand, unsupported operator,
out-of-range interval, count overflow or target-reader rejection without a
partial document. Formula construction shall use public tl-syntax constructors
and readers and shall not import private wire structs.

The formula document contains exactly `schema_version`, `semantic_profile`,
`root`, and `nodes` under the selected `tl-syntax.formula/v1` contract.
`observation_state`, the surrounding-execution closure, is the only
semantic-profile selector: `open` requires `mltl.online-prefix/v1`;
`closed-complete` requires `mltl.closed-trace/v1`; `closed-incomplete` admits
neither evaluator request. A decision-scope closure, progress assertion,
settlement basis or final truth shall not select or change the profile.

The bridge shall serialize the formula as compact UTF-8 JSON using the target
contract's public field names and FR-016 Unicode-scalar key ordering, escaping
and minimal-integer rules. It shall compute `formula_ref` as lowercase SHA-256
over `quire-contract-ir`, one zero byte,
`quire.contract.tl-formula-artifact/v1`, one zero byte, and those exact formula
bytes. A caller-supplied name, syntax string, root identity or previous result
shall not replace that content identity.

TC-039 shall reproduce these independent canonical formula vectors exactly.
The byte counts exclude the Markdown newline after each code block. The digest
is the `formula_ref` construction above, not a digest of displayed prose.

Closed-trace `eventually[1,2]` over proposition 7 is 202 UTF-8 bytes and has
`formula_ref`
`f83fe8b808deb9a8a4af8ece6f2bc51f2fe3d6b5e10e1270f5fda189dcc456b3`:

```json
{"nodes":[{"kind":"proposition","proposition":7},{"interval":{"end":2,"start":1},"kind":"future","operand":0}],"root":1,"schema_version":"tl-syntax.formula/v1","semantic_profile":"mltl.closed-trace/v1"}
```

Changing only that formula's semantic profile to online-prefix produces 203
UTF-8 bytes and `formula_ref`
`41c91a141defa84acd1964ac0204603baf06d8a8b0aa5a28c41deeb08ec56465`:

```json
{"nodes":[{"kind":"proposition","proposition":7},{"interval":{"end":2,"start":1},"kind":"future","operand":0}],"root":1,"schema_version":"tl-syntax.formula/v1","semantic_profile":"mltl.online-prefix/v1"}
```

Closed-trace proposition 7 `until[1,2]` proposition 8 is 247 UTF-8 bytes and
has `formula_ref`
`a73f37d68c56854b8ea87b56e33407cf63da20b02aea568d0e98ef55c2346132`:

```json
{"nodes":[{"kind":"proposition","proposition":7},{"kind":"proposition","proposition":8},{"interval":{"end":2,"start":1},"kind":"until","left":0,"right":1}],"root":2,"schema_version":"tl-syntax.formula/v1","semantic_profile":"mltl.closed-trace/v1"}
```

## Valuation, trace and evaluator request

After formula construction, the bridge shall verify a complete rectangular
valuation population: exactly one admitted FR-025 `valued` decision for every
ordered observation position and every proposition occurring in the formula.
It shall reject a duplicate, missing, foreign-observation, wrong-revision,
wrong-predicate or wrong-proposition cell. For each cell, the observation
reader shall authenticate one injective binding from the cell's
`position_ref` to the same zero-based position, `anchor_ref`, `snapshot_ref`
and `invocation_ref` carried by that FR-025 valued decision. No
anchor/snapshot/invocation tuple or `result_projection_ref` may be replayed at
another position in the same observation. Unused FR-025 projection
correspondences require no valuation cell and do not enter the TL trace.
Position zero is the native temporal evaluation/activation anchor; later
positions follow the exact event or sample order selected by `clock_ref`.
Pre-anchor history does not enter this future-only request.

The canonical valuation-set array sorts first by zero-based position and then
by numeric `proposition_id`. Each element contains exactly `position`,
`position_ref`, `anchor_ref`, `snapshot_ref`, `invocation_ref`, `predicate_ref`,
`proposition_id`, `result_projection_ref`, and JSON Boolean `value`.
`position_ref` is the exact observation-authority identity for that position;
the three following references are copied exactly from both its authenticated
position binding and the admitted FR-025 valued decision. `valuation_set_ref`
is lowercase SHA-256 over
`quire-contract-ir`, one zero byte,
`quire.contract.native-temporal-valuation-set/v1`, one zero byte, and the
compact FR-016 canonical JSON array bytes.

The bridge shall construct one exact selected `tl-mltl.trace/v1` document. Its
public fields are `schemaVersion`, `traceId`, `closed`, and `instants`.
`schemaVersion` is `tl-mltl.trace/v1`; `traceId` is the exact
`observation_ref`; `closed` is false for `open` and true for
`closed-complete`; and `instants[i]` is the ascending array of exactly those
formula proposition IDs whose complete valuation-set cell at position `i` is
true. A false proposition is omitted only after its explicit false cell has
been verified. The TL evaluator verdict time is exactly zero. A
`closed-incomplete` observation yields no trace.

The selected trace public strict reader shall accept the generated bytes.
`trace_ref` is lowercase SHA-256 over `quire-contract-ir`, one zero byte,
`quire.contract.tl-trace-artifact/v1`, one zero byte, and the compact FR-016
canonical trace bytes.

The bridge shall then construct one exact selected `tl-mltl.command/v1`
evaluation request with public fields `schemaVersion`, `operation`,
`formulaId`, `formula`, and `trace`. Their values are respectively
`tl-mltl.command/v1`, `evaluate`, `formula_ref`, the exact formula document,
and the exact trace document. No analyze, mapping or ambient CLI option is
admitted. The selected request public strict reader shall accept the generated
bytes. `evaluator_request_ref` uses the same digest construction with domain
`quire.contract.tl-evaluator-request-artifact/v1` and the compact FR-016
canonical request bytes.

The canonical correspondence tuple contains exactly `activation_ref`,
`activation_state`, `bridge_contract`, `capture_contract`, `capture_ref`,
`capture_revision`, `capture_digest`, `clock_contract`, `clock_ref`,
`clock_revision`, `clock_digest`, `formula_ref`, `evaluator_request_ref`,
`native_contract`, `native_subject_ref`, `native_subject_revision`,
`native_subject_digest`, `observation_contract`, `observation_ref`,
`observation_revision`, `observation_digest`, `observation_state`,
`predicate_projection_contract`,
`projection_set_ref`, `target_evaluator_contract`, `target_formula_contract`,
`target_request_contract`, `target_semantic_contract`, `target_trace_contract`,
`trace_ref`, and `valuation_set_ref`. It uses compact FR-016 canonical JSON and the
same digest construction with profile
`quire.contract.native-temporal-correspondence-ref/v1`.

When any tuple member or exact formula byte changes, the bridge shall produce a
different `correspondence_ref` even if displayed text or an observed Boolean is
equal.

## Admitted semantic domain

The v1 support table is closed:

| Native profile / request | Required TL target | Projection decision |
|---|---|---|
| event-position false-extension; bounded future core; open | `mltl.online-prefix/v1` | admitted when all other premises hold |
| event-position false-extension; bounded future core; closed-complete | `mltl.closed-trace/v1` | admitted when all other premises hold |
| fixed-sample false-extension with exact epoch/positive rational period/unit and one total valuation per required sample; open or closed-complete | matching online-prefix or closed-trace profile | admitted when all other premises hold |
| either false-extension profile; closed-incomplete or missing required sample/history | no evaluator request | incomplete, not unsupported and not false padding |
| timestamped-event finite-window | current TL profiles | unsupported; no index conversion is inferred |
| `once`, `historically`, `since`, `triggered` or mixed past/future | current future-only TL profiles | unsupported until an accepted past/history target exists |

For false-extension profiles, the bridge shall preserve the native convention
that atomic predicates are false beyond a complete closed trace while Boolean
constants retain their values. It shall preserve inclusive intervals and the
FR-091 lower-bound convention: in `p until[a,b] q`, `p` is required from offset
`a` through the instant before the witness; offsets before `a` are irrelevant.

For a one-position complete closed trace with `p=true`, finite-window
`always[0,1] p` is true while false-extension `always[0,1] p` is false. The
bridge shall use this discriminator to refuse finite-window correspondence.

When a fixed-sample request is admitted, the correspondence shall retain exact
epoch, positive rational period, unit and sample-position mapping through the
selected `clock_ref`. The bridge shall not infer samples from elapsed wall time.

When a predicate projection, capture, required sample or required history is
not yet available under an otherwise supported semantic profile, the bridge
shall return `incomplete` or `unavailable` according to the owning authority;
it shall not call the semantic domain `unsupported`.

## Progress, closure and result joining

When observation state is `open`, the bridge shall admit only an online-prefix
request. A result may carry final truth while its decision scope remains open
only with `decisive-witness` for true or `decisive-counterexample` for false,
an exact complete decision-support set, and the selected profile's proof that
every admitted continuation preserves it. Otherwise an open decision scope has
truth `pending` and settlement basis `unsettled`.
A decision scope independently proven `closed-complete` by its exact progress
authority may instead settle with `closed-scope` while the surrounding
observation/execution remains open; this does not change the request's
online-prefix profile or close the surrounding axis.

When observation state is `closed-complete`, the bridge shall admit only a
closed-trace request. Its decision scope is `closed-complete`; a final truth
uses settlement basis `closed-scope`. A `pending` truth or `unsettled` basis in
that internally closed result is refused rather than compared with the other
producer.

When observation state is `closed-incomplete`, the bridge shall emit no
evaluator request and shall return `incomplete` even if a target evaluator
could manufacture a Boolean by false extension.

When both result documents are available, the bridge shall verify them with
their selected public strict readers before constructing a result-bearing join
decision. Each available result shall bind the exact
`correspondence_ref`, `formula_ref`, `trace_ref`, `evaluator_request_ref`,
observation identity/revision/digest, verdict time zero and semantic profile.
Each available result's `<prefix>_execution_closure` shall equal the admitted
`observation_state`; an unequal value is refused with
`temporal_closure_mismatch` before producer comparison, and no
`<prefix>_decision_scope_closure` value is compared with `observation_state`.
The bridge shall preserve producer-owned result identities and shall not
restamp either result.

For a healthy comparison, both normalized result views shall bind the same
decision-scope progress, surrounding-execution progress, and completeness
contract selections, identities, revisions and digests; closure values;
settlement basis; canonical decision-support identities; completeness state;
and admitted observation facts. Equal local reference strings or digests under
different contract selections are unequal premises. The two closure axes
remain independent: a completed decision scope does not close the surrounding
execution. Global completeness may remain `incomplete` without removing a
final truth only when every fact in the exact decision-support set is complete;
the gap remains in both result views and any downstream adequacy decision.

The result join mapping is exact:

| Verified state | Required join |
|---|---|
| both completed views carry the same final truth and the same valid `closed-scope` or matching decisive settlement/support premises | `agreement` / `equal-final` with that Boolean and empty causes |
| both completed views carry `pending` on an open decision scope with `unsettled` basis and equal progress/completeness premises | `agreement` / `equal-pending`, no Boolean and empty causes |
| either required result/producer is unavailable | `unavailable` / `not-compared`, no Boolean |
| an embedded progress or completeness contract is unsupported, unavailable or conflicting after both outer result readers authenticate its selection claim | matching precedence-selected kind / `not-compared`, trusted-subset shape and no Boolean |
| either assessment execution is `unsupported` | `unsupported` / `not-compared`, no Boolean |
| either assessment execution is `resource-incomplete`, or truth is unavailable because its exact decision support is incomplete | `incomplete` / `not-compared`, no Boolean |
| either assessment execution is `failed` | `failed` / `not-compared`, no Boolean |
| either assessment execution is `refused`; a validly shaped result is stale, wrong-profile or wrong-subject; or an internal truth/settlement/closure combination is invalid, including closed-complete plus pending | `refused` / `not-compared`, no Boolean |
| either result reports contradicted completeness/progress/closure, unequal content claims one result identity, or two otherwise valid views disagree on final truth, open final-versus-pending, progress, either closure axis, settlement, support or completeness | `conflict` / `mismatch`, no Boolean |

Every non-agreement join has a nonempty cause set from its exact allocated
dimensions. A structurally invalid result fails its selected strict reader and
returns `invalid_native_temporal_bridge` before any join decision; it is not a
typed semantic disagreement.

Later observation, progress, closure or completeness input may supersede or
invalidate any affected result, including one previously settled from a
complete premise. The new result shall name the exact corrected-input identity
and, for invalidation, the contradicted premise. The bridge shall validate the
selected relation, supplied predecessor bytes, identity, digest, subject,
producer, strictly earlier revision, contradicted premise, and corrected input
before accepting that edge. It shall not rewrite immutable prior result or
observation bytes, reopen a closed observation in place, or treat a premise
contradiction as an admitted continuation of the old input. Global result-graph
validation remains with the result authority; this pure join validates only
each supplied direct predecessor and correction relation.

The bridge shall operate only on supplied validated immutable values. It shall
not parse native or TL text, evaluate a predicate or temporal formula, read
ambient runtime state, invoke a callback/plugin, access the network, or add a
Java, Node, Electron or new Python semantic path.

## Dependencies

- FR-025 supplies the admitted predicate/signal/proposition population and is a
  hard prerequisite. A hardcoded proposition, Boolean default or formula-text
  name is forbidden while that projection is unavailable.
- `agent-ix/quire-specification#15` must publish accepted
  FR-048/061/090/091/093/094/095/110/112/113 definitions plus public
  source-bound temporal-subject, clock/observation/capture/progress/completeness,
  result-availability and native-result contracts and strict readers. The
  current draft is not an accepted implementation input.
- Exact public tl-syntax formula/semantic contracts and tl-mltl
  trace/evaluator-request/evaluator-report contracts and strict readers must be
  selected by immutable revisions and public schema digests. The selected
  result view must expose the independent axes required above. A Rust type,
  branch head or copied schema without a published contract does not satisfy
  admission.
- FR-012 supplies local clause/source identities. FR-023 remains the separate
  whole-clause common-expression projection and is not the temporal-tree source.
- Issue #52 owns native integration coordination. Issue #57 consumes this
  bridge for output-only FRETish mapping and is not a prerequisite.
- Any first-party implementation and qualification logic added for this
  requirement shall be Rust. This specification does not claim implementation,
  executed differential evidence, qualification or release readiness.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-026-AC-1 | Left-to-right postorder construction maps every native future-core node occurrence to the exact primitive TL node, assigns contiguous operand-before-consumer IDs without deduplication, binds every `holds(expr)` to its FR-025 proposition, constructs a complete position-authority-bound valuation set and exact TL trace/evaluation request, rejects cell replay/swapping, and passes every selected public reader. | Test (TC-039) |
| FR-026-AC-2 | Exact formula bytes and `formula_ref` change for every semantic tree/profile/proposition mutation but not for source-only display changes; valuation, trace and request identities change for their exact input mutations; `correspondence_ref` additionally changes for every source, subject, predicate projection/value, clock/capture content revision, observation position, activation or target-contract mutation. | Test (TC-039) |
| FR-026-AC-3 | Event-position and exact fixed-sample false-extension requests admit only their matching open-prefix or closed-complete target, selected only by `observation_state` as surrounding-execution closure, so an open observation with a `closed-complete` decision scope still selects `mltl.online-prefix/v1` and a result whose `<prefix>_execution_closure` differs from `observation_state` refuses with `temporal_closure_mismatch`; timestamped finite-window, past/mixed time, unknown profiles and a changed until lower-bound convention return `unsupported` with no formula. | Test (TC-039) |
| FR-026-AC-4 | Every interval at `0 <= a <= b <= u32::MAX`, including zero width and `u32::MAX`, preserves inclusive bounds; negative, fractional, inverted, unbounded or larger bounds refuse before construction without partial nodes. | Test (TC-039) |
| FR-026-AC-5 | Observation state, assessment execution, decision-scope progress/closure, surrounding-execution progress/closure, truth, settlement basis, deciding-fact identities, completeness and join kind remain independently encoded: valid equal settled and pending pairs agree, closed-complete pending refuses, runtime gaps are incomplete/unavailable rather than unsupported, and no non-final join exposes a Boolean. | Test (TC-039) |
| FR-026-AC-6 | Both real result readers reject stale formula/correspondence/trace/request/observation/profile identities, foreign progress clock/subject/scope/source/boundary bindings and impossible axis combinations; equal final results with equal contracts, progress, settlement and deciding-fact premises agree, open final/pending or Boolean/axis disagreement conflicts, unsupported/incomplete/failed/refused/contradicted execution retains its distinct kind, and each same-producer superseding or invalidating relation validates its predecessor, corrected input and any contradicted premise while preserving prior bytes without claiming global graph validation. | Test (TC-039) |
| FR-026-AC-7 | A one-position `always[0,1] p` vector and `p until[1,2] q` lower-bound vector distinguish finite-window and wrong-until semantics from the admitted false-extension profiles. | Test (TC-039) |
| FR-026-AC-8 | Missing leaf/position valuations, replayed or swapped position cells, duplicate/non-tree nodes, target formula/trace/request/result-reader rejection, count/depth/byte/allocation failure, unaccepted or conflicting contracts and every closed-dimension mismatch return a deterministic non-admitted decision or operation diagnostic with no partial artifact, parser/evaluator invocation or alternate authored TL/FRETish surface; unused FR-025 correspondences remain harmless but still change `projection_set_ref` and correspondence identity. | Test (TC-039) |
