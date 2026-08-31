---
id: REV-006
title: "Issue #10 schema, corpus, and runner implementation review"
type: Review
status: in_progress
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/10
    type: reviews
---
# REV-006: Issue #10 conformance implementation review

## Scope and independence boundary

This review covers the issue #10 implementation against FR-018 through FR-020,
STD-001, NFR-001 through NFR-003, and TC-018. The independent reviewer used
read-only static inspection and did not run tests or edit files. Local gate
results are producer claims, not independent approval. Automatic CI and crate
publication remain outside scope and disabled.

## Findings

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-125 | high | The sole package fixture claimed the whole inventory without observing its constructs, diagnostics, or boundaries. | fixed: 89 targeted fixtures replace the omnibus record; the runner derives observations from each input/result and rejects every unobserved `covers` token, with a mutation regression that injects and detects a false claim. |
| FND-126 | high | Plain package JSON deserialization's depth-128 guard preempted the public depth-256 contract. | fixed: package version and wire decoding now use the same stack-growing, recursion-limit-disabled deserializer as conformance JSON; semantic preflight remains authoritative. |
| FND-127 | high | Recursive schema validation and expression conversion ran before bounded-depth preflight. | fixed: every schema instance receives an iterative JSON-depth scan capped above the exact semantic boundary before recursive schema traversal; expression conversion therefore sees only bounded input. |
| FND-128 | medium | The manifest lacked the before/after metadata and post-read size checks used for referenced files. | fixed: manifest reads now compare type, length, and modification metadata before/after and recheck actual bytes against the cap. |
| FND-129 | high | `orphaned_clause_reference` and canonical resource exhaustion were unreachable through the four declared fixture operations. | fixed: the package operation accepts a schema-pinned optional probe wrapper containing clause resolutions and a canonical byte limit; behavior remains declarative and never depends on fixture ID. |
| FND-130 | high | The 100000-node semantic edge could not be represented reliably below the 16 MiB per-file limit, so its required exact boundary fixture was not constructible. | fixed: the registry limit is 25000 nodes, still above the 10000-entry collection edge while remaining representable below the wire byte cap. |
| FND-131 | high | Recursive validation/expression conversion overflowed the default stack for the required legal depth-256 corpus fixture. | fixed: bounded preflight remains first, while recursive schema validation and fixture execution run on explicitly bounded 16 MiB stacks; exact-depth and over-depth corpus fixtures and tests now complete without process failure. |
| FND-132 | high | Semantic preflight omitted value types embedded in expression literal nodes, allowing their depth and node count to bypass the public limits. | fixed: expression traversal now contributes every embedded integer, rational, option, and collection value type to the same iterative type-depth and global-node preflight; TC-018 includes an over-depth embedded-type process regression. |
| FND-133 | medium | Referenced-file preload checked metadata size before reading but did not independently recheck the actual byte count or require it to equal the stable metadata snapshot. | fixed: manifest and referenced-file preload now compare actual bytes with before/after length and retain the explicit post-read byte cap. |

## Gap analysis

| Required outcome | Authoritative implementation evidence | Result |
|---|---|---|
| Two Draft 7 schemas with fixed identities and digests | root/corpus schemas, manifest digests, adjacent sidecars, schema compilation tests | satisfied |
| Four declarative operations without fixture-ID dispatch | `ConformanceOperation`, schema-named inputs/expectations, `execute` dispatch solely on operation | satisfied |
| Honest complete inventory | 89 targeted fixtures, actual-result observation check, false-claim mutation regression | satisfied |
| Every diagnostic and obligation | inventory derives from `DiagnosticCode::ALL` and four obligation values; claimed tokens must occur in actual diagnostics | satisfied |
| Every public construct and exact boundary | fixed registries, successful construct observation, structural exact-edge observation, exact/one-past generated fixtures | satisfied |
| Deterministic comparison and process protocol | twice-run byte equality, seven mismatch kinds, exit 0/1/2 and six operational codes | satisfied |
| Unsafe input and resource resistance | safe relative/symlink containment, preload stability, digest checks, byte/count/depth/node limits, bounded recursive stacks | satisfied |
| Portable public interface | explicit profiles, private validated fields/fallible wire conversion, Rust 1.75 locked build, `publish = false` | satisfied |
| CI/publication boundary | workflow retains only `workflow_dispatch`; no workflow dispatch or publication is part of this change | satisfied |

The original issue phrase “positive and negative fixtures” is discharged through
the complete successful construct corpus plus the complete negative diagnostic,
boundary, mutation, and operational corpus. An orphaned classification cannot
itself be a semantically positive case; FR-018 is the reviewed normative
refinement of that issue-level shorthand.

## Closing gate

FND-125 through FND-133 have producer fixes and the full local gate passes. A
closing independent static review is still required. This document does not
claim independent approval, downstream execution, or release authority.
