---
id: SR-12153
title: "IR-852 gap analysis of IR PR #439"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@edd6bf4fe524637261c7d6a5c0bfbbaa33efe77d (PR #439); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-252, AC-269..278), crates/quire-contract-model/src/checked_package/{selected_population.rs,common.rs}, crates/quire-contract-model/src/checked_package/v2/{operations.rs,structural.rs,mod.rs,model_members.rs}; producer agent-ix/quire-spec-language branch qsl819-population-bound (FR-365, FR-380, FR-104); agent-ix/quire-specification main FR-322"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-852 gap analysis of IR PR #439

## Summary

Ticket: IR-852. This is a planless gap analysis. The PR is spec-only, so there is
no production code to match. The analysis checks the computed matrix, the spec's
claims about origin/main code, and the cross-repository chain the key depends on.

- **Matrix.** `quire matrix --scope . --format json` lists FR-038-AC-269..278, each
  untagged. This is expected: each row is PLANNED / UNRUN and no code exists. No
  existing tagged criterion changed status.
- **No code claim is false at origin/main.** No `reaches_population` or
  `CheckedReachesPopulationError` symbol exists. `operation.member` is opaque
  (operations.rs:199-207). `application_dependencies` adds only reference targets
  and the member `declaration` (structural.rs:652-695). The application key set is
  closed (common.rs:1003-1008).
- **Producer agreement (QSL branch `qsl819-population-bound`, commit ee77bb9ee).**
  - FR-365 "The population link" writes `population: <id>` in `operation.member`,
    absent rather than `null` when the population is unbounded.
  - It names the least-ordinal `Type` occurrence, adds the population id to
    `dependencies`, and gives one link per reachability (several per claim).
  - FR-380 emits the `model_population` node over `Population<E>[N]`, with E the
    near-end type.
  - FR-365-AC-6 and AC-13 are amended to match.
  - All of this agrees with the IR section.
- **Consumer fit.** An application-keyed accessor that returns FR-053's selector
  matches the per-reachability link.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038-AC-269..278 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | PLANNED / UNRUN (IR-852) |
| The link (producer clause) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The key is produced by the QSL lowering of a reachability over a bounded population and admitted by the QSpec checked-package application rule (FR-322), which this requirement follows; until that rule admits the key, no package carries it. |
| FR-038-AC-252 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | its operation.member holds a population only through the optional population key |
| QSL FR-365 The population link | context_only | quire-spec-language@qsl819-population-bound spec/functional/FR-365-reach-an-object-over-a-relationship-end-in-a-state-clause.md | S4 SHALL write population: <id> in the application's operation.member |
| QSL FR-380 | context_only | quire-spec-language@qsl819-population-bound spec/functional/FR-380-bound-a-domain-package-population-and-link-a-reachability-to-its-node.md | QSpec FR-153 (the population declaration), FR-322 (the population member key of a quire.op.model.reaches application). |
| QSpec FR-322 member rule | context_only | quire-specification@origin/main spec/objects/interfaces/FR-322-checked-package-artifact.md | The wire member names the relationship node as its declaration and the end's role as its name |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The IR section says the key is "admitted by the QSpec checked-package application rule (FR-322), which this requirement follows". No such rule exists yet. QSpec FR-322 on quire-specification main defines `relationship_end` as `declaration` plus `name` and says nothing of `population` (FR-322 :257-275). No quire-specification branch carries the amendment. QSL FR-380 and FR-365 defer the key to "QSpec FR-322 (the `population` member key ...)". The QSL-819 spec exists only as branch `qsl819-population-bound` with no PR. So the three-repo chain has its keystone missing: the upstream contract both specs cite. If IR merged first, the IR reader would admit a key that no upstream contract defines. Fix (gating, not wording): hold the merge until the QSL-819 spec and the QSpec FR-322 amendment land. Then cite the merged FR-322 criterion id instead of "which this requirement follows". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3325-3329 |

## Verdict

CONDITIONAL. Read for this spec-only scope: the ten untagged criteria are declared
PLANNED / UNRUN and are the code PR's obligation. The mechanical
untagged-criterion FAIL applies when that code PR lands. The one finding is a
cross-repository gating gap, not a defect in the IR text's design.

## Coverage

- Criteria in scope: 10 new (AC-269..278) and 1 reworded (AC-252). Tagged: 0 of 10
  new (all PLANNED). AC-252 keeps its existing IR-824 tags.
- Reverse gap: none. The PR adds no code.
- Stubs / coverage inflation: none. There are no tests.
- Semantic review: skipped (spec-only PR; not requested).
- Plan completion: not assessed

## New findings (disposition pass 1)

Reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce, against the upstream amendment that now exists: quire-specification PR #268 (open, branch `qsl-819-population-bound`, FR-322 "Population link of a `reaches`", FR-322-AC-73, `adverse.json` `reaches_population_cases`), and quire-spec-language PR #923 (open, branch `qsl819-population-bound`, 00f7403ec).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The IR reader check disagrees with the QSpec FR-322 amendment it says it follows. (1) QSpec #268 refuses a `population` "that does not resolve" with `ill_typed`/`operator-ineligible`, and `adverse.json` records `reaches-population-names-absent-node` with exactly that code. IR AC-272 refuses an absent node `missing_declaration`/`missing-selection`. (2) QSpec's schema types `population` as `NodeId` inside a closed `relationship_end` member, so a non-NodeId value such as `null` is a schema refusal. IR AC-272 refuses it `missing_declaration`/`missing-selection`. (3) QSpec has the schema refuse `population` "on any other member kind"; IR AC-275 leaves other operations at their present outcome, which is admission for the opaque kinds. (4) IR also refuses a population node with no `Type` occurrence and a `max` that is missing, duplicated, not a binding or not an integer literal; QSpec names only the negative-`max` refusal and admits `max` 0. Those are extra IR refusals the contract does not state. With both PRs merged as they are, IR's reader would fail the published `reaches_population_cases` vector. Fix: settle the codes with the QSpec owner before either PR merges (on the merits, either both mirror the `declaration` precedent or both use `ill_typed`), then cite FR-322-AC-73 and run `reaches_population_cases` under `make conformance-qspec` in the code PR. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3383-3413, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4778, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4781 |

## Dispositions

Round 1, reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The wording part is fixed: the link now says "that amendment is pending ... this requirement does not claim that the key is already admitted by FR-322". The gate remains. The QSpec amendment now exists as quire-specification PR #268, open and not merged, and the QSL-819 spec is PR #923, open and not merged. Merge this PR only after both land. Then cite the merged FR-322-AC-73 instead of "pending". The two specs also disagree; see FND-002. |
