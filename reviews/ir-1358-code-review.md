---
id: SR-2026110602354603
title: "IR-1358 PR 923 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1358; PR #923; checked_package/v2 model population catalog and public exports; changed Rust tests"
review_set: subset
---

## Summary

Reviewed PR head `2bb5c66eb2600fcac7cf810cf61483b474a75d58` against FR-038-AC-474 through AC-477, FR-019, the retained model index and Rust conventions. The catalog borrows admitted roots and population/member originals, constructs effective type identities under finite limits, and introduces no vendored content or hash/pin mechanism. No applicable AssuranceProfile exists under `spec/`.

## Verdict

**CONDITIONAL** — a retained declaration of a non-object meaning can receive the wrong typed unresolved cause.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `population_member_type_exists` omits retained `other_types`, dimensions and units. A population member naming an admitted `Namespace` or `SystemsPort` declaration therefore returns `MissingType` although a selected declaration of another kind exists; classify every retained known non-object declaration as `WrongKind`. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1049; crates/quire-contract-model/src/checked_package/v2/model_population_catalog.rs:278 |

## Examined scope

- role: examined; id: FR-038-AC-474; excerpt: "An admitted `CheckedPackageV2` exposes a public selected-model population catalog from the model documents it admitted at each selected `model_root`. It returns every selected root in ascending selected identity order, including a root with no populations, and every population of each root in ascending original identity order, including populations with no graph occurrence or application."
- role: examined; id: FR-038-AC-475; excerpt: "Missing, cyclic-alias or wrong-kind member resolution returns a typed error for the entire call, with admitted selected model/root and original population/member locus, never a silently dropped member or successful partial catalog."
- role: examined; id: FR-038-AC-476; excerpt: "The catalog call accepts finite explicit work and output-entry limits, charges selected-root, population, alias-resolution and member visits plus copied effective-identity component bytes as work, and every returned root, population and member descriptor as storage before allocation, and returns a typed work or entry-limit refusal without partial catalog."
- role: examined; id: FR-038-AC-477; excerpt: "The catalog does not use `reaches_population`, `select_population`, source-unused graph inference, or an observation as its source of population facts."
- role: examined; id: FR-019; excerpt: "`checked_package` (V2 selected-model population catalog)"

The PR changes no CI workflow. On disposition head `c4448928e1ed0699f97802486505acccab468b45`, GitHub Supply-chain Checks passed and Rust Checks were pending at review time. The coder reported focused tests, Clippy, formatting and targeted Quire validation passing; the reviewer ran no Cargo. The code lead owns the final gate.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at PR head `c4448928e1ed0699f97802486505acccab468b45`: the retained declaration lookup now includes `other_types`, dimensions, units and the remaining admitted declaration maps. A real reader-admitted Namespace population member returns `WrongKind` in a new value test. |

## Disposition round 1

**PASS for FND-001.** The same selected declaration now receives the correct typed cause; no new defect appears in the focused fix diff.
