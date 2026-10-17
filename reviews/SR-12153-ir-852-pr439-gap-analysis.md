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

## New findings (disposition pass 4)

Reviewed at 76c8df6a932662e51653c4b4f76127981bf8b795, against quire-specification origin/main 6f6136a6c (PR #268 merged) and quire-spec-language origin/main dc845c3e8 (PR #923 merged; FR-365/FR-380 unchanged since 00f7403ec).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | Two of the new citations attribute to FR-322-AC-73 statements it does not make. FR-322-AC-73 (FR-322 :1523) lists the schema validation, the preimage, the dependency, admission for `max` 3 and 0, and the three `ill_typed` refusals and the `invalid_semantic_graph` refusal. It says nothing about a negative `max` or about other member kinds. Those statements are in FR-322's prose section "Population link of a `reaches`" (:1096-1098 and :1109-1110), and the negative-`max` rule itself is RM-POP-MAX (:869, FR-322-AC-62). (1) "For a negative `max` FR-322-AC-73 refers to the model-population `max` rule of FR-322" (FR-038 :3425): AC-73 does not refer to it. (2) "FR-322-AC-73 has the schema refuse the key on every other member kind" (:3448): AC-73 does not say that. The schemas do refuse it, because only `relationship_end` declares `population` and every member variant is `additionalProperties: false`. (3) Also, "states the key, its checks and their order in FR-322-AC-73" (:3329-3330): the order ("follows the destination-type check of FR-322-AC-57") is in the section, not in AC-73. The behaviour the IR text describes is right; only the pointer is off. A reader who follows the citation to AC-73 will not find the rule. Fix: in (1) and (2), cite FR-322's "Population link of a `reaches`" rule (and RM-POP-MAX / FR-322-AC-62 for the negative `max`), not AC-73. In (3), say the section states the key, checks and order and that FR-322-AC-73 tests them. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3329-3331, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3425-3427, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3448-3449 |

## Dispositions (round 4)

Round 4, reviewed at 76c8df6a932662e51653c4b4f76127981bf8b795. QSpec PR #268 is MERGED (6f6136a6c, on quire-specification origin/main). QSL PR #923 is MERGED (dc845c3e8, on quire-spec-language origin/main).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Both upstream PRs are merged, so the gate is met. FR-322 on main now has the "Population link of a `reaches`" rule (:1088-1114) and FR-322-AC-73 (:1523). Both V2 schemas give `relationship_end` an optional `population` typed `NodeId` (schema.json :127, node-identity-preimage.schema.json :51). `adverse.json` publishes the seven `reaches_population_cases`. They match the IR outcomes recorded in round 2. FR-038 now cites FR-322-AC-73 in place of "pending". Its claims of code, stage and agreement hold against main. One citation is imprecise; that is FND-006. |

## New findings (disposition pass 5)

Reviewed at 20a4eaf2fda35792c51e613505fc40ca5b3d3d4e, against quire-specification origin/main 6f6136a6c.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | The round-5 edit widened the general rule of "The link" so that it now overclaims. It used to say "Where FR-322-AC-73 states a refusal". It now says "Where FR-322 states a refusal, the reader check below follows its code, cause and pointer; a difference of stage ... is stated where it arises". Every refusal FR-322-AC-73 states does match the IR codes. But FR-322 as a whole also states two refusals whose code differs from IR's, and the bullets below say so: (1) a non-node-id `population` fails the schema and is refused `malformed_wire`, where IR refuses `ill_typed` ("The difference is code and stage", :3412); (2) a negative `max` is refused `invalid_package`/`invalid-derived-shape` at the node, where IR refuses `ill_typed` at `member.population` ("differ in code, pointer and stage", :3431). So the general sentence ("follows its code, cause and pointer"; only a "difference of stage" is noted) contradicts two of the bullets it introduces. Fix: either go back to "Where FR-322-AC-73 states a refusal", or keep "FR-322" and say "a difference of code, pointer or stage ... is stated where it arises". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3330-3334, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3407-3412, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3424-3433 |

## Dispositions (round 5)

Round 5, reviewed at 20a4eaf2fda35792c51e613505fc40ca5b3d3d4e.

| FND | outcome | reason |
| --- | --- | --- |
| FND-006 | fixed | All three citations now point where the text actually is. (1) The negative-`max` rule: "FR-322's section "Population link of a `reaches`" refers to the model-population `max` rule (RM-POP-MAX, FR-322-AC-62)". This matches FR-322 :1109-1110, :869 and :1512. (2) The refusal on other member kinds: "FR-322's section "Population link of a `reaches`" has the schema refuse the key on every other member kind" (FR-322 :1096-1098). (3) "states the key and its checks in FR-322-AC-73, and their order in its section". AC-73 (:1523) states the key and its refusals; the section (:1105-1106) states the order. The widened refusal sentence in the same edit is FND-007. |

## New findings (disposition pass 6)

Reviewed at e5e582aaca6bedc492066f406cba2217d2e105a6, against quire-specification origin/main 6f6136a6c. I checked the widened refusal sentence against every rule it introduces. These rules match FR-322 in code, cause and pointer: the `navigate` rule, the absent-node rule, the not-a-`model_population` rule and the wrong-`T` rule (FR-322 :1096-1109), and the dependency join (`invalid_semantic_graph`). The no-`Type`-occurrence rule is IR-only, and the text says so. The non-node-id rule and the negative-`max` rule differ, and the text says so.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | low | The `max` rule states the FR-322 difference only for a negative `max`, but FR-322 states the same derived-shape refusal for more cases. FR-322's semantic-shape check (:883-885) refuses `invalid_package`/`invalid-derived-shape` at the node a `bounded_domain`/`model_population` "whose `semantic_type` is not the `set<reference<T>>` node or whose `max` is absent, negative or not an integer literal". The reaches section says the node's `max` "is read as for any `model_population` node" (:1109). This reader refuses an absent `max` and a non-integer-literal `max` `ill_typed`/`operator-ineligible` at `member.population` (AC-279). So these cases differ in code, pointer and stage exactly as the negative case does. But the bullet and AC-279's parenthetical ("the negative case differs from the code FR-322 gives it at the node") name only the negative case, and the general sentence says each difference "is stated where it arises". A `semantic_type` that is not `set<reference<T>>` is the same kind of gap. FR-322 refuses it at the node's derived-shape stage. This reader has no such check (FR-038-AC-131, PLANNED). Its object-type rule goes through FR-053's `population_object_type`, which does not check the forms. So it either refuses `ill_typed` at `member.population` or admits, and the text states neither. No published vector exercises any of these cases. Fix: in the `max` bullet, replace "For a negative `max`" with "For a `max` that is absent, negative or not an integer literal (FR-322's model-population shape rule, RM-POP-MAX)", and make AC-279's parenthetical "the missing, non-literal and negative cases". Add one clause for a `semantic_type` that is not `set<reference<T>>`: FR-322 refuses it `invalid-derived-shape` at the node; until AC-131 this reader refuses it, or admits it, through the object-type rule. Optional cosmetic: at :3329, "and their order in its section" reads better as "and their order in FR-322's section". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3331-3335, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3420-3435, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4822 |

## Dispositions (round 6)

Round 6, reviewed at e5e582aaca6bedc492066f406cba2217d2e105a6.

| FND | outcome | reason |
| --- | --- | --- |
| FND-007 | fixed | The sentence now reads "follows its code, cause and pointer, except that a difference of code, pointer or stage (FR-322's schema stage against this reader's operation stage, which has no member schema, and FR-322's derived-shape check against this reader's lack of one) is stated where it arises". This covers both cases FND-007 named: the `malformed_wire` difference (:3413) and the negative-`max` difference (:3432). The rules that do match FR-322 are listed in this round's heading. The differences the `max` bullet still leaves unstated are FND-008. |

## New findings (disposition pass 7)

Reviewed at 83492489859fd49d70fe81edebcf34a00e40166f, against quire-specification origin/main 6f6136a6c (FR-322 "Derived type forms and forwarding" :836-890, "Population link of a `reaches`" :1088-1114).

The author raised two more cases: a duplicated `max` and a non-binding `max`. These are acceptable and are not a remaining IR gap. FR-322's refusal text (:884) names only an absent, negative or non-integer-literal `max`. Its accepted form (:848, :877) is one integer `max` binding, and it states no refusal or code for two `max` bindings. So the IR sentence "Where FR-322 states a refusal" does not reach the duplicated case, and IR's `ill_typed` there contradicts no stated FR-322 code. Any ambiguity is QSpec's, not IR's. A member named `max` that is not a binding leaves no `max` binding, so FR-322 reads it as "absent", and IR's text already covers the absent case. Optionally, IR could add "not a binding" to the absent case in AC-279's parenthetical. That is cosmetic. A second cosmetic point: RM-POP-MAX is the negative-bound vector only. The absent and non-literal cases come from FR-322's semantic-shape check in "Derived type forms and forwarding" (:883-885), which could be cited beside it.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | low | The new `semantic_type` clause understates the difference. It says this reader "does not check the `semantic_type` of the population node here", and that "The differences are in code, pointer and stage". For a `semantic_type` that is not `set<reference<T>>`, the difference is one of outcome: IR can admit what FR-322 refuses. FR-053's `population_object_type` (selected_population.rs:285-309) takes the first member `target` of the `semantic_type` node and then the first member `target` of that node, and checks neither form. So a population node whose `semantic_type` is, for example, a `Bag` or `Sequence` over a `Reference<E>` derives `E` and passes the object-type rule. The reader admits it, `select_population` returns `Finite`, and FR-322 refuses it `invalid_package`/`invalid-derived-shape` at the node (:883-884). The general sentence's exception list ("a difference of code, pointer or stage", :3331-3335) has the same gap. Fix (one clause): "and does not check the form of its `semantic_type`, so a node whose `semantic_type` is not `set<reference<T>>` but whose chain still names `E` is admitted here where FR-322 refuses it; the differences are in outcome, code, pointer and stage". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3430-3438, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3331-3335, crates/quire-contract-model/src/checked_package/selected_population.rs:285-309 |

## Dispositions (round 7)

Round 7, reviewed at 83492489859fd49d70fe81edebcf34a00e40166f.

| FND | outcome | reason |
| --- | --- | --- |
| FND-008 | fixed | The `max` bullet now names the absent, negative and non-integer-literal cases as FR-322 derived-shape refusals at the node (FR-322 :883-885, applied through :1109). It says this reader refuses each `ill_typed` at `member.population` until AC-131, and it names the `semantic_type` case. AC-279's parenthetical now reads "the missing, non-literal and negative cases differ". The :3329 cosmetic is done ("their order in FR-322's section"). The `semantic_type` clause understates its difference; that is FND-009. |

## Dispositions (round 8)

Round 8, reviewed at 056dcef304b3860f32725ddc9e9710144b5bfcb3. I checked the changed hunks (FR-038 :3331-3335 and :3424-3441, AC-279 :4829) against `selected_population.rs` and FR-322 on quire-specification origin/main 6f6136a6c. They contain no remaining inaccuracy and no new finding. One cosmetic point is left, and it is not recorded as a finding. AC-279's parenthetical now lists "not-a-binding" among the cases that differ "as the reader check states", but the `max` bullet lists only "absent, negative or not an integer literal". The two agree in substance, because a non-binding `max` is FR-322's absent case. A reader may still want "(a `max` that is not a binding is FR-322's absent case)" added to the bullet.

| FND | outcome | reason |
| --- | --- | --- |
| FND-009 | fixed | The bullet now says the reader "does not check the form of the node's `semantic_type` either (FR-053's `population_object_type` follows the first member target twice and checks neither form), so a node whose `semantic_type` is not `set<reference<T>>` but whose chain still names `E` is admitted here where FR-322 refuses it". This matches `selected_population.rs:285-309` and FR-322 :883-884. The differences are now "in outcome, code, pointer and stage". The general sentence (:3331-3335) now reads "a difference of outcome, code, pointer or stage". The two cosmetics from round 7 are also applied: the shape-rule citation "in 'Derived type forms and forwarding', whose negative-bound vector is RM-POP-MAX", and "not-a-binding" in AC-279. |

## New findings (disposition pass 9)

Reviewed at 07dc8d630383acd1ed63ed3930f3cd4014690398, after the merge of origin/main 157c317 (IR #440, #438, #437). Nothing from main was dropped. `git diff origin/main...HEAD -- spec` removes only three lines. The first is the IR-852 forward reference ("a separate decision (IR-852)"), now replaced by the section. The second is AC-252, re-added with only the cross-reference sentence. The third is the FR-038 coverage row, re-added with only the AC-269..280 range, a new 🚧 sentence and AC-262..268 changed from PLANNED to implemented. The rows for AC-262..268 are byte-identical to main. TC-048 adds lines and removes none. The round-8 cosmetic clause (9eb21d6) is accurate.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | low | The TC-048 summary row's Traces To cell (tests.md :32) omits FR-038-AC-269 through AC-280. It is byte-identical to main, so the merge kept it unchanged. Main's IR-840 (#440) made that cell list every FR-038 criterion that a Verification cell or a TC-048 procedure section binds to TC-048, planned criteria included (AC-165 and AC-239..241, both PLANNED, are listed). Every one of AC-269..280 has Verification "Test (TC-048)" and its own TC-048 section ("Typed population of a reaches application"), and the coverage row and the per-criterion rows of this PR name TC-048. Measured on HEAD: of the FR-038 criteria whose Verification names TC-048, exactly AC-269..280 are missing from the cell. A reverse lookup ("what does TC-048 verify") therefore misses this PR's twelve criteria. `quire coverage --strict` does not catch it (still 43/0). Fix: append `FR-038-AC-269, ..., FR-038-AC-280` to the TC-048 row's Traces To cell. | spec/checked_package/matrix/tests.md:32, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4820-4831 |

## Dispositions (round 10)

Round 10, reviewed at 62a49df2d3569b6924d2f4f6e15174adf9e8e461. The one commit since 07dc8d6 changes only tests.md line 32 and this SR file.

| FND | outcome | reason |
| --- | --- | --- |
| FND-010 | fixed | The TC-048 Traces To cell now ends with "FR-038-AC-268, FR-038-AC-269, ..., FR-038-AC-280". A character-level diff against 07dc8d6 shows that this is the only change in tests.md. The cell holds 218 FR-038 ids, in ascending order, with no duplicates and no id from another FR. Every FR-038 criterion whose Verification column names TC-048 is now listed. |
