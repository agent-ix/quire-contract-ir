---
id: FR-057
title: "Bind the IR contribution to cross-repository conformance"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: depends_on
  - target: ix://agent-ix/engineering-assurance/FR-008
    type: depends_on
  - target: ix://agent-ix/engineering-assurance/FR-013
    type: depends_on
  - target: ix://agent-ix/engineering-assurance/FR-019
    type: depends_on
  - target: ix://agent-ix/quire-cli/FR-020
    type: depends_on
  - target: ix://agent-ix/quoin/FR-030
    type: depends_on
  - target: ix://agent-ix/quoin/FR-032
    type: depends_on
  - target: ix://agent-ix/quoin/FR-069
    type: depends_on
---
# FR-057: Bind the IR contribution to cross-repository conformance

## Description

The IR conformance contribution SHALL associate the published inputs, participating consumer obligations and
actual native results through the existing shared assurance contracts before it
is reported as conforming in a selected cross-repository campaign.

This requirement defines the IR-facing contribution and its acceptance
conditions. It does not create a campaign runner, evidence record family,
retained corpus format or general-purpose conformance framework. Engineering
Assurance and Quoin remain the shared contract owners. The IR-146 campaign
closeout remains a separately owned execution and human decision.

## Inputs

- The campaign owner's explicit selection of participating repositories,
  candidate revisions, governing definitions and required consumer obligations.
- The selected published IR corpus and its schemas, trace registry,
  expectations and referenced canonical files under
  [FR-018](./FR-018-conformance-corpus.md).
- Actual structured producer results under
  [FR-020](./FR-020-json-conformance-interface.md), and consumer-native results
  under each consumer's existing governing contract.
- Existing shared definition, execution, result and retained-evidence references
  under `ix://agent-ix/engineering-assurance/FR-008` and Quoin's store and adapters.

## Outputs

An IR contribution whose scope, native outcomes and shared evidence references
are explicit, or an explanation of the missing, mismatching or unrepresented
obligation that prevents that contribution from qualifying. These are campaign
judgments over existing records, not a new serialized result protocol.

## Behavior

### Select and obtain the published inputs

The campaign owner SHALL select the required repository/consumer-obligation
population before its results are assessed. If a required participant,
obligation or governing definition is absent, then the IR contribution SHALL
remain unqualified rather than silently reducing that population.

Each IR corpus consumer SHALL use the selected published corpus together with
its corresponding schemas, trace registry, expectations and referenced
canonical files. Before accepting a contribution, the consumer SHALL establish
that those input bytes equal the selected upstream publication. Comparing only
a directory name, corpus ID, schema identity or fixture count does not establish
that equality. A changed, missing or additional file in that selected input
population SHALL prevent acceptance.

The contribution SHALL identify the selected input source and candidate through
the existing shared execution/input bindings. Dependency selections SHALL remain
in the owning manifests and lockfiles. This requirement SHALL NOT introduce a
second revision catalog, checksum ledger, copied corpus, or new import manifest.
It neither generalizes the historical vendored-corpus convention nor changes
existing canonical identity semantics.

### Execute and compare the actual selected population

The producer fixture population is every input in the selected published IR
corpus, with its corresponding expectation and referenced canonical files.
A consumer projection does not shrink this producer population or change the
runner's closed argument grammar.

The IR producer SHALL retain the closed invocation, explicit roots, bounded
preload and comparison semantics of [FR-018](./FR-018-conformance-corpus.md) and
[FR-020](./FR-020-json-conformance-interface.md). A complete matching producer run
requires exit 0 and exactly one structured `match` row per selected fixture,
with the expected corpus, operation, fixture and tool/schema/profile identities,
observed token inventory and trace metadata. An expected-invalid fixture can
match: its authored refusal must agree with the actual structured result.

If a run is empty, truncated, malformed, duplicated, contains a foreign or extra
fixture, lacks a required fixture or trace, or reports a mismatch or incomplete
execution, then that run SHALL NOT qualify as a matching contribution. Exit 1
retains complete-mismatch semantics; exit 2 retains incomplete-run semantics.
Exit 0 alone, unstructured console text, a test tag, a static export, or a
successful schema check SHALL NOT establish native conformance.

Each participating consumer SHALL exercise its selected obligations through its
own actual public semantics, using the same selected IR inputs or an
owner-specified projection of them. The projection and comparison obligations
SHALL be selected by the governing consumer contract, not inferred from the
fixture name or from whichever cases the consumer happens to support. Merely
running the IR producer in a consumer checkout does not demonstrate that
consumer's semantic agreement. If an obligation requires independent consumer
agreement and that execution or oracle is unavailable, then it SHALL remain
unqualified. A matching IR corpus run establishes the producer's stated
behavior, not independent correctness.

The consumer SHALL preserve the native meaning of expected refusals, mismatch,
unsupported, unavailable and incomplete results. No such result may be renamed
to pass or treated as an exclusion without a new explicit campaign-scope
decision. When a governing negative control expects a refusal, a matching refusal
qualifies only that refusal-behavior obligation, not independent semantic
agreement. Contract and temporal contributions remain separate native domains;
this requirement supplies no implicit translation between them.

### Associate results through their existing owners

| Concern | Existing authority | IR contribution boundary |
| --- | --- | --- |
| IR input validity, canonical bytes, observed tokens and native comparisons | [FR-018](./FR-018-conformance-corpus.md), [FR-020](./FR-020-json-conformance-interface.md) | Publish and execute the domain contract; do not decide another consumer's semantics. |
| Static obligations and governing definitions | `ix://agent-ix/quire-cli/FR-020`, consumed through `ix://agent-ix/engineering-assurance/FR-008` | Reference actual definitions; static coverage is not a run. |
| Bounded producer execution and native failure states | `ix://agent-ix/engineering-assurance/FR-019` | Reuse its declared execution/input/adapter bindings and limits; do not add a local process runner. |
| Native result transcription and trace preservation | `ix://agent-ix/quoin/FR-069` | Reuse `contract-conformance` for the declared IR JSONL protocol and the existing applicable adapter for a consumer's native format. |
| Retained run/binding records and evidence audit | `ix://agent-ix/quoin/FR-030`, `ix://agent-ix/quoin/FR-032`, `ix://agent-ix/engineering-assurance/FR-008` | Retain and assess through those owners; do not introduce another evidence store or integrity record. |
| Campaign sufficiency and completion | Campaign decision owner, with existing ix-flow human-decision handling under Engineering Assurance | IR-146 records the selected campaign's decision; no local command or adapter decides sufficiency. |

Each accepted native result SHALL be associated with its actual governing
definition, candidate, execution and selected inputs through the shared owner
contracts. Missing, stale, foreign or unresolved associations SHALL prevent that
contribution from qualifying. A copied run ID, matching corpus ID, coherent
Cargo lock or newly generated report is not a substitute for those associations.
If an existing shared interface cannot represent a required association, then
the campaign SHALL record that gap with its shared owner rather than inventing
an IR-local envelope or inferring the missing relation.

IR result intake SHALL preserve the producer's `trace_ids` through Quoin FR-069.
A legacy row that the adapter accepts without trace metadata SHALL NOT acquire
inferred traces or satisfy a selected obligation that requires them. An
unrepresented native result SHALL remain visible and unqualified. The adapter's
success does not by itself establish corpus completeness, consumer agreement,
trace-criterion sufficiency or overall campaign acceptance.

The campaign SHALL use the shared audit/retention contracts to assess the
referenced evidence. Missing, failed, vacuous, suspect, stale or unreadable
required evidence SHALL remain an explicit non-success for the affected scope.
An audit is a read-only assessment, not authorization to rerun a producer or
rewrite retained history. A rerun does not silently reaffirm changed definitions.

### Bound the completion claim

The IR contribution SHALL qualify only when every selected producer and consumer
obligation has its required matching native result, complete population and
accepted shared associations. A passed repository-local test, coherent lock,
producer-only corpus run or readable historical compatibility corpus SHALL NOT
be promoted to a cross-repository pass. The campaign decision owner SHALL retain
the final sufficiency/completion decision separately from those results.

Where an observation is governed as a measurement, the campaign SHALL identify
the applicable active MeasurementPlan through the existing Engineering Assurance
measurement contract. If a required plan or decision owner is unresolved, then
the campaign SHALL report that unresolved input rather than create a generic
plan or claim completion. This requirement creates no MeasurementPlan.

New first-party production and qualification logic for this contribution SHALL
be Rust. Reusable assurance logic SHALL remain with its shared owner; IR-only
logic SHALL remain domain validation. A missing reusable capability SHALL be
routed to that owner. Existing Quoin use is retained without expanding its
implementation-language scope; no wrapper around a new non-Rust semantic
implementation satisfies this obligation.

## Open Owner Decisions

The live campaign's participant/obligation selection and IR-146 human decision
owner remain to be confirmed. Engineering Assurance and Quoin owner concurrence
on this proposed domain binding is required before its acceptance. No approval
of a review-method subset or shared-owner amendment is inferred from this draft.

The qa-corpus PR #23 removal proposal and Engineering Assurance FR-011's retained
compatibility-corpus contract remain a separate, unresolved owner matter. This
requirement changes neither. Historical compatibility data is not evidence of a
live candidate execution. This draft, its validation and its review do not
implement or run the campaign and do not close IR-123's program obligations.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-057-AC-1 | Inspection identifies the selected participants, required consumer obligations, governing definitions and campaign decision owner; an absent required selection remains unresolved rather than becoming an implicit exclusion. | Inspection |
| FR-057-AC-2 | A consumer accepts byte-equal selected published inputs; changing, omitting or adding a selected input, expectation, schema, trace-registry or referenced canonical file prevents acceptance despite unchanged names, IDs and file counts. | Test |
| FR-057-AC-3 | A nonempty complete producer run with exact fixture identities, matching native outcomes, observed inventory and required trace metadata qualifies its producer contribution; an expected-invalid fixture qualifies only when its actual structured refusal matches its expectation. | Test |
| FR-057-AC-4 | Empty, truncated, malformed, duplicated, extra or foreign fixture results, a missing fixture or required trace, exit 1 mismatch and exit 2 incomplete execution each prevent a matching contribution, including controls where the process nevertheless exits 0. | Test |
| FR-057-AC-5 | A selected independent consumer obligation is discharged only by that consumer's actual public-semantic comparison under its governing projection/oracle; a producer-only rerun, unavailable oracle, unsupported result or undocumented subset does not discharge it. | Test |
| FR-057-AC-6 | A matching result with correct shared definition/candidate/execution/input associations qualifies; removing, substituting or making any required association stale prevents qualification without inferring a replacement from corpus ID, revision text or a coherent lock. | Test |
| FR-057-AC-7 | Contract-conformance intake preserves supplied trace IDs; an accepted legacy row without traces, an unmatched required trace or an unrepresented native state remains visible and cannot satisfy an obligation requiring that information. | Test |
| FR-057-AC-8 | Missing, failed, vacuous, suspect, stale or unreadable required shared evidence prevents qualification; an audit, a passing rerun or an accessible historical compatibility record does not fabricate or reaffirm the missing evidence. | Test |
| FR-057-AC-9 | Inspection finds only existing shared execution, adapter, store, audit and human-decision contracts, no new generic envelope, process runner, copied corpus, revision/checksum catalog or inferred cross-domain equivalence; new first-party executable logic is Rust and existing Quoin retention is not permission to expand non-Rust semantics. | Inspection |
| FR-057-AC-10 | An all-obligations-satisfied contribution and the human completion decision remain distinct; absent owner/required MeasurementPlan, unrelated repository-local passes or this spec's validation cannot establish campaign completion. | Inspection, Test |

## Dependencies

- [StR-003](../../core/stakeholder/StR-003-reviewable-assurance.md) requires
  reviewable failures and reproducible conformance.
- [FR-018](./FR-018-conformance-corpus.md) and
  [FR-020](./FR-020-json-conformance-interface.md) own the unchanged IR corpus and
  process protocol.
- `ix://agent-ix/engineering-assurance/FR-008` owns the separation and association
  of definitions, executions, results, evidence and measurements;
  `ix://agent-ix/engineering-assurance/FR-013` owns the migration and shared/domain
  boundary; `ix://agent-ix/engineering-assurance/FR-019` owns bounded execution.
- `ix://agent-ix/quire-cli/FR-020` owns static assurance export; an exported
  definition is not an execution or result.
- `ix://agent-ix/quoin/FR-069` owns the actual campaign-native adapters and trace
  preservation; `ix://agent-ix/quoin/FR-030` owns retained runs and bindings;
  `ix://agent-ix/quoin/FR-032` owns evidence audit.
- IR-146 is the campaign execution/closeout work, not an implementation supplied
  by this specification. Shared-owner decisions above remain explicit prerequisites.
