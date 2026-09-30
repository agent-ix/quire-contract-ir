---
id: SR-601
title: "spec review of PR 221 (schema 1.1 only, no registered migration)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@c66dc211eae993347ec39dadf7fc398ee6ad42bf; spec/contract/FR-016-canonicalization-digests.md, spec/contract/FR-017-version-orphan-coverage.md, spec/contract/FR-018-conformance-corpus.md, spec/contract/STD-001-diagnostic-registry.md, spec/interface/FR-019-rust-library-interface.md, spec/interface/FR-020-json-conformance-interface.md, spec/nonfunctional/NFR-003-diagnostic-integrity.md, spec/stakeholder/StR-002-portable-interchange.md, spec/contract-test-matrix.md, spec/contract/FR-011-package-identity.md (context)"
review_set: base
---
# SR-601: spec review of PR 221

## Summary

Ticket: IR-362. Reviews the spec-only diff of PR 221 against origin/main (7caa62c). Owner decision: schema 1.1 is the only supported version, every other version fails `unsupported_schema_version`, no migration, no 1.0 reader. Spec/code disagreement on migration items is expected (code follows in a separate PR) and is not flagged here. Checked: residual migration / 1.0 / `migration_receipt` / `unregistered_migration` text, cross-document consistency of FR-016..FR-020, STD-001, NFR-003, StR-002 and the matrix, the boundary-token rename, and AC testability. `make spec`: validate 243/243 grammar-clean; coverage 22 unbacked rows, the same set as origin/main, none new.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-018 still says four fixture operations / four-operation enum; the migration row was removed, leaving three. | spec/contract/FR-018-conformance-corpus.md:48 |
| FND-002 | high | FR-020-AC-2 requires "all seven mismatch kinds"; FR-020 now registers six, so the AC cannot pass. | spec/interface/FR-020-json-conformance-interface.md:114 |
| FND-003 | medium | FR-017-AC-1 says every other major is rejected with `unsupported_schema_version`; major 0 is `invalid_schema_version` per STD-001 and FR-017 Behavior. | spec/contract/FR-017-version-orphan-coverage.md:94 |
| FND-004 | medium | FR-017-AC-1 does not name 1.0, the removed legacy version, as a required rejection case; a test probing only 1.2 and 2.0 passes while 1.0 is still read. | spec/contract/FR-017-version-orphan-coverage.md:94 |
| FND-005 | medium | No requirement says what constructing or canonicalizing an in-memory package with a non-1.1 SchemaVersion returns; STD-001 limits `unsupported_schema_version` to wire preflight. | spec/contract/FR-016-canonicalization-digests.md:73-77 |
| FND-006 | medium | "Schema minor zero is valid" survives in STD-001 and FR-011-AC-3 and reads as a 1.0 support path. | spec/contract/STD-001-diagnostic-registry.md:34 |
| FND-007 | low | FR-016 claims digests change with schema version, which cannot be exercised when only 1.1 is canonicalizable. | spec/contract/FR-016-canonicalization-digests.md:52-55 |
| FND-008 | low | `unsupported_schema_version` location changed from `schema_version.major` to "`schema_version` path" without saying whether sub-paths are used. | spec/contract/STD-001-diagnostic-registry.md:82 |

## Finding Detail

**FND-001** (high, confidence high, soundness; spec/contract/FR-018-conformance-corpus.md:48, 59, 88, 98): "one of the four operations" (48), "The four closed fixture operations are:" (59) followed by a three-row table, "the four-operation enum" (88, 98). The inventory in FR-018-AC-1 is derived from that enum, so the operation-token count is contradictory. Fix: "three" at 48 and 59, "three-operation enum" at 88 and 98.

**FND-002** (high, confidence high, untestable-ac; spec/interface/FR-020-json-conformance-interface.md:114): FR-020:62-63 now lists six kinds (`validity`, `diagnostics`, `canonical_bytes`, `canonical_digest`, `dependencies`, `coverage`); AC-2 still says "all seven mismatch kinds in fixed order". Fix: "all six mismatch kinds".

**FND-003** (medium, confidence high, ambiguous; spec/contract/FR-017-version-orphan-coverage.md:94): "rejects every other major or minor with `unsupported_schema_version`" includes major 0, which STD-001:34, FR-011-AC-3, FR-017:31-32 and the `schema.zero_major` boundary give `invalid_schema_version`. Fix: "accepts schema 1.1 and rejects every other well-formed version (nonzero major) with `unsupported_schema_version` before semantic interpretation; a zero major keeps `invalid_schema_version`."

**FND-004** (medium, confidence medium, untestable-ac; spec/contract/FR-017-version-orphan-coverage.md:94, spec/contract/FR-018-conformance-corpus.md:113-116): "every other ... minor" cannot be tested exhaustively, and the single `schema.unsupported_minor` token is satisfied by one fixture. The regression this change exists to prevent is 1.0 still being accepted. Fix: name the boundary cases in AC-1 ("including 1.0, 1.2 and 2.0"), or split the token so 1.0 has its own fixture.

**FND-005** (medium, confidence high, coverage; spec/contract/FR-016-canonicalization-digests.md:73-77, spec/contract/STD-001-diagnostic-registry.md:82): FR-016 says canonicalization is "defined only for ... schema version 1.1", but SchemaVersion accepts any nonzero major (FR-011) and `ContractPackage::new` is a public checked constructor, so a 1.0 or 1.9 package can reach the canonical APIs without wire preflight. No requirement gives the result, and STD-001's condition is limited to "Wire preflight reads ...". Today's code returns `unregistered_migration` there; the follow-up has no spec for the replacement. Fix: state that `ContractPackage::new` (or every canonical API) rejects any version other than 1.1 with `unsupported_schema_version` at `schema_version`, and widen the STD-001 condition to cover that path.

**FND-006** (medium, confidence medium, soundness; spec/contract/STD-001-diagnostic-registry.md:34, spec/contract/FR-011-package-identity.md:48): "Zero schema major; schema minor zero is valid" and FR-011-AC-3 "Schema minor zero is valid" were written for the 1.0 source path. A reader now takes them to mean a 1.0 package is valid, which contradicts FR-017. IR-362 also names FR-011 as in scope. Fix: "schema minor zero is well-formed; FR-017 refuses every version other than 1.1 as unsupported", or drop the clause.

**FND-007** (low, confidence high, untestable-ac; spec/contract/FR-016-canonicalization-digests.md:52-55): "so a package's canonical bytes and digest change with its schema version even when every other semantic field is preserved" needs two canonicalizable versions to test. Fix: "so the schema version is part of the package digest input."

**FND-008** (low, confidence medium, ambiguous; spec/contract/STD-001-diagnostic-registry.md:82): the old row required `schema_version.major` for a major mismatch; the new row says "`schema_version` path". Code emits `schema_version.major` for majors and `schema_version` for minors. Fix: state the exact path, for example "`schema_version` (object path) for every unsupported version".

## Scope

- `FR-016` (spec/contract/FR-016-canonicalization-digests.md), examined: The package semantic value includes `schema_version` ... so a package's canonical bytes and digest change with its schema version ... Canonicalization is defined only for values already accepted by FR-011 through FR-015 and for schema version 1.1. It performs no repair, migration, or best-effort interpretation.
- `FR-017` (spec/contract/FR-017-version-orphan-coverage.md), examined: Schema 1.1 is the only supported version. The library registers no schema migration; a later migration is introduced by its own requirement. ... Any version other than 1.1 fails `unsupported_schema_version`.
- `FR-017-AC-1` (spec/contract/FR-017-version-orphan-coverage.md), examined: Version preflight accepts schema 1.1 and rejects every other major or minor with `unsupported_schema_version` before semantic interpretation.
- `FR-017-AC-2` (spec/contract/FR-017-version-orphan-coverage.md), examined: Shallow, deep, uncovered, and each closed orphan reason have positive/negative fixtures; ...
- `FR-018` (spec/contract/FR-018-conformance-corpus.md), examined: package schema for the supported schema version 1.1; subschemas `packageInput`, `expressionInput`, `coverageInput`; three-row operation table; boundary registry with `schema.1_1`, `schema.zero_major`, `schema.unknown_major`, `schema.unsupported_minor`.
- `FR-018-AC-1` (spec/contract/FR-018-conformance-corpus.md), examined: Running the corpus directory yields one matching row per input ... union of observed tokens equals the published inventory ...
- `FR-018-AC-2` (spec/contract/FR-018-conformance-corpus.md), examined: Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing.
- `STD-001` (spec/contract/STD-001-diagnostic-registry.md), examined: `unsupported_schema_version` | Wire preflight reads a valid schema version other than 1.1 | `schema_version` path; no semantic span. Wire precedence is JSON/top-level structure, schema-version numeric grammar, unsupported version, then semantic package interpretation.
- `FR-019` (spec/interface/FR-019-rust-library-interface.md), examined: construction, validation, dependency derivation, canonicalization, digest, and coverage-classification operations; canonical module row without `migrate_reference_body` / `MigrationReceipt`.
- `FR-019-AC-1` (spec/interface/FR-019-rust-library-interface.md), examined: ... every conversion is fallible, canonical profiles are explicit, ...
- `FR-019-AC-2` (spec/interface/FR-019-rust-library-interface.md), examined: The complete negative corpus executes package/expression decode, validation, canonicalization, and coverage through `catch_unwind`; ...
- `FR-020` (spec/interface/FR-020-json-conformance-interface.md), examined: Mismatch kinds are `validity`, `diagnostics`, `canonical_bytes`, `canonical_digest`, `dependencies`, and `coverage`.
- `FR-020-AC-2` (spec/interface/FR-020-json-conformance-interface.md), examined: Process fixtures pin exit 1 with all seven mismatch kinds in fixed order and exit 2 for each of the six closed operational codes ...
- `NFR-003` (spec/nonfunctional/NFR-003-diagnostic-integrity.md), examined: Parsing, validation, dependency derivation, canonicalization, coverage classification, and the conformance runner.
- `StR-002-VC-2` (spec/stakeholder/StR-002-portable-interchange.md), examined: Unsupported schema versions produce structured rejection.
- `TC-017` (spec/contract-test-matrix.md), examined: Canonical bytes, digests, version preflight, and orphan classes conform.
- `FR-011-AC-3` (spec/contract/FR-011-package-identity.md), context_only: ... a zero schema major ... fail with their registered structured diagnostic codes. Schema minor zero is valid.

## Verdict

The migration operation, `migrationInput`, `migration_receipt`, `unregistered_migration`, `migrate_reference_body` and `MigrationReceipt` are gone from all live spec text; the only remaining "migration" words are the intended "no migration" statements (FR-016:74, FR-017:27-28) and an unrelated ADR-0053 line. The boundary rename is consistent: `schema.1_0` and `schema.unregistered_minor` appear nowhere in spec, `schema.unsupported_minor` appears only in the FR-018 registry, and no matrix row or TC file names schema boundary tokens. NFR-003, StR-002-VC-2, FR-019 and the TC-017 matrix row are consistent. Two count mismatches left behind by the removal (FND-001, FND-002) make FR-018 contradictory and FR-020-AC-2 impossible to pass, and FR-017-AC-1 needs its zero-major case and its 1.0 case made precise. Request changes.
