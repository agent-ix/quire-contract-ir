---
id: FR-020
title: "Expose versioned JSON and conformance-runner interfaces"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
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
diagnostics, canonical digest, dependency identities, tool identity, and exit classification.

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
`canonical_bytes`, `canonical_digest`, `dependencies`, and `coverage`. A fixture with several drifts retains all applicable kinds once in
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

The operational classification is exact: malformed, missing, repeated, or
non-UTF-8 arguments map to `invalid_invocation`; a corpus with no fixtures, an
`inputs/` or `expectations/` entry that is not a UTF-8 `.json` file, an
expectation with no input, a fixture name without a known operation, a malformed schema, fixture input or
expectation, a successful package the published schema rejects, and an
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

The code change that lands the v1 decimal-string integers (IR-274 code change
B) amends the published schemas, the corpus and the runner together, because
the runner and the executable-projection binder read the schemas at run time
and a schema that disagrees with the corpus fails the run as `invalid_corpus`.
This requirement states the target; until that change the schemas and corpus
hold the earlier number spelling and are not amended. After it, the published
fixture schema carries the eight integer members of FR-013 (an integer type's
`minimum` and `maximum`, a rational type's `numerator_minimum`,
`numerator_maximum` and `maximum_denominator`, an integer literal's `value`, a
rational literal's `numerator` and `denominator`) as `IntegerString` strings
(`^(0|-?[1-9][0-9]*)$`), and the fixture and package schemas bound a revision
and a byte offset at 2^53 (FR-011, FR-012). Two levels decide an input that
carries a JSON number in one of the eight members, and they do not overlap: a
valid-operation input is checked against the schema first, so it is
`invalid_corpus`; a fixture that exercises the decoder's refusal supplies the
wire to the decoder directly and its expectation carries `invalid_wire_format`,
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
| FR-020-AC-1 | A process test runs the published corpus twice without linking a test harness to the library and obtains byte-identical JSON Lines, one `match` with non-empty observed trace ids per fixture input, exit 0, empty stderr, and complete tool/schema/profile identity. | Test (TC-018) |
| FR-020-AC-2 | Process fixtures pin exit 1 with all six mismatch kinds in fixed order and exit 2 for each of the six closed operational codes (`invalid_invocation`, `invalid_corpus`, `unsupported_profile`, `unsafe_path`, `fixture_io`, `resource_exhausted`), with no absolute path in the error record; stdout/stderr separation, no partial output, `--version`, unknown/repeated arguments, non-UTF-8 argument handling, and pre-decode rejection of a 60000-level referenced JSON input are exact. | Test (TC-018) |
| FR-020-AC-3 | Planned, landing with code change B: the published fixture schema accepts `"-9223372036854775808"`, `"0"` and `"9223372036854775807"` in each of the eight integer members and rejects `0`, `1.0`, `"+1"`, `"01"`, `"-0"` and `""` in each; the package schema accepts a revision and a byte offset of `9007199254740992` and rejects `9007199254740993`; a decoder fixture that supplies a JSON number in one of the eight members straight to the decoder returns `invalid_wire_format` and its expectation carries that code, while the same number in a valid-operation input is `invalid_corpus` at the schema check; and every canonical file of the published corpus whose object holds one of the eight members equals the expected bytes written in the test, with the eight members only as strings, and the published corpus runs exit 0 with every fixture a `match`. | Test (TC-018) |

## Dependencies

FR-018 defines corpus content.
