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

## New findings (disposition pass 2)

Reviewed at 444e675529d80ac4561e1f98a0e3ff1aeb96e475, against quire-specification PR #268 at 33e6aaff080118064e123aab230877634b5a9b27 (open) and quire-spec-language PR #923 at 00f7403ec2cfb10ef7c717ae2cd4cb4b77a6669f (open).

The four cross-repository points the author now states:

1. **Non-NodeId values.** The difference of stage is stated, but the code claim is wrong; see FND-004.
2. **Other member kinds.** Honestly stated: the amendment's schema refuses the key on every other member kind, and this reader does not add that refusal. No #268 vector exercises it.
3. **IR-only refusals.** Mostly honest. The no-`Type`-occurrence refusal does not trip on the #268 vectors: QSpec's own builder (`append_population_chain`) clones the population node from the fixture's `bounded_domain`/`integer_range` node 139f40f80f, whose occurrences are `[{ordinal: 0, role: type}]`. The vectors' `max` values `"3"` and `"0"` are accepted by `population_maximum`. The negative `max` is a code disagreement, not just "the amendment states the negative case only"; see FND-003.
4. **No QSpec vector for the `navigate` refusal.** True: `reaches_population_cases` has no navigate case.

Read as published descriptors, the seven `reaches_population_cases` agree with the IR head:

| Case | Expected outcome |
| --- | --- |
| no key | admitted |
| `max` 3 | admitted |
| `max` 0 | admitted |
| integer-range node | `ill_typed`/`operator-ineligible` |
| absent node | `ill_typed`/`operator-ineligible` |
| another model declaration | `ill_typed`/`operator-ineligible` |
| omitted from dependencies | `invalid_semantic_graph` |

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | AC-279's negative `max` case conflicts with QSpec FR-322. FR-322 on main already has `RM-POP-MAX` (FR-322 :867): a negative `max` on a `model_population` node is refused `invalid_package`/`invalid-derived-shape` at that node. The #268 amendment says "a negative `max` refuses as it does there". IR AC-279 refuses it `ill_typed`/`operator-ineligible` at `member.population`, and the IR prose says only "The amendment states the negative case only". IR has no derived-shape check for `model_population` (`Shape::of` gives none; FR-038-AC-131 is still PLANNED), so this is a real difference of code, pointer and stage, and it is not stated. No #268 vector exercises it. Fix: state the difference honestly (QSpec's derived-shape refusal at the node, owned by FR-038-AC-131, against IR's operation-stage `ill_typed` at `population` until then). Or use the FR-322 code and pointer for the negative case. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3416-3422, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4810, crates/quire-contract-model/src/checked_package/v2/derived_keys.rs:82 |
| FND-004 | low | The rule for a non-node-id `population` says that the amendment's schema stage refuses it and that "this reader ... refuses at its operation stage with the same code". The code is not the same. A V2 schema failure is refused `malformed_wire` (FR-322 :708 states this for a schema failure), while IR refuses `ill_typed`/`operator-ineligible`. The divergence is code and stage, not stage alone. No #268 vector exercises it. Fix: say "with `ill_typed`/`operator-ineligible` where the amendment's schema refuses `malformed_wire`", or adopt `malformed_wire`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3401-3409 |
| FND-005 | medium | The new TC-048 step says to run the published `reaches_population_cases` "through `make conformance-qspec` and assert the same outcomes". Those cases are descriptors, not packages, so IR must build each package itself. QSpec's own builder cannot be copied. It appends the `Reference`, `Set` and population nodes (`append_rekeyed_structural_node`, tests/checked_package_v2.rs:4858-4888) with the template's `{type, 0}` occurrence but adds no `source_map` row. IR's reader requires the source map to equal the occurrence set exactly (common.rs:737-763), so a package built that way is refused `invalid_source_map` before any population check, including in the three admitted cases. As descriptors, IR does not fail the #268 vectors. Built as QSpec builds them, IR would fail every case. Fix: write the IR construction rule into the TC step: one `Type` occurrence and one `source_map` row for each appended node, and reuse an existing `Reference<Order>` instead of appending a duplicate. Raise the missing source-map rows with the QSpec owner on #268. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2381-2382, crates/quire-contract-model/src/checked_package/common.rs:737-763 |

## Dispositions (round 2)

Round 2, reviewed at 444e675529d80ac4561e1f98a0e3ff1aeb96e475.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Gate unchanged. QSpec PR #268 (FR-322-AC-73) and QSL PR #923 are both OPEN. Merge this PR after both, and cite the merged FR-322-AC-73 in place of "that amendment is pending". |
| FND-002 | fixed | For the vector cases the IR codes now match #268. An unresolved, absent, non-population or wrong-`T` `population` is refused `ill_typed`/`operator-ineligible` at `member.population`. `navigate` is refused first. Omitted dependencies are `invalid_semantic_graph`. `max` 0 is admitted. The remaining differences are now stated in the text. The two misstated ones are FND-003 and FND-004; the vector-construction gap is FND-005. |

## Dispositions (round 3)

Round 3, reviewed at ae53240f24cec523e98e83e211a1e0cc20b74425. QSpec PR #268 (33e6aaf) and QSL PR #923 (00f7403ec) are both still OPEN. The PR #439 title now reads AC-269..280. The body no longer carries the IR-841 text: it says "Refs IR-852", has a "Do not merge until" section naming both upstream PRs, and lists six open cross-repo points, each matching the spec text.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The merge gate is unchanged. QSpec #268 and QSL #923 are both OPEN. Merge this PR only after both merge, and cite FR-322-AC-73 in place of "pending". |
| FND-003 | fixed | The negative-`max` disagreement is now stated honestly. The spec says the amendment's FR-322 rule refuses `invalid_package`/`invalid-derived-shape` at the node, that this reader has no derived-shape check for `model_population` (FR-038-AC-131, PLANNED), and that until then it refuses `ill_typed` at `member.population`. It says the two differ in code, pointer and stage, that the amendment's code governs once AC-131 lands, and that no vector exercises the case. AC-279 cross-refers to this. |
| FND-004 | fixed | The spec now says "refuses at its operation stage with ill_typed/operator-ineligible, where the amendment's schema failure is refused malformed_wire. The difference is code and stage." This is accurate (FR-322 :708). |
| FND-005 | fixed | The TC-048 step now requires the IR harness to build each descriptor into a package, giving each appended node exactly one `Type` occurrence and one matching `source_map` row and reusing an existing `Reference` of the object type. The rule is buildable. `validate_source_map_entries` (common.rs:725-825) also needs each row to have a nonempty region (`start < end`) in a locked source, but overlap is checked only within one entry, so the harness can copy a region from an existing row. The QSpec-side construction gap is recorded in the PR body for the #268 owner. |
