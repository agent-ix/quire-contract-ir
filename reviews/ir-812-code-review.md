---
id: SR-2026110283807
title: "IR-812 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-812; Cargo.toml, Cargo.lock, tests/it/source_obligation_quire.rs, tests/it/main.rs, FR-052 status and TC-452 status"
review_set: subset
---

## Summary

Ticket: IR-812. Independent scoped review of native Quire consumer qualification for FR-052-AC-16. No applicable AssuranceProfile exists under spec/. No source edits, Cargo gates, backend or CI operations were run. Rust review is folded into code-review.

## Verdict

FAIL. Dependency policy integration must be fixed before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new quire-rs Git dependency and its filament-core-data transitive Git source are absent from deny.toml allow-git while unknown-git is deny; the required make deny / CI policy gate therefore rejects this dependency graph. | Cargo.toml:104; deny.toml:44 |

## Coverage

Plan completion: not assessed

FR-052-AC-16 (role: examined), spec/model/functional/FR-052-source-obligation-correspondence.md:

With PLAT-1165's real Quire producer, normal exhaustion of the validated selected scope issues `Complete` with every authored criterion and its provenance; a valid empty scope completes with zero rows. Changing or deleting one authored criterion in the selected Quire corpus changes the owner-issued inventory before generation or proof results, including when that criterion has zero mappings. An unreadable selected document, skipped criterion row, failed discovery, missing declared table/ID column, and early stop each report typed `Incomplete` with emitted partial rows, and deletion of a row from full owner output fails comparison even when the shortened count matches. Until this producer exists, no Quire `Complete` or full FR-052 census is claimed.

FR-052 status paragraphs and TC-452 status (role: examined): AC-16 is Implemented by native Quire consumer controls; AC-7, AC-9 through AC-11 and the four-owner census remain Planned/UNRUN. No acceptance text, requirement behavior, owner allocation, or public API changed.

Inspected all seven native tests and their trace attributes. The computed matrix reports AC-16 tagged by exactly those seven tests. Full comparison checks native selection/identity, entire obligation, row/line loci and native count; removing a row with matching shortened count fails. Authored identity conversion borrows native AuthoredRow and rejects RenderedRow/CombinatorialTable exhaustively. Native causes and original interruption/walk errors remain borrowed typed owner data. Native model validation and selected document enumeration are used directly.

Controls cover unmapped criteria, edits and both row deletions, valid empty corpus, missing root, missing section/table/ID column, unusable row, unreadable UTF-8 and permission-denied document, actual failed directory discovery with healthy partial rows, stop boundaries 0/1/2/3 and interruption, duplicate ambiguity and foreign selection. Permission tests require a Unix process without permission-bypass privilege; the supplied focused run executed all seven tests without skips. No parser, schema, native inventory, enum or adapter implementation was copied. The root dependency is dev-only with default features disabled; the model and production API remain unchanged.

Read supplied workspace all-targets/conformance-cli check and workspace lib/it Clippy logs and the seven-test passing log. Those logs support focused behavior only; the final local and hosted gates remain the lead's responsibility. Spec validation is performed independently. Additional spec analyses are inapplicable because only status narrative changed. Semantic inspection was explicitly requested for AC-16; no broad semantic audit ran.
