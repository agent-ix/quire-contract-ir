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
| FND-134 | high | Recursion-limit-disabled JSON decode bounded depth only after allocating a recursive value, so rejecting an extremely deep value could abort while recursively dropping it on the main stack. Public `ReferenceBody` also exposed direct derived deserialization without preflight. | fixed: a string-aware iterative nesting scan rejects input before deserialization, every retained recursive value has a fixed syntactic depth cap, deep process/library regressions use `catch_unwind`, and validated `ReferenceBody` no longer implements direct serde deserialization. |
| FND-135 | high | The conformance schema reached recursive compilation without its own depth preflight; a larger thread stack raised the crash threshold but was not a fail-closed control. | fixed: both schemas pass the same pre-decode nesting cap, iterative decoded-depth check, identity check, and complete Draft 7 compilation before the conformance schema validates any instance. Thread-termination details no longer claim stack exhaustion is recoverable. |
| FND-136 | high | The published package schema was compiled but never applied, allowing its package definition and the conformance schema's negative-fixture definition to drift silently. | fixed: every semantically successful package, migration, or coverage operation must also validate against the published package schema; TC-018 pins the complete set of intentional schema-negative fixtures so either schema drift or success-path divergence fails. |
| FND-137 | medium | Structural boundary observation was independent of the actual result, so regenerating expectations after a regression could preserve an exact-boundary claim on a rejected at-limit fixture. | fixed: every valid edge/minimum/maximum/normalization token requires semantic success, while every invalid edge requires its specific actual diagnostic; `collection.maximum` moved to a successful exact-limit fixture. |
| FND-138 | medium | TC-018 neither used `catch_unwind` over the complete corpus nor asserted per-row mismatch ordering; its set union discarded the order it claimed to verify, and FR-020 also contradicted itself by calling that order sorted. | fixed: linked execution of all 89 fixtures is inside `catch_unwind`, the five-drift row pins registry order exactly, the seven-kind union remains complete, and FR-020 now consistently says unique fixed registry order. |
| FND-139 | medium | Duplicate IDs, unknown and unsorted coverage, fixture-count overflow, and oversized referenced-file guards lacked mutation tests. | fixed: TC-018 mutates each condition independently and pins `invalid_manifest` or `resource_exhausted` before partial output. |
| FND-140 | low | A bare `manifest.json` argument had an empty parent path and failed root canonicalization. | fixed: empty parents normalize to `.` and a process regression runs successfully from the corpus directory with the bare filename. |
| FND-141 | low | Unsafe-path errors echoed attacker-authored absolute path values in the stable error record. | fixed: all preload/path failures expose only closed manifest field names; an `/etc/shadow` probe asserts the value is absent from stderr. |
| FND-142 | low | The semantic-node over-limit fixture added a declaration plus its type and therefore tested 25002 rather than the required 25001. | fixed: the over fixture adds exactly one nested option type to the 25000-node fixture, and TC-018 pins that one-field transformation. |

## Gap analysis

| Required outcome | Authoritative implementation evidence | Result |
|---|---|---|
| Two Draft 7 schemas with fixed identities and digests | root/corpus schemas, manifest digests, adjacent sidecars, bounded schema compilation tests, successful package validation, pinned intentional schema-negative set | satisfied |
| Four declarative operations without fixture-ID dispatch | `ConformanceOperation`, schema-named inputs/expectations, `execute` dispatch solely on operation | satisfied |
| Honest complete inventory | 89 targeted fixtures, actual-result observation check, false-claim mutation regression | satisfied |
| Every diagnostic and obligation | inventory derives from `DiagnosticCode::ALL` and four obligation values; claimed tokens must occur in actual diagnostics | satisfied |
| Every public construct and exact boundary | fixed registries, successful construct observation, outcome/diagnostic-gated exact-edge observation, exact/one-past generated fixtures | satisfied |
| Deterministic comparison and process protocol | twice-run byte equality, seven mismatch kinds, exit 0/1/2 and six operational codes | satisfied |
| Unsafe input and resource resistance | safe relative/symlink containment, stable field-only errors, preload stability, digest checks, pre-decode nesting cap, iterative byte/count/depth/node limits | satisfied |
| Portable public interface | explicit profiles, private validated fields/fallible wire conversion, Rust 1.75 locked build, `publish = false` | satisfied |
| CI/publication boundary | workflow retains only `workflow_dispatch`; no workflow dispatch or publication is part of this change | satisfied |

The original issue phrase “positive and negative fixtures” is discharged through
the complete successful construct corpus plus the complete negative diagnostic,
boundary, mutation, and operational corpus. An orphaned classification cannot
itself be a semantically positive case; FR-018 is the reviewed normative
refinement of that issue-level shorthand.

## Closing gate

FND-125 through FND-142 have producer fixes. The complete local CI, strict
specification, Rust 1.75 locked-build, and whitespace gates pass. A fresh
closing independent review of the changed candidate is still required. This
document does not claim independent approval, downstream execution, or release
authority.
