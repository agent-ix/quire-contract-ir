---
id: SR-1472
title: "scope-boundary review of PR 291 (typed STD-001 code, FR-044)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@c109f7eaba2d15f2d8d526f0b0f9dfdfc240a0dd; git diff origin/main...HEAD (base 6fb6e974efd6b9a9c74515ee7e07df250fa4aacf): FR-044 Home and Dependencies, STD-001 cause-code scope note, AD-006 seam bullet and Decision C; checked against quire-contract-codegen origin/main f3ece43 (Cargo.toml, src/kani, spec) and quire-spec-language Cargo.toml"
review_set: subset
---
# SR-1472: scope-boundary review of PR 291

## Summary

Ticket: IR-605. Ownership and repository boundaries of the new type and its
codes.

Measured: the type's home in `quire-contract-model` with no root re-export
fits FR-039 and AD-006 Decision A. Codegen already declares
`quire-contract-model` directly (`Cargo.toml:21`), and quire-spec-language's
workspace manifest names it, so "adds no edge" holds. The type depends on no
QSL, codegen or runtime type. Keeping the lowering codes out of STD-001 matches
IR-347's move of the lowerings to codegen. The ticket text (untrusted, IR-605
description) listed `kani_dispatch_unowned` among IR's registry codes; the
PR's exclusion is the one consistent with IR-347, and the ownership gap that
leaves is SR-1470 FND-001.

Examined: FR-044 Home, Registered codes and Dependencies; STD-001 note after
the cause table; AD-006 seam bullet and Decision C.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AD-006 and STD-001 state codegen facts that do not hold at codegen origin/main f3ece43. The rewritten AD-006 bullet keeps the citation `src/kani_execution.rs:690`; that file does not exist and the comparison is at `src/kani/classify.rs:225`. Decision C and STD-001 say codegen's codes are "registered in codegen's registry", but codegen's spec has no registry document. Both name only `kani_corpus_*` as codegen's own codes, while codegen also mints `kani_profile_input_mismatch` into `KaniOutcome` (`bounded_kani_corpus.rs:403`). State the registry as codegen's target (with its ticket) and fix the citation and the list | spec/assurance/AD-006-codegen-consumption-seam.md:91-96, 127-130; spec/core/functional/STD-001-diagnostic-registry.md:133-138 |

## Verdict

Boundaries sound; one low finding on stale or unsupported cross-repo claims.

## Dispositions

Round 1, reviewed at cace2678410ee69ae71e656e26f80541ba2e31e4.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | The registry claim and the code list are fixed, and the path is now right (`src/kani/classify.rs`), but the new citation names the function `classify_run`, which does not exist at codegen origin/main f3ece43; the comparison at `classify.rs:225` is inside `classify_success`. Name `classify_success` or drop the function name |

Round 2, reviewed at cf90862eda642d465b90d4cc13fa3f51bfbade57.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | cf90862 |
