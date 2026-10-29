---
id: FR-020
title: "Expose versioned JSON and conformance-runner interfaces"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
  - target: ix://agent-ix/ix-cli-kit/FR-017
    type: depends_on
  - target: ix://agent-ix/ix-cli-kit/FR-006
    type: depends_on
  - target: ix://agent-ix/ix-cli-kit/FR-003
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: depends_on
---
# FR-020: Expose versioned JSON and conformance-runner interfaces

## Contract

```yaml
name: ContractIrJsonConformanceApi
ownership: quire-contract-ir
inputs:
  - UTF-8 JSON bytes
  - optional conformance corpus directory
  - explicit schema and canonicalization profile identities
outputs:
  - JSON Lines conformance results
  - stable process exit classification
invariants:
  - output vocabulary is independent of Rust type names and debug formatting
  - standard output remains machine-readable
  - operational errors are written to standard error
compatibility:
  schema: versioned and fail-closed
```

## Description

The repository shall expose a versioned JSON package interface and a
process-level conformance runner whose output is independent of Rust type names
and debug formatting.

## Inputs

UTF-8 JSON bytes, an optional conformance corpus directory, and explicit schema and
canonicalization profile identities.

## Outputs

JSON Lines results containing fixture ID, observed coverage tokens and their criterion trace IDs, validity, ordered
diagnostics, canonical bytes, dependency identities, tool identity, and exit classification.

## Behavior

The executable name is `quire-contract-conformance`. Its protocol identity is
`quire.contract.conformance-jsonl/v1`. The closed invocation is
`quire-contract-conformance run --corpus <directory> --schemas <directory>` plus
`--version`; unknown,
missing, repeated, or non-UTF-8 arguments are invocation failures. The runner
does not search parent directories, environment variables, network locations,
or a default corpus or schema directory.

For a valid corpus, standard output contains exactly one compact JSON object
and newline per fixture in fixture-name order. No banner or progress text appears.
Each result contains protocol, corpus ID, fixture ID, operation, closed status
`match` or `mismatch`, unique mismatch kinds in fixed registry order, the fixture's observed `covers`
and sorted `trace_ids`, actual structured result, and tool identity: crate version, package-schema path, canonical profile,
and runner protocol. Mismatch kinds are `validity`, `diagnostics`,
`canonical_bytes`, `dependencies`, and `coverage`. A fixture with several drifts retains all applicable kinds once in
this fixed registry order, which is not lexical sorting. Diagnostic messages may be emitted for humans but never
participate in comparison.

Exit 0 means every emitted fixture is `match`. Exit 1 means the corpus and
environment were valid and every fixture ran, but at least one expectation
mismatched. Exit 2 means an invocation, corpus/schema/profile/path/I/O/resource
failure prevented a complete run. Exit 2 emits no standard
output and exactly one compact JSON error plus newline to standard error with
protocol, closed code (`invalid_invocation`, `invalid_corpus`,
`unsupported_profile`, `unsafe_path`, `fixture_io`, or `resource_exhausted`),
and stable path; prose detail is non-comparable. Panics,
partial JSON, and mixed stdout/stderr records are forbidden.

### Shared CLI boundary

The root bridge package SHALL consume `ix-cli-kit` from its authoritative
`agent-ix/ix-cli-kit` Git source only through the default-off `conformance-cli`
feature required by `quire-contract-conformance`, with the selected dependency
recorded in `Cargo.lock`. The root's default production dependency closure and
the cycle-free `quire-contract-model` package SHALL NOT contain `ix-cli-kit`.
The repository SHALL NOT copy the kit's implementation or introduce an alternate
argument grammar to perform this adoption.

The executable SHALL pass its caller-owned status through
`ix_cli_kit::exit::caller_exit_code`, as specified by the kit's FR-017. Its only
selected statuses remain 0, 1 and 2 with the meanings above, including exit 2
for malformed invocation. These statuses SHALL NOT be reclassified through the
kit's separate `Outcome` taxonomy: IR's completed mismatch and incomplete-run
statuses retain their existing protocol meanings. No route SHALL request the
reserved status 5. The kit's pass-through error SHALL be handled explicitly
without Rust's `Result` termination emitting an additional diagnostic or selecting
an implicit status.

The executable SHALL serialize existing result and diagnostic envelopes with the
kit's `json::write_compact`, retaining the serialized field order and appending
one caller-owned newline. Result serialization SHALL finish in the existing
private buffer before stdout delivery. The kit's writer can leave partial bytes
on failure, so that failure SHALL discard the private result buffer and retain
the existing `resource_exhausted` result-buffer error. This adoption SHALL NOT
replace either envelope with a kit diagnostic record or sort its fields. Primary
result bytes SHALL be delivered through the kit's `streams::write_primary_stdout`;
a write or flush error retains `fixture_io`, path `stdout`, and exit 2. As with
any output-device failure, this does not guarantee that already-written stdout
bytes can be retracted. Diagnostic delivery remains best effort on stderr and
retains the existing error-buffer fallback.

The closed invocation, version line, six operational JSON codes, complete-run
payload classification, corpus comparison and ordering, published input, schema,
and canonical-file bytes SHALL remain unchanged. Published expectation files
SHALL change only to remove canonical-digest fields; the corresponding mismatch
kind SHALL be removed, and the generator's file-regeneration gate SHALL retire.
All other fixture comparisons and vectors SHALL remain unchanged. In particular,
no model error-cause discriminator is needed to map different resource failures
to different process statuses: every incomplete run continues to select exit 2.

The operational classification is exact: malformed, missing, repeated, or
non-UTF-8 arguments map to `invalid_invocation`; a corpus with no fixtures, an
`inputs/` or `expectations/` entry that is not a UTF-8 `.json` file, an
expectation with no input, a fixture name without a known operation, a malformed schema, fixture input or
expectation, a successful package or `document_json` expression document the
published schema rejects, and an
observed coverage union that differs from the inventory map to
`invalid_corpus`; unknown package or conformance schema identities map to
`unsupported_profile`; absolute,
traversing, escaping, or escaping-symlink paths map to `unsafe_path`; missing,
unreadable, non-regular, or changed-after-preload files map to `fixture_io`; and
file/count/byte/allocation/pre-decode-nesting limits map to
`resource_exhausted`.
The total logical preload budget is 67108864 bytes across the schemas, inputs,
expectations, and canonical files; a repeated canonical path is charged on
every reference. Two further exact limits bound one run and are exported
alongside it: a single file may be at most 16777216 bytes, and a corpus may
hold at most 10000 fixtures. Both are exceeded
before any result is emitted and map to `resource_exhausted` like every other
preload limit. The per-file limit is strictly below the total budget, so no
single file can exhaust a run on its own. Raw JSON nesting is scanned before
recursive materialization and is limited to 576 levels.

The code change that landed the v1 decimal-string integers (IR-274 code change
B) amended the published schemas, the corpus and the runner together, because
the runner and the executable-projection binder read the schemas at run time
and a schema that disagrees with the corpus fails the run as `invalid_corpus`.
The published fixture schema carries the eight integer members of FR-013 (an integer type's
`minimum` and `maximum`, a rational type's `numerator_minimum`,
`numerator_maximum` and `maximum_denominator`, an integer literal's `value`, a
rational literal's `numerator` and `denominator`) as `IntegerString` strings
(`^(0|-?[1-9][0-9]*)$`), and the fixture and package schemas bound a revision
and a byte offset at 2^53 (FR-011, FR-012). Two levels decide an input that
carries a JSON number in one of the eight members, and they do not overlap: a
valid-operation input is checked against the schema first, so it is
`invalid_corpus`; a fixture that exercises the decoder's refusal supplies the
wire to the decoder directly (an input of the form `{"document_json": "<text>"}`,
which the package and expression operations both accept) and its expectation
carries `invalid_wire_format`,
as FR-013 states for every v1 loader. Every canonical file of the corpus whose
object holds one of the eight members (47 of the 75 canonical files and 59 of
the 99 inputs at the time of writing) is re-recorded with `quire-canonical`'s
bytes (FR-016), and the corpus holds no earlier number spelling of them.

The runner reads every referenced file before emitting its first result, then
executes fixtures without mutation and buffers every result until the complete
run succeeds or mismatches; any operational failure discards the buffer before
writing the single standard-error record. Error detail, when present, is a
closed normalized phrase with no OS error text or absolute path. Repeating a run
over unchanged bytes is byte-identical. It performs no network access, clock
reads, random generation, absolute-path emission, or host-map-order output.
`--version` prints the crate version and protocol identity to standard output
and exits 0 without reading a corpus.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-020-AC-1 | A process test runs the published corpus twice without linking a test harness to the library and obtains byte-identical JSON Lines, non-empty observed trace ids per fixture input, empty stderr, and complete tool/schema/profile identity. | Test (TC-018) |
| FR-020-AC-2 | Process fixtures verify exit 1 with all five mismatch kinds (`validity`, `diagnostics`, `canonical_bytes`, `dependencies`, `coverage`) in that fixed order and exit 2 for each of the six closed operational codes (`invalid_invocation`, `invalid_corpus`, `unsupported_profile`, `unsafe_path`, `fixture_io`, `resource_exhausted`), with no absolute path in the error record; stdout/stderr separation, no partial output, `--version`, unknown/repeated arguments, non-UTF-8 argument handling, and pre-decode rejection of a 60000-level referenced JSON input are exact. | Test (TC-018) |
| FR-020-AC-3 | The published fixture schema accepts `"-9223372036854775808"`, `"0"` and `"9223372036854775807"` in each of the eight integer members. | Test (TC-018) |
| FR-020-AC-4 | The published package schema accepts a revision and a byte offset of `9007199254740992`. | Test (TC-018) |
| FR-020-AC-5 | The fixture input schema refuses as `invalid_corpus` values above `9007199254740992` for each of these members: a probe clause-reference revision; a coverage trace source or target-span revision or byte offset; and a coverage trace target revision. | Test (TC-018) |
| FR-020-AC-6 | The fixture input schema passes the value `9007199254740992` for each of these members: a probe clause-reference revision; a coverage trace source or target-span revision or byte offset; and a coverage trace target revision. | Test (TC-018) |
| FR-020-AC-7 | The runner returns `invalid_wire_format` when a decoder fixture supplies a JSON number in one of the eight integer members straight to the decoder. | Test (TC-018) |
| FR-020-AC-8 | The expectation of a decoder fixture that supplies a JSON number in one of the eight integer members carries the code `invalid_wire_format`. | Test (TC-018) |
| FR-020-AC-9 | The same number as the decoder fixture supplied to the decoder, placed in one of the eight integer members of a valid-operation input, is refused as `invalid_corpus` at the fixture schema check. | Test (TC-018) |
| FR-020-AC-10 | Every canonical file of the published corpus whose object holds one of the eight integer members equals the expected bytes written in the test. | Test (TC-018) |
| FR-020-AC-11 | Every canonical file of the published corpus whose object holds one of the eight integer members spells those members only as strings. | Test (TC-018) |
| FR-020-AC-12 | A published corpus run exits 0 and emits one row per fixture input, with every row reporting `match`. | Test (TC-018) |
| FR-020-AC-13 | The published fixture schema rejects `0`, `1.0`, `"+1"`, `"01"`, `"-0"` and `""` in each of the eight integer members. | Test (TC-018) |
| FR-020-AC-14 | The published package schema rejects a revision and a byte offset of `9007199254740993`. | Test (TC-018) |
| FR-020-AC-15 | A successful `document_json` expression document that the published schema rejects makes the runner exit 2 with the code `invalid_corpus`. | Test (TC-018) |
| FR-020-AC-16 | The root binary uses the authoritative kit's caller-owned exit API for statuses 0, 1 and 2; matched corpus and version invocations exit 0, complete mismatches exit 1, and every existing operational code, including invalid invocation, exits 2. No route requests 5 or implicitly terminates through the kit's pass-through error. | Test |
| FR-020-AC-17 | Shared compact serialization preserves the exact existing result and diagnostic envelopes, serialized field order and one-newline framing; result serialization errors discard the private buffer, while stdout write/flush failure retains exit 2 and `fixture_io` at `stdout`. Existing fallback diagnostics are retained. | Test |
| FR-020-AC-18 | Only the root bridge package optionally depends on `ix-cli-kit`, from its authoritative Git source with the selection in Cargo.lock. Locked Cargo metadata shows no kit in the root's default production closure and shows it in the root's `conformance-cli` closure; the conformance binary requires that feature, the model has no kit dependency, and no kit implementation or alternative argv grammar is copied into the repository. | Inspection, Test (TC-018) |
| FR-020-AC-19 | Published conformance input, schema, and canonical-file bytes remain unchanged. Published expectation files change only by removal of canonical-digest fields, and the generator's byte-for-byte file-regeneration gate retires; all other expected values and comparison behavior remain unchanged. Process tests retain the closed argv rejection cases, exact version line, byte-identical repeated corpus output, and complete five-kind mismatch payload behavior. | Inspection, Test |

## Dependencies

FR-018 defines corpus content. FR-028 preserves the cycle-free model boundary.
`ix://agent-ix/ix-cli-kit/FR-017` owns caller-exit pass-through and reserved-status
handling. `ix://agent-ix/ix-cli-kit/FR-006` owns compact JSON writing.
`ix://agent-ix/ix-cli-kit/FR-003` owns stream discipline and the
`streams::write_primary_stdout` helper for primary-byte delivery. IR retains its existing protocol,
argument parser and operational classifications.
