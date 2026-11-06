---
id: SR-2026110602354604
title: "IR-1358 PR 923 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1358; PR #923; FR-038-AC-474 through AC-477, TC-466, changed catalog tests and source"
review_set: subset
---

## Summary

The computed Quire matrix marks all four new FR-038 criteria tagged, and the tests exercise real admitted packages and the public accessor. Two claimed properties lack assertions that can fail for the corresponding incorrect implementation. No applicable AssuranceProfile exists under `spec/`.

## Verdict

**CONDITIONAL** — deterministic selected-root order and complete budget charging remain unproved by the new tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The two `model_root` nodes are appended in the same order as their selected identities (`acme/orders`, then `acme/other`), so returning graph order would pass the test despite AC-474 requiring selected-identity order. The assertions also compare only each root's package, allowing a wrong path or ordinal with the same package. Reverse the graph order and compare each complete admitted root identity. | tests/it/checked_package_v2_model_fields.rs:67 |
| FND-002 | medium | The exact work and entry limits are copied from `catalog.work()` and `catalog.entries()` themselves. Removing a population/member/alias charge would lower those counters and still satisfy the exact and one-under checks. Assert independent expected charges for this fixture before deriving boundary limits. | tests/it/checked_package_v2_model_fields.rs:119 |

## Examined scope

- role: examined; id: FR-038-AC-474; excerpt: "It returns every selected root in ascending selected identity order, including a root with no populations, and every population of each root in ascending original identity order, including populations with no graph occurrence or application. Each row carries the selected model identity, the genuine root node identity and the original population identity."
- role: examined; id: FR-038-AC-475; excerpt: "Every catalog population returns its complete member original-identity set in ascending identity order. Each member carries the genuine alias-normalized FR-150 `EffectiveId` of its selected object type, including the complete owner/original-key chain."
- role: examined; id: FR-038-AC-476; excerpt: "Exact limits succeed, one below the first exceeded charge refuses with consumed amount and the selected model/root and original population/member locus; a cloned admitted package gives the same catalog and charge."
- role: examined; id: FR-038-AC-477; excerpt: "A consumer can validate every selected model's named populations before inspecting an observation, with distinct original population identities even where their normalized member type identities match."

Plan completion: not assessed. This review was scoped to the PR diff under the repository's review cadence. `quire matrix --scope . --format tsv` found FR-038-AC-474 through AC-477 tagged; trace binding alone does not establish the two behaviors above.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at PR head `c4448928e1ed0699f97802486505acccab468b45`: graph roots are asserted as `acme/orders` then `aaa/other`, while catalog roots are asserted in the reverse identity order; each complete `CheckedNodeId` is compared with the indexed admitted wire root. |
| FND-002 | fixed | Round 1 at PR head `c4448928e1ed0699f97802486505acccab468b45`: the test independently counts 2 root, 2 population, 3 member and 4 alias visits plus three effective-identity component charges, asserts exact work and seven entries, then uses those expected values for exact and one-under calls at the final member locus. |

## Disposition round 1

**PASS for FND-001 and FND-002.** The new assertions can fail on wrong selected-root order, wrong root node identity, or omitted charges; no new gap appears in the focused fix diff.
