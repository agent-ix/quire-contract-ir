---
id: SR-12150
title: "IR-852 spec review (base)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@edd6bf4fe524637261c7d6a5c0bfbbaa33efe77d (PR #439); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed population of a reaches application, AC-269..278, AC-252 reword), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-852 spec review (base)

## Summary

Ticket: IR-852. Spec-only PR #439 adds the FR-038 section "Typed population of a
reaches application": an optional `population` key in a reaches application's
`operation.member`, the accessor `reaches_population`, a reader check in
`check_relationship_member`, the dependency-join extension, AC-269..278 (PLANNED),
a reworded AC-252, a TC-048 procedure and tests.md rows. Each claim was checked
against origin/main code: `selected_population.rs` (`select_population` :157,
`select_admitted` :210-257, `population_maximum` :259-283, `population_object_type`
:285-309), `v2/operations.rs` (`OperationWire` :199-207, member opaque;
`check_relationship_member` :2132-2286; the calls after it at :858-880),
`v2/structural.rs` (`application_dependencies` :652-695, join compared at :358-362),
`common.rs` :1003-1008 (closed application key set), `v2/model_members.rs`
:125-163 (refusal constructors), `v2/mod.rs` :1985-2095 (stage order), and the QSL
branch `qsl819-population-bound` (FR-365 "The population link", FR-380).

Verified clean: `operation.member` is an opaque `Value` (no key-set change is needed,
and today a `population` key is admitted unread); the application key covers the
whole body, so the key re-keys the node with no preimage change; every code/cause
named exists (`MissingDeclaration`/`MissingSelection` = `ModelRefusal::unselected`,
`IllTyped`/`OperatorIneligible` = `ModelRefusal::ineligible`,
`InvalidSemanticGraph` with no cause = the join's `Defect::Dependencies`); dependency
edges are resolved only after `validate_operations` (mod.rs:2041-2066), so an
absent-node `population` reaches the operation-stage `missing_declaration`, as an
absent `declaration` does; the accessor takes the application id, not the claim,
and returns FR-053's selector, which agrees with QSL FR-365 (one link per
reachability, several per claim); the "least ordinal `Type` occurrence" is well
defined (occurrences are unique per (role, ordinal), mod.rs:1966-1975) and is the
convention QSL FR-365 writes down; the receiver end type and the population object
type agree with QSL FR-380 (T is the near-end type E); no new code, cause or pointer
vocabulary; the AC-269 id follows AC-268 with no collision; no research or roadmap
content.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| Measured-gap paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A quire.op.model.reaches application carries no population: its body keys are the closed set term, operator, operation, result_type and arguments, its operation.member is {kind, declaration, name} |
| The link | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | An admitted quire.op.model.reaches application MAY carry one additional key in operation.member, population, whose value is the node id of one bounded_domain / model_population node. |
| The accessor | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | CheckedPackageV2::reaches_population(&self, application: &CheckedNodeId) -> Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError> |
| Finite-outcome sentence | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The selector is exactly what FR-053's Admitted source accepts, so select_population over it returns Finite with the node's own max |
| Error paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Admission refuses each of those states, so UnresolvedPopulation is reachable only from a package that a crate-internal post-admission mutation changed. |
| The reader check | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | check_relationship_member shall check a population key of a reaches application after every check it makes today for that application |
| Dependencies paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The application-node dependency join (application_dependencies) shall add the population node id |
| What is not added | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The new error type is exported at the model root and listed in FR-019's Public items table by the code change that exports it |
| FR-038-AC-252 (reworded) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | its operation.member holds a population only through the optional population key that FR-038-AC-269 through FR-038-AC-278 specify |
| FR-038-AC-269 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | returns Ok(Some(selector)) ... occurrence the Type occurrence at ordinal 1, and select_population over Admitted with that selector returns Finite with maximum 3 |
| FR-038-AC-270 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | no population key it returns Ok(None), also in a package that holds a model_population node of the receiver end's declared type |
| FR-038-AC-271 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | refuses UnknownNode; ... NotReachesApplication for each; ... UnresolvedPopulation for each |
| FR-038-AC-272 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | not a node id, or is a node id with no node in the graph, is refused ... missing_declaration/missing-selection |
| FR-038-AC-273 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | not a bounded_domain / model_population node, or a model_population node with no Type occurrence ... ill_typed/operator-ineligible |
| FR-038-AC-274 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | over an object type other than the declared type of the selected receiver end, including a subtype ... refused |
| FR-038-AC-275 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A quire.op.model.navigate application whose operation.member carries a population key is refused |
| FR-038-AC-276 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | admitted only when its dependencies hold the population node id |
| FR-038-AC-277 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | a package that carries an invalid population key and an earlier existing defect is refused for the earlier defect |
| FR-038-AC-278 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | binds the result as Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError> and exhaustively matches |
| TC-048 population section | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | Build admitted packages from the IR-824 and IR-851 reaches fixtures, adding a bounded_domain / model_population node |
| tests.md FR-038 row and AC-269..278 rows | examined | spec/checked_package/matrix/tests.md | AC-269 through AC-278 (IR-852) ... are PLANNED / UNRUN, spec only |
| FR-053 select_population | context_only | crates/quire-contract-model/src/checked_package/selected_population.rs | select_admitted ... population_maximum ... population_object_type |
| QSL FR-365 The population link | context_only | quire-spec-language@qsl819-population-bound spec/functional/FR-365-... | S4 SHALL write population: <id> in the application's operation.member ... the occurrence of the node's Type role with the least ordinal |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The spec says that a selector from `reaches_population` makes `select_population` return `Finite`. Admission does not guarantee that. Admission never reads a `model_population` node's `max` body. `body_grammar` is plain `Term`, there is no derived key (`derived_keys.rs:82`), and `composite_operands.rs:823` reads no bound. So a reader-admitted package can hold a population node whose `max` is missing, duplicated, not an integer literal, negative or above u64. The reader check passes tag, form, `Type` occurrence and object type. The accessor returns `Some(selector)`, and `select_admitted` then returns `NotPopulation` (`population_maximum`, :242-244). AC-269 holds only because its fixture's `max` is 3. Decision on the merits: the reader check should also refuse a population node whose body FR-053's `max` read rejects, `ill_typed`/`operator-ineligible` at `population`, reusing `population_maximum` rather than a second parser. The check already enforces every other FR-053 condition (tag, form, `Type` occurrence, the `Set`/`Reference` chain), so `max` is the one gap, and closing it makes `Some(selector)` imply `Finite`, which is what the consumer relies on. Add an AC for it. The alternative is to drop the guarantee: say `select_population` returns its own outcome and `NotPopulation` for a malformed `max`. That leaves CG a reachable `NotPopulation` branch from an admitted package. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3350-3354, crates/quire-contract-model/src/checked_package/selected_population.rs:242-283, crates/quire-contract-model/src/checked_package/v2/mod.rs:1591-1594, crates/quire-contract-model/src/checked_package/v2/derived_keys.rs:82 |
| FND-002 | medium | The third reader-check bullet goes beyond where the check runs and beyond its AC. It refuses "a `population` key on an application of any operation other than `quire.op.model.reaches`". But `check_relationship_member` runs only for `navigate` and `reaches` (`operations.rs:858-867`), and AC-275 tests only `navigate`. A `population` key in the member of a `field`, `position`, `element`, `operation` or `type_argument` application stays unread and admitted, against the prose. The dependency paragraph also does not say whether the join adds `population` for a non-reaches application. That decides whether AC-275's fixture must list the id in `dependencies`. If the join does not add it, a listed id is an extra dependency, and the structural stage refuses `invalid_semantic_graph` before any operation-stage check (`mod.rs:1989`). Fix: narrow the bullet to `navigate`, saying other member kinds stay opaque as today. Or name the stage that checks every other operation and add an AC. Either way, state the join's scope (reaches only, or any application whose member carries `population`). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3389-3391, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3395-3402, crates/quire-contract-model/src/checked_package/v2/operations.rs:858-867, crates/quire-contract-model/src/checked_package/v2/structural.rs:652-695 |
| FND-003 | low | The reader check defines the object type in prose: "the `Reference` target of the `Reference` target of the `Set` the node's `semantic_type` names". It does not tie that walk to FR-053's `population_object_type`, the one `select_population` uses. The two walks can drift apart. FR-053's walk takes the first member's `target` and does not check the forms. If the reader check is stricter (it checks forms) or looser, then an admitted population could resolve to a different `object_type`, or none (`NotPopulation`). Fix: say that the reader check derives the object type with FR-053's derivation, so there is one derivation, as the accessor paragraph already requires for the accessor. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3383-3388, crates/quire-contract-model/src/checked_package/selected_population.rs:285-309 |

## Verdict

PASS WITH FINDINGS. The link, the accessor's shape, the reuse of FR-053's selector,
the code/cause vocabulary, the dependency-join ordering and the agreement with the
QSL-819 branch (key name, value, absent-not-null, least-ordinal convention, E as the
object type, one link per reachability) were verified. FND-001 is the author's open
question (whether the reader checks the `max` body). The answer is yes: check it
with FR-053's own reader. Note, not a defect: the least-ordinal occurrence can be
the `Type` occurrence of a `Population<E>[N]` parameter type that shares the node,
so the `Finite` provenance may point at that parameter's span rather than at the
`bound` declaration. Both specs accept this (QSL FR-365: "the node does not say
which place it came from").

## New findings (disposition pass 1)

Reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The narrowed fifth bullet says that for any other operation "`operation.member` stays opaque and unread as it is today". That is false for several member kinds. `member_is_well_formed` closes the `type_argument`, `temporal_interval` and `fairness` member shapes (temporal.rs:421-438) and `clause_body` closes the `state_clause` member to `{kind, clause}` (structural.rs:574). A `population` key there is refused today. AC-275's "keeps its present admission outcome" is correct; only the prose overstates it. The open QSpec FR-322 amendment (quire-specification #268) also says "on any other member kind the schema refuses it". Fix: say that a member of any other kind keeps its present outcome (opaque kinds unread, closed kinds refused as today), and do not say it is unread. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3409-3413, crates/quire-contract-model/src/checked_package/v2/temporal.rs:421-438, crates/quire-contract-model/src/checked_package/v2/structural.rs:574 |
| FND-005 | low | The reader-check rules apply "in this order", and the `navigate` rule comes last. A `navigate` application whose `population` names no node therefore refuses `missing_declaration`/`missing-selection` (rule 1), not AC-275's `ill_typed`/`operator-ineligible`. A node of the wrong form refuses under rule 2 with the same code but for a different reason. AC-275 says only "a node id that its `dependencies` list", so the fixture is underdetermined. Fix: make the `navigate` rule the first population rule (refuse a key on `navigate` before resolving it, as QSpec #268 states it). Or say in AC-275 that the id names a valid population node of the receiver end's type. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3383-3413, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4781 |

## Dispositions

Round 1, reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | A third reader rule now refuses a `max` that FR-053's `population_maximum` rejects, read through that function itself. AC-279 backs it. `Finite` is now justified "Because the reader check below refuses a population node whose max FR-053 cannot read". |
| FND-002 | fixed | The fifth bullet is narrowed to `navigate`. The join is scoped to reaches and navigate applications. AC-275 covers the listed-id and omitted-id cases and other operations. The residual prose overstatement is FND-004. |
| FND-003 | fixed | The reader derives the object type "through FR-053's population_object_type itself, so that there is one derivation". |

## New findings (disposition pass 2)

Reviewed at 444e675529d80ac4561e1f98a0e3ff1aeb96e475.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The new paragraph on other operations sorts member kinds into "closed today (`type_argument`, `temporal_interval`, `fairness`, `state_clause`)" and "opaque today (`field`, `position`, `element`)". It leaves out `operation` and `profile_operator`. `member_is_well_formed` returns true for both (temporal.rs:430-436) and no stage closes their shape, so they also admit the key unread. A reader of the two lists cannot tell what happens to those two kinds. Fix: add `operation` and `profile_operator` to the opaque list, or say "every other kind is opaque". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3432-3438, crates/quire-contract-model/src/checked_package/v2/temporal.rs:421-438 |

## Dispositions (round 2)

Round 2, reviewed at 444e675529d80ac4561e1f98a0e3ff1aeb96e475.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | The prose now says that a member kind whose shape is closed today refuses the key as it does today, and an opaque kind admits it unread, and it names the kinds. AC-275 matches. The incomplete opaque list is FND-006. |
| FND-005 | fixed | The `navigate` rule is now the first population rule ("before it resolves the value ... the refusal does not depend on what the value names"). AC-275 covers an absent node, an integer-range node and a valid population node. |
