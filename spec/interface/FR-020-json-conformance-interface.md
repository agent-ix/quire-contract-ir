---
id: FR-020
title: "Expose versioned JSON and conformance-runner interfaces"
type: FR
object: interface
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
---
# FR-020: Expose versioned JSON and conformance-runner interfaces

## Contract

```yaml
name: ContractIrJsonConformanceApi
version: quire-contract-ir-v0.1
ownership: quire-contract-ir
inputs:
  - UTF-8 JSON bytes
  - optional fixture manifest path
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
  licensing: MIT OR Apache-2.0
  publication: disabled pending a later human release decision
```

## Description

The repository shall expose a versioned JSON package interface and a
process-level conformance runner whose output is independent of Rust type names
and debug formatting.

## Inputs

UTF-8 JSON bytes, an optional fixture-manifest path, and explicit schema and
canonicalization profile identities.

## Outputs

JSON Lines results containing fixture ID, validity, ordered diagnostics,
canonical digest, dependency identities, tool identity, and exit classification.

## Behavior

The executable name is `quire-contract-conformance`. Its protocol identity is
`quire.contract.conformance-jsonl/v1`. The closed invocation is
`quire-contract-conformance run --manifest <path>` plus `--version`; unknown,
missing, repeated, or non-UTF-8 arguments are invocation failures. The runner
does not search parent directories, environment variables, network locations,
or a default manifest.

For a valid manifest, standard output contains exactly one compact JSON object
and newline per fixture in manifest order. No banner or progress text appears.
Each result contains protocol, corpus ID, fixture ID, operation, closed status
`match` or `mismatch`, unique mismatch kinds in fixed registry order, actual structured result,
and tool identity: crate version, package-schema path/digest, canonical profile,
and runner protocol. Mismatch kinds are `validity`, `diagnostics`,
`canonical_bytes`, `canonical_digest`, `dependencies`, `migration_receipt`, and
`coverage`. A fixture with several drifts retains all applicable kinds once in
this fixed registry order, which is not lexical sorting. Diagnostic messages may be emitted for humans but never
participate in comparison.

Exit 0 means every emitted fixture is `match`. Exit 1 means the manifest and
environment were valid and every fixture ran, but at least one expectation
mismatched. Exit 2 means an invocation, manifest/schema/profile/path/I/O/resource
failure prevented a complete run. Exit 2 emits no standard
output and exactly one compact JSON error plus newline to standard error with
protocol, closed code (`invalid_invocation`, `invalid_manifest`,
`unsupported_profile`, `unsafe_path`, `fixture_io`, or `resource_exhausted`),
and stable path; prose detail is non-comparable. Panics,
partial JSON, and mixed stdout/stderr records are forbidden.

The operational classification is exact: malformed, missing, repeated, or
non-UTF-8 arguments map to `invalid_invocation`; manifest JSON/schema/unknown
field/duplicate ID, schema or inventory digest drift, and malformed fixture
input or expectation map to `invalid_manifest`; unknown package, conformance,
canonical, or protocol identities map to `unsupported_profile`; absolute,
traversing, escaping, or escaping-symlink paths map to `unsafe_path`; missing,
unreadable, non-regular, or changed-after-preload files map to `fixture_io`; and
manifest/file/count/byte/allocation/pre-decode-nesting limits map to
`resource_exhausted`.
The total logical preload budget is 67108864 bytes across the manifest,
schemas, inventory, inputs, expectations, and canonical files; a repeated path
is charged on every authored reference. Raw JSON nesting is scanned before
recursive materialization and is limited to 576 levels.

The runner reads every referenced file before emitting its first result, then
executes fixtures without mutation and buffers every result until the complete
run succeeds or mismatches; any operational failure discards the buffer before
writing the single standard-error record. Error detail, when present, is a
closed normalized phrase with no OS error text or absolute path. Repeating a run
over unchanged bytes is byte-identical. It performs no network access, clock
reads, random generation, absolute-path emission, or host-map-order output.
`--version` prints the crate version and protocol identity to standard output
and exits 0 without reading a manifest.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-020-AC-1 | A process test runs the published corpus twice without linking a test harness to the library and obtains byte-identical JSON Lines, one `match` per authored fixture, exit 0, empty stderr, and complete tool/schema/profile identity. | Test (TC-018) |
| FR-020-AC-2 | Process fixtures pin exit 1 with all seven mismatch kinds in fixed order and exit 2 for all six closed operational codes; stdout/stderr separation, no partial output, `--version`, unknown/repeated arguments, non-UTF-8 argument handling, and pre-decode rejection of a 60000-level referenced JSON input are exact. | Test (TC-018) |

## Dependencies

FR-018 defines corpus content; PGM-01 defines tool and evidence identity.
