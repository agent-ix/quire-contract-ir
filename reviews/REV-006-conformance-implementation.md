---
id: REV-006
title: "Issue #10 schema, corpus, and runner implementation review"
type: Review
status: complete
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
| FND-125 | high | The sole package fixture claimed the whole inventory without observing its constructs, diagnostics, or boundaries. | fixed: targeted fixtures replace the omnibus record; the runner derives observations from each input/result and rejects every unobserved `covers` token, with mutation regressions that inject and detect false claims. |
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
| FND-138 | medium | TC-018 neither used `catch_unwind` over the complete corpus nor asserted per-row mismatch ordering; its set union discarded the order it claimed to verify, and FR-020 also contradicted itself by calling that order sorted. | fixed: linked execution of all 99 fixtures is inside `catch_unwind`, the five-drift row pins registry order exactly, the seven-kind union remains complete, and FR-020 now consistently says unique fixed registry order. |
| FND-139 | medium | Duplicate IDs, unknown and unsorted coverage, fixture-count overflow, and oversized referenced-file guards lacked mutation tests. | fixed: TC-018 mutates each condition independently and pins `invalid_manifest` or `resource_exhausted` before partial output. |
| FND-140 | low | A bare `manifest.json` argument had an empty parent path and failed root canonicalization. | fixed: empty parents normalize to `.` and a process regression runs successfully from the corpus directory with the bare filename. |
| FND-141 | low | Unsafe-path errors echoed attacker-authored absolute path values in the stable error record. | fixed: all preload/path failures expose only closed manifest field names; an `/etc/shadow` probe asserts the value is absent from stderr. |
| FND-142 | low | The semantic-node over-limit fixture added a declaration plus its type and therefore tested 25002 rather than the required 25001. | fixed: the over fixture adds exactly one nested option type to the 25000-node fixture, and TC-018 pins that one-field transformation. |
| FND-143 | high | A deeply nested referenced JSON value could be materialized before depth validation and abort while recursively dropping on the main stack. | fixed: a string-aware raw-byte depth scan rejects nesting before `Value` allocation; manifest, schema, and referenced-input process probes include the reviewer's 60000-level/120 KB case and require the stable `resource_exhausted` exit-2 record. |
| FND-144 | high | Structural boundary claims could survive semantic failure, and a package stale-reference diagnostic could claim an artifact-domain boundary. | fixed: successful boundaries require operation success, invalid boundaries require their owning diagnostic, and artifact boundaries are coverage-operation-only. Mutations reject `revision.current`, `schema.1_1`, and `source_span.minimum` on an identity-rejected package and `artifact.stale` on a package operation. |
| FND-145 | high | `make ci` could pass if the conformance tests were ignored, because it had no direct corpus census. | fixed: `ci` runs a separate `corpus` process census and byte-for-byte `corpus-repro` gate, while the Rust lane uses `--include-ignored`; `check-corpus` is a compatibility alias. Automatic hosted triggers remain disabled. |
| FND-146 | high | Quire's configured `Status` classifier skipped matrices whose archetype requires the header `Coverage Status`, so complete-but-unbacked rows were not classified. | contained locally: renaming the archetype-required header to `Status` was empirically rejected by `quire validate`; `make spec` therefore adds an independent fail-closed matrix census that resolves every completed row to a completed executable Rust or Python test. The upstream Quire contradiction and its reported 84/94 backing metric remain explicit rather than being called a clean coverage pass. |
| FND-147 | high | The observation mechanism that determines whether `covers` claims are truthful had no owning acceptance criterion. | fixed: FR-018-AC-3 specifies the family-specific success/diagnostic/domain rules, bound digests, aggregate budget, and reproducible generation gate; TC-018 traces to it. |
| FND-148 | medium | The 16 MiB per-file cap allowed an authored corpus to retain up to 160 GiB in aggregate. | fixed: manifest and every logical referenced read consume a checked 64 MiB aggregate budget, including repeated paths; a repeated-large-fixture mutation requires `resource_exhausted`. |
| FND-149 | medium | Python discovery silently excluded test modules outside `test_*.py`. | fixed: governance discovers every `tests/*.py` module, function-style unittest adapters enumerate all `test_*` callables instead of maintaining case allowlists, and the matrix census recognizes both module functions and `unittest.TestCase` methods. |
| FND-150 | medium | The corpus generator was unowned, ungated, and hard-coded to an author-local `/tmp` binary. | fixed: FR-018-AC-3 owns the generator, its marker traces to the requirement, runner/output paths are configurable with a portable Cargo-target default, and `corpus-repro` regenerates in scratch space and compares the complete file census and bytes. |
| FND-151 | medium | The reported all-match census could be mistaken for independent semantic correctness even though the implementation freezes its own outputs. | fixed: FR-018, the corpus README, this review, and the PR disclosure distinguish deterministic regression stability from an independently implemented downstream oracle. The current census is 99/99. |
| FND-152 | medium | The published CLI did not authenticate fixture or canonical bytes even though adjacent sidecars existed. | fixed: manifest rows bind each input and expectation digest, canonical records bind raw-byte digests, the runner checks them before execution, and tamper regressions fail through the CLI. Sidecars remain external pin material rather than the trust mechanism. |
| FND-153 | medium | Named Draft 7 schemas and large instances were cloned/recompiled and a thread spawned for every validation. | fixed: the runner compiles all named conformance validators and the package validator once on one long-lived bounded-stack worker, transfers owned instances without deep cloning, and reuses those validators for the full preload. |
| FND-154 | medium | Evidence verification conflated unavailable Git objects with available-but-invalid evidence. | fixed: `EvidenceUnavailable` has exit 3, verification failure has exit 1, FR-009-AC-3 specifies the distinction, and TC-013 pins both paths. |
| FND-155 | low | One coverage fixture remained an omnibus 16-token record. | fixed: eight focused coverage fixtures separately exercise shallow, deep, uncovered, cross-package, missing, stale, digest-mismatch, and duplicate behavior; no coverage row claims more than four tokens. |
| FND-156 | low | `observe_revisions` retained a dead match arm. | fixed: the unreachable arm was removed. |
| FND-157 | low | A glob re-export made the promised conformance API surface implicit. | fixed: `lib.rs` enumerates the stable conformance types, functions, registries, and limits explicitly. |
| FND-158 | low | Semantic-node counting used truncating `usize as u32` casts on unvalidated input. | fixed: all such counts use checked conversion with `u32::MAX` saturation. |
| FND-159 | low | The license allowlist carried three unmatched entries. | fixed: the unused BSD-3-Clause, CDLA-Permissive-2.0, and ISC entries were removed. |
| FND-160 | low | Make target names diverged from corpus-bearing sibling repositories and no `msrv` target existed. | mitigated without rewriting immutable evidence conventions: `check-corpus`, `verify-evidence`, and `msrv` targets now expose the shared names. Historical evidence-schema paths and repo-specific generator layout remain compatibility constraints outside issue #10; cross-repository consolidation belongs to the program tooling work identified by the reviewer. |
| FND-161 | medium | Producer closing audit found the new unavailable-versus-invalid evidence test was absent from a hand-maintained `load_tests` allowlist and therefore had not executed. | fixed: both function-style unittest modules now census every `test_*` callable dynamically; the lane increased from 12 to 13 executed tests and the exit-3/exit-1 regression emits and verifies both classifications. |
| FND-162 | high | The success/diagnostic boundary gate had no mutation regression, so replacing it with unconditional acceptance left the prior candidate green. | fixed: TC-018 now injects structural, wire-depth, and cross-domain artifact claims onto failing/wrong-domain fixtures; each must return `invalid_manifest`, so unconditional acceptance makes the suite fail. |
| FND-163 | high | Successful-package validation against the published schema was real but unguarded; deleting it left the prior candidate green. | fixed: TC-018 tightens the valid Draft 7 package schema with an unsatisfied required property, updates its manifest digest, and requires the runner to reject semantically successful packages before output. |
| FND-164 | high | The public package decoder ignored unknown object members even though both published schemas close every object. | fixed: all package wire structs and tagged enums deny unknown fields; the public API regression and a raw-text corpus fixture require `invalid_wire_format`, matching the published package schema rejection set. |
| FND-165 | high | Public pre-decode wire-depth rejection incorrectly emitted `semantic_input_too_large`, whose registry condition covers only semantic limits. | fixed: public wire syntax/shape/depth failures emit `invalid_wire_format`; STD-001 states the 576-level parser boundary, while the process runner retains its operational `resource_exhausted` classification for manifest/reference preload. |
| FND-166 | medium | The wire-depth limit was private, unstated, and absent from the conformance boundary inventory. | fixed: `MAX_WIRE_JSON_DEPTH = 576` is a stable crate-root export owned by FR-019/FR-020; raw package fixtures cover `wire.depth.maximum` and `wire.depth.over_maximum` at exactly 576/577 without recursively materializing them during preload. |
| FND-167 | medium | Removing `Deserialize` from public `ReferenceBody` silently narrowed the claimed stable surface and deleted a direct-deserialization assertion. | fixed by making the supported boundary explicit: FR-019 excludes serde deserialization impls from v0.1 stability, requires untrusted package JSON through the bounded `ContractPackage::from_json_*` entry points, and retains supported-entry regressions for malformed/unknown members and semantic null-anchor behavior. |
| FND-168 | medium | The semantic package model depended on a JSON depth helper and constant owned by the conformance harness. | fixed: semantic/wire limits and the raw scanner live in a dedicated `limits` module consumed by both identity and conformance layers; the public API no longer calls into the runner module. |
| FND-169 | medium | The operation-success predicate was duplicated in package-schema and coverage-observation gates. | fixed: both gates call one `operation_succeeded` predicate. |
| FND-170 | medium | Rust governance tests hard-coded `python3`, ignored Make's `PYTHON` selection, and discarded the child error when stdout was not JSON. | fixed: the test recipe supplies `QUIRE_GOVERNANCE_PYTHON=$(PYTHON)`, tests honor it with a `python3` fallback, and parse/status failures include child status and stderr. |
| FND-171 | low | Named-schema validation errors exposed a schema definition name instead of the authored manifest field. | fixed: each persistent-worker request carries its exact `fixtures.<id>.input` or expectation field path into the stable error record. |
| FND-172 | low | Linked corpus tests hard-coded a second fixture-count source. | fixed: linked and process tests derive their census from the manifest; the direct Make gate compares the complete ordered result-ID list to the complete authored fixture-ID list. |
| FND-173 | low | The PR body described the earlier 89-fixture/FND-133 candidate after the head had advanced. | fixed at candidate publication: the body reports the 99-row corpus, FND-125 through FND-174 dispositions, exact local gate scope, Quire limitation, unavailable independent closer, and no hosted CI/downstream/cross-platform claim. |
| FND-174 | low | Structural boundary filtering allocated a second owned-string set on every fixture. | fixed: each candidate boundary is result/diagnostic-filtered directly into the final observation set, eliminating the intermediate set and its duplicate strings. |

## Gap analysis

| Required outcome | Authoritative implementation evidence | Result |
|---|---|---|
| Two Draft 7 schemas with fixed identities and digests | root/corpus schemas, manifest digests, adjacent sidecars, bounded schema compilation tests, successful package validation, pinned intentional schema-negative set | satisfied |
| Four declarative operations without fixture-ID dispatch | `ConformanceOperation`, schema-named inputs/expectations, `execute` dispatch solely on operation | satisfied |
| Honest complete inventory | 99 targeted fixtures, family-specific actual-result observation checks, false-claim and cross-domain mutation regressions | satisfied |
| Every diagnostic and obligation | inventory derives from `DiagnosticCode::ALL` and four obligation values; claimed tokens must occur in actual diagnostics | satisfied |
| Every public construct and exact boundary | fixed registries, successful construct observation, outcome/diagnostic-gated exact-edge observation, exact/one-past generated fixtures | satisfied |
| Deterministic comparison and process protocol | twice-run byte equality, seven mismatch kinds, exit 0/1/2 and six operational codes | satisfied |
| Unsafe input and resource resistance | safe relative/symlink containment, stable field-only errors, preload stability, manifest-bound digest checks, pre-decode nesting cap, per-file and aggregate byte/count/depth/node limits | satisfied |
| Portable public interface | explicit profiles, private validated fields/fallible wire conversion, Rust 1.75 locked build, `publish = false` | satisfied |
| CI/publication boundary | workflow retains only `workflow_dispatch`; no workflow dispatch or publication is part of this change | satisfied |

The original issue phrase “positive and negative fixtures” is discharged through
the complete successful construct corpus plus the complete negative diagnostic,
boundary, mutation, and operational corpus. An orphaned classification cannot
itself be a semantically positive case; FR-018 is the reviewed normative
refinement of that issue-level shorthand.

## Closing gate

FND-125 through FND-174 have producer dispositions. Reviewer comment
`5481162405` was written against `37eb001`, and comment `5481528618` reviewed
`db24d90`; their still-applicable findings were reproduced or checked against
the current candidate rather than dismissed as stale. A fresh independent
closing CLI review was attempted twice but was unavailable (`API Error:
ENOTIMP`), so the closing gap audit remains a producer result and found
FND-161. After that remediation, the exact `be548a0` candidate passed the
composite local CI lane with 13 Python tests and 34 Rust tests, the independent
99-row corpus and byte-reproduction lanes, Rust 1.75 compatibility, 67/67 Quire
grammar validation plus the fail-closed local completed-row census, the
optimized 60,000-level depth regression, and `git diff --check`. GitHub Actions
was not dispatched and reported no branch runs or PR checks. The GitHub
reviewer was asked to re-review `be548a0`; no later response or formal review
was present at closure capture. Accordingly, this completed finding record
remains explicitly inconclusive as independent approval. It does not claim
downstream execution, cross-platform CI, or release authority.
