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
  - target: ix://agent-ix/quire-specification/FR-095
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
and correspondence or return a typed fail-closed decision without a formula.

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
  `holds(expr)` occurrence in the temporal tree.
- Exact `ContractSelection` values for `tl-syntax.formula/v1`, the selected TL
  semantic profile, and the public tl-mltl evaluator/report contract.
- Exact clock and observation `ContractSelection` values; authority-verified
  clock-binding and observation bytes; observation identity, revision and
  digest;
  activation identity; immutable capture-environment identity; and observation
  state `open`, `closed-complete` or `closed-incomplete`.
- For result joining, an exact result-availability contract selection and
  authority-verified assertion bytes, identity, revision and digest; exact
  native-result and TL-result contract selections; and, for each result that
  assertion classifies `available`, immutable result bytes, producer-owned
  identity, revision and digest plus the exact trace identity and
  correspondence reference it claims.

Each `ContractSelection` uses the exact five-string FR-025 shape `contract`,
`package_version`, `repository`, immutable `revision`, and lowercase 64-hex
`schema_digest`. A branch, moving version range, copied schema or private Rust
wire type is not an accepted selection.

Each supplied or generated document contains at most 67,108,864 bytes and
semantic depth 256. The native temporal tree and node-correspondence array each
contain at most 10,000 entries. Every interval satisfies
`0 <= lower <= upper <= 4294967295`; the upper bound is the TL `u32::MAX`, not
an unbounded sentinel.

## Outputs

- A `TemporalProjectionDecision` with kind `admitted`, `incomplete`,
  `unavailable`, `unsupported`, `refused` or `conflict`.
- An admitted decision contains the exact derived formula document and bytes,
  `formula_ref`, ordered node correspondences, `correspondence_ref`, and no
  causes. Every other kind contains no formula, formula reference, node
  correspondence or usable evaluator request and has a nonempty ordered cause
  set.
- A `TemporalResultJoinDecision` for an admitted correspondence with kind
  `agreement`, `incomplete`, `unavailable`, `unsupported`, `failed`, `refused`
  or `conflict`.
  Only an `agreement` with comparison `equal-final` contains one Boolean value.
  An `equal-pending` agreement contains no Boolean value and is not a final
  verdict.

The bridge emits no native or TL evaluation result, proof, retained evidence,
qualification claim, accreditation statement or release decision.

## Public v1 records

Every `TemporalProjectionDecision` contains exactly `format`, `kind`,
`bridge_contract`, `native_contract`, `native_subject_ref`,
`native_subject_revision`, `native_subject_digest`,
`predicate_projection_contract`, `projection_set_ref`, `clock_contract`,
`clock_ref`, `observation_contract`, `observation_ref`,
`observation_revision`, `observation_digest`, `observation_state`,
`activation_ref`, `capture_ref`,
`target_formula_contract`,
`target_semantic_contract`, `target_evaluator_contract`, and `causes`.
`format` is `quire.contract.native-temporal-projection-decision/v1`.
`bridge_contract` is the literal bridge profile from the Description; every
other field ending in `_contract` is its exact `ContractSelection` input.

An admitted projection additionally contains exactly `formula_document`,
`formula_ref`, `node_correspondences`, and `correspondence_ref`. Every
non-admitted projection omits those four members and uses only the projection
cause dimensions allocated below.

Each node correspondence contains exactly `native_node_ref`,
`native_expression_span`, `tl_node_id`, and `predicate_ref`.
`predicate_ref` is nonempty only for a native `holds(expr)` leaf and is empty
for every other node. Native spans retain their source identity and u64 byte
offsets in this correspondence. Generated TL nodes omit their optional u32
diagnostic span; truncating or re-parenting a native span is forbidden.

Every `TemporalResultJoinDecision` contains exactly the base fields `format`,
`kind`, `correspondence_ref`, `formula_ref`, `trace_ref`, `observation_ref`,
`observation_contract`, `observation_revision`, `observation_digest`,
`observation_state`,
`availability_contract`,
`native_result_contract`, `tl_result_contract`, `comparison`, and `causes`.
`format` is `quire.contract.native-temporal-result-join/v1`.

A contract-admission-failure join has kind `unsupported` or `unavailable`,
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

When both results are available, the decision additionally contains exactly
`native_result_ref`, `native_result_revision`, `native_result_digest`,
`tl_result_ref`, `tl_result_revision`, `tl_result_digest`, `native_progress`,
and `tl_progress`. Result references are producer-owned nonempty identities,
revisions are positive u64, and digests are lowercase SHA-256 over the exact
supplied bytes. The selected public readers shall validate those bytes before
the result-bearing decision is constructed.

The closed progress values are `final-true`, `final-false`, `pending`,
`incomplete`, `failed`, and `refused`. The closed comparison
values are `equal-final`, `equal-pending`, `not-compared`, and `mismatch`.
Only `agreement` with `equal-final` additionally contains exactly one JSON
Boolean `value`. Every other shape omits `value`.

Each cause contains exactly `dimension`, `code`, and `rejected_ref`, plus
`raw_discriminator` only for an unknown well-formed selection or profile.
The closed projection dimensions, in output order, are `native-contract`,
`native-subject`, `predicate-projection`, `temporal-profile`, `operator`,
`interval`, `clock`, `observation-contract`, `observation`, `activation`, `capture`,
`formula-contract`, `semantic-contract`, `evaluator-contract`, and `formula`.
The closed result-join dimensions, in output order, are
`availability-contract`, `native-result-contract`, `tl-result-contract`,
`native-result`, `tl-result`, `correspondence`, `formula`, `trace`,
`observation`, `semantic-profile`, `progress`, `closure`, and `supersession`.
A projection shall not contain a result-join-only cause and a result join shall
not contain a projection-only cause. Causes sort by their applicable closed
dimension order, then code, rejected identity, presence of
`raw_discriminator`, and raw UTF-8 discriminator bytes. Unknown well-formed semantic domains are
`unsupported`; accepted but unreachable contracts are `unavailable`; missing
runtime observations/history are `incomplete`; stale or mismatched identities
in an otherwise validly shaped document are `refused`; unequal content claiming one identity or unequal
conclusive native/TL results are `conflict`.

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
| `temporal_predicate_projection_unsupported` | `predicate-projection` | `unsupported` |
| `temporal_predicate_projection_unavailable` | `predicate-projection` | `unavailable` |
| `temporal_predicate_projection_mismatch` | `predicate-projection` | `refused` |
| `temporal_profile_unsupported` | `temporal-profile` | `unsupported` |
| `temporal_operator_unsupported` | `operator` | `unsupported` |
| `temporal_interval_invalid` | `interval` | `refused` |
| `temporal_clock_incomplete` | `clock` | `incomplete` |
| `temporal_clock_contract_unsupported` | `clock` | `unsupported` |
| `temporal_clock_unavailable` | `clock` | `unavailable` |
| `temporal_clock_mismatch` | `clock` | `refused` |
| `temporal_observation_incomplete` | `observation` | `incomplete` |
| `temporal_observation_contract_unsupported` | `observation-contract` | `unsupported` |
| `temporal_observation_contract_unavailable` | `observation-contract` | `unavailable` |
| `temporal_observation_mismatch` | `observation` | `refused` |
| `temporal_activation_mismatch` | `activation` | `refused` |
| `temporal_capture_incomplete` | `capture` | `incomplete` |
| `temporal_capture_mismatch` | `capture` | `refused` |
| `temporal_formula_contract_unsupported` | `formula-contract` | `unsupported` |
| `temporal_formula_contract_unavailable` | `formula-contract` | `unavailable` |
| `temporal_semantic_contract_unsupported` | `semantic-contract` | `unsupported` |
| `temporal_semantic_contract_unavailable` | `semantic-contract` | `unavailable` |
| `temporal_evaluator_contract_unsupported` | `evaluator-contract` | `unsupported` |
| `temporal_evaluator_contract_unavailable` | `evaluator-contract` | `unavailable` |
| `temporal_formula_rejected` | `formula` | `refused` |
| `temporal_identity_conflict` | `formula` | `conflict` |
| `temporal_availability_contract_unsupported` | `availability-contract` | `unsupported` |
| `temporal_availability_contract_unavailable` | `availability-contract` | `unavailable` |
| `temporal_native_result_contract_unsupported` | `native-result-contract` | `unsupported` |
| `temporal_native_result_contract_unavailable` | `native-result-contract` | `unavailable` |
| `temporal_tl_result_contract_unsupported` | `tl-result-contract` | `unsupported` |
| `temporal_tl_result_contract_unavailable` | `tl-result-contract` | `unavailable` |
| `temporal_native_result_unavailable` | `native-result` | `unavailable` |
| `temporal_tl_result_unavailable` | `tl-result` | `unavailable` |
| `temporal_native_result_incomplete` | `native-result` | `incomplete` |
| `temporal_tl_result_incomplete` | `tl-result` | `incomplete` |
| `temporal_native_result_failed` | `native-result` | `failed` |
| `temporal_tl_result_failed` | `tl-result` | `failed` |
| `temporal_native_result_refused` | `native-result` | `refused` |
| `temporal_tl_result_refused` | `tl-result` | `refused` |
| `temporal_result_binding_mismatch` | `correspondence`, `formula`, `trace`, `observation` or `semantic-profile` | `refused` |
| `temporal_result_identity_conflict` | `native-result` or `tl-result` | `conflict` |
| `temporal_progress_mismatch` | `progress` | `conflict` |
| `temporal_closure_mismatch` | `closure` | `refused` |
| `temporal_supersession_invalid` | `supersession` | `refused` |

`temporal_projection_resource_exhausted` and
`temporal_result_join_resource_exhausted` are operation diagnostics rather
than causes because allocation failure cannot yield a complete decision.
Contract admission for a result join precedes assertion interpretation; an
unadmitted availability or result reader yields a base-only `unsupported` or
`unavailable` join without trusting assertion/result fields. After admission,
every post-contract-admission join contains the verified assertion fields.

Projection kind precedence is `conflict`, `refused`, `unsupported`,
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
`root`, and `nodes` under the selected `tl-syntax.formula/v1` contract. The
selected observation state determines the profile: `open` requires
`mltl.online-prefix/v1`; `closed-complete` requires
`mltl.closed-trace/v1`; `closed-incomplete` admits neither evaluator request.

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
`f408207cf312e5c387741885e343381f16c9b29bfa4c2a02f58632ffad17a861`:

```json
{"nodes":[{"kind":"proposition","proposition":7},{"interval":{"end":2,"start":1},"kind":"future","operand":0}],"root":1,"schema_version":"tl-syntax.formula/v1","semantic_profile":"mltl.closed-trace/v1"}
```

Changing only that formula's semantic profile to online-prefix produces 203
UTF-8 bytes and `formula_ref`
`6f894f388c80dc2e54ee29c8ff833bc017efc502aa1bf2d7bcfc87c8ad35df5c`:

```json
{"nodes":[{"kind":"proposition","proposition":7},{"interval":{"end":2,"start":1},"kind":"future","operand":0}],"root":1,"schema_version":"tl-syntax.formula/v1","semantic_profile":"mltl.online-prefix/v1"}
```

Closed-trace proposition 7 `until[1,2]` proposition 8 is 247 UTF-8 bytes and
has `formula_ref`
`a8f2b5894964123635284ac0d87c0e831439dcd7a60be87947237576dc08544e`:

```json
{"nodes":[{"kind":"proposition","proposition":7},{"kind":"proposition","proposition":8},{"interval":{"end":2,"start":1},"kind":"until","left":0,"right":1}],"root":2,"schema_version":"tl-syntax.formula/v1","semantic_profile":"mltl.closed-trace/v1"}
```

The canonical correspondence tuple contains exactly `activation_ref`,
`bridge_contract`, `capture_ref`, `clock_contract`, `clock_ref`, `formula_ref`,
`native_contract`, `native_subject_ref`, `native_subject_revision`,
`native_subject_digest`, `observation_contract`, `observation_ref`,
`observation_revision`, `observation_digest`, `observation_state`,
`predicate_projection_contract`,
`projection_set_ref`, `target_evaluator_contract`, `target_formula_contract`,
and `target_semantic_contract`. It uses compact FR-016 canonical JSON and the
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
request. A native or TL result may be final early only if every permitted
continuation preserves it; otherwise its progress is `pending`.

When observation state is `closed-complete`, the bridge shall admit only a
closed-trace request and shall refuse a `pending` result as inconsistent.

When observation state is `closed-incomplete`, the bridge shall emit no
evaluator request and shall return `incomplete` even if a target evaluator
could manufacture a Boolean by false extension.

When both result documents are available, the bridge shall verify them with
their selected public strict readers before constructing a result-bearing join
decision. Each available result shall bind the exact
`correspondence_ref`, `formula_ref`, `trace_ref`, observation identity/revision
and semantic profile. The bridge shall preserve producer-owned result identities
and shall not restamp either result.

The result join mapping is exact:

| Verified state | Required join |
|---|---|
| open; both progress values are the same final Boolean | `agreement` / `equal-final` with that Boolean |
| open; both are `pending` | `agreement` / `equal-pending`, no Boolean |
| closed-complete; both are the same final Boolean | `agreement` / `equal-final` with that Boolean |
| either available result reports `incomplete` | `incomplete` / `not-compared`, no Boolean |
| either required result/producer is unavailable | `unavailable` / `not-compared`, no Boolean |
| either execution failed | `failed` / `not-compared`, no Boolean |
| either execution reports `refused`, or a validly shaped result is stale, wrong-profile or wrong-subject | `refused` / `not-compared`, no Boolean |
| unequal final Booleans, final versus pending, or unequal content claiming one identity | `conflict` / `mismatch`, no Boolean |

Late data may supersede an open observation/result under a new revision and
direct-predecessor identity. It shall not rewrite immutable prior bytes or a
closed observation. Global result-graph validation remains with the result
authority; this pure join validates only the supplied direct predecessor.

The bridge shall operate only on supplied validated immutable values. It shall
not parse native or TL text, evaluate a predicate or temporal formula, read
ambient runtime state, invoke a callback/plugin, access the network, or add a
Java, Node, Electron or new Python semantic path.

## Dependencies

- FR-025 supplies the admitted predicate/signal/proposition population and is a
  hard prerequisite. A hardcoded proposition, Boolean default or formula-text
  name is forbidden while that projection is unavailable.
- `agent-ix/quire-specification#15` must publish accepted FR-048/090/091/093/095
  definitions plus a public source-bound temporal-subject reader and native
  result contract. The current draft is not an accepted implementation input.
- The exact tl-syntax formula schema/reader and tl-mltl evaluator/report contract
  must be selected by immutable revisions and public schema digests. A Rust
  implementation type without a published contract does not satisfy admission.
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
| FR-026-AC-1 | Left-to-right postorder construction maps every native future-core node occurrence to the exact primitive TL node, assigns contiguous operand-before-consumer IDs without deduplication, binds every `holds(expr)` to its FR-025 proposition, and passes the selected public formula reader. | Test (TC-039) |
| FR-026-AC-2 | Exact formula bytes and `formula_ref` change for every semantic tree/profile/proposition mutation but not for source-only display changes; `correspondence_ref` additionally changes for every source, subject, predicate projection, clock, observation, activation, capture or target-contract mutation. | Test (TC-039) |
| FR-026-AC-3 | Event-position and exact fixed-sample false-extension requests admit only their matching open-prefix or closed-complete target; timestamped finite-window, past/mixed time, unknown profiles and a changed until lower-bound convention return `unsupported` with no formula. | Test (TC-039) |
| FR-026-AC-4 | Every interval at `0 <= a <= b <= u32::MAX`, including zero width and `u32::MAX`, preserves inclusive bounds; negative, fractional, inverted, unbounded or larger bounds refuse before construction without partial nodes. | Test (TC-039) |
| FR-026-AC-5 | Open, closed-complete and closed-incomplete remain distinct from native/TL progress and result kind: equal early-final and equal-pending pairs agree as specified, closed-complete pending refuses, runtime gaps are incomplete/unavailable rather than unsupported, and no non-final join exposes a Boolean. | Test (TC-039) |
| FR-026-AC-6 | Both real result readers reject stale formula/correspondence/trace/observation/profile identities; equal final results agree, final/pending or Boolean disagreement conflicts, failure remains failed, and direct supersession preserves prior bytes without claiming global graph validation. | Test (TC-039) |
| FR-026-AC-7 | A one-position `always[0,1] p` vector and `p until[1,2] q` lower-bound vector distinguish finite-window and wrong-until semantics from the admitted false-extension profiles. | Test (TC-039) |
| FR-026-AC-8 | Missing leaf mappings, duplicate/non-tree nodes, target-reader rejection, count/depth/byte/allocation failure, unaccepted contracts and every closed-dimension mismatch return a deterministic non-admitted decision or operation diagnostic with no partial formula, parser/evaluator invocation or alternate authored TL/FRETish surface; unused FR-025 correspondences remain harmless but still change `projection_set_ref` and correspondence identity. | Test (TC-039) |
