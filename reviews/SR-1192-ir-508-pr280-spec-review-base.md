---
id: SR-1192
title: "spec review of PR 280 (FR-345 abstraction relation body, TC-224)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@f5bb9052b113d34353396cf8821c60c01c8140ed; git diff origin/main...HEAD (base 1117eba64f329fad9f7324b0ec9d2149be66f07f): spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md, spec/checked_package/matrix/TC-224-checked-package-v2-abstraction-relation-body.md, spec/checked_package/matrix/tests.md, spec/core/matrix/tests.md, spec/spec.md, spec/tests.md; checked against merged QSpec at quire-specification origin/main 2f846f88bd646781acc4704da4e46a3b54bf0116 (FR-451, FR-353, FR-450, FR-342, FR-322, proposals/checked-package-v2/schema.json) and IR FR-038, FR-040, FR-344 and crates/quire-contract-model/src/checked_package/v2 at the reviewed sha"
review_set: base
---
# SR-1192: spec review of PR 280

## Summary

Ticket: IR-508. Spec-only PR. It adds FR-345, which has the V2 reader admit merged
QSpec FR-451's `correspondence`/`abstraction_relation` body, and TC-224 with nine
planned ACs. It also adds matrix, tests and spec.md rows.

Measured at the reviewed sha:

- `quire validate` over spec, plan and reviews passes. It reports one grammar
  finding (FR-014, the baseline). `quire coverage --strict` reports 34 unbacked
  rows, against 23 at base 1117eba (measured on both). The 11 extra rows are
  FR-345, TC-224 and FR-345-AC-1..9, all planned. No CLAUDE.md, AGENTS.md or
  Makefile text requires the strict count to stay at 23. The FR-344 spec-first
  PR (#173) added planned unbacked rows the same way. The rise is acceptable,
  and IR-509's tests remove it.
- These parts match FR-451 and the published schema: the form gate (code has
  four `CorrespondenceForm` members; the schema's `CorrespondenceNode` enum adds
  `abstraction_relation`), the body shape, the keys, the `node_id` preimage, the
  `semantic_type`, the `dependencies`, the canonical order, the per-node check
  order (identity, order, targets, members) and the package-wide key uniqueness
  over object, population and operation keys. No other IR spec lists the closed
  correspondence forms, so the gate contradicts nothing. An unknown form still
  refuses at `semantic_form` (FR-344-AC-1).
- `RustIdentifier`, `RustField` and `RustReceiver` are plain `minLength: 1`
  strings in the schema. So `"not a name"` passes the schema and reaches the
  member step's syntax check, as AC-6 says.
- The defects are listed below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-345-AC-6 says a `fields` name that `ConfigVersion` does not expose refuses `missing_declaration`/`missing-name`. This contradicts FR-345's own Reader order step 4 (`invalid_model_binding`/`malformed-declaration`) and merged FR-451 "Reader order" step 4, which lists "a `fields` name that is not an effective field member of the object type" under `invalid_model_binding`/`malformed-declaration`. FR-451 defers to a resolution's own code only for FR-342's operation resolution. FR-040's frame-entry rule (missing-name) does not apply to this body. Change AC-6 to `invalid_model_binding`/`malformed-declaration` at the entry | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:192, 155-168 |
| FND-002 | high | Reader order step 4 says any frame `operation` that does not resolve under FR-040's anchor resolution refuses `invalid_model_binding`/`malformed-declaration`. It keeps FR-040's own code only for "an unrecovered owner or an ambiguous name". Merged FR-451 says such an operation refuses "with that section's refusals" (FR-342 "Operation resolution"). FR-342 and FR-342-AC-4 refuse an undeclared name, or a field's name, as `missing_declaration`/`missing-name`. So FR-345 gives an undeclared operation name the wrong code. No AC covers that case. Keep every FR-342/FR-040 resolution code (missing-name, ambiguous-name, missing-selection or stale-node-key, malformed-declaration for an inherited-only operation), and add an undeclared operation name to AC-6 | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:155-168, 192 |
| FND-003 | high | AC-1's minimal valid body uses `rust_type: ["crate", "ConfigVersion"]` and `function: ["crate", "attempt_update"]`. Under FR-450 "Rust spellings", every `RustPath` segment is a non-keyword Rust identifier. `crate` is a strict keyword, and FR-450-AC-5 refuses even `r#crate`. Under FR-345's own member step, this "admits" example therefore refuses `invalid_model_binding`/`malformed-declaration`. Use non-keyword segments, for example `["config_store", "ConfigVersion"]` | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:187 |
| FND-004 | high | AC-4 and AC-5 give one input two refusals. AC-4: a node whose `dependencies` omit a body target refuses `invalid_semantic_graph` at the node (identity step). AC-5: a target missing from `dependencies` refuses `missing_declaration`/`missing-name` at the target (targets step). The identity step runs first and requires `dependencies` to equal the body's targets exactly. So once it passes, every target is a declared dependency, and the targets step's missing-name branch is unreachable. The conflict comes from merged FR-451 (AC-3 against AC-4), but FR-345 copies it unresolved. Pick one outcome as a marked IR reading pending a QSpec ruling, or define the reachable missing-name case, and file the QSpec question | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:191, 190, 143-154 |
| FND-005 | high | TC-224 Description says the reader refuses "with the code, cause and locus FR-451 fixes". FR-451 fixes no locus for the shape, identity or order refusals. It gives no refusal code or cause for a schema failure (FR-451-AC-1 says only "fail the schema"). FR-345 takes the shape code and "no cause" from FR-040, and the identity locus ("at the node") and order locus ("at the array") are its own. The test description credits FR-451 with things FR-451 does not say. Say "the code, cause and locus FR-345 names" | spec/checked_package/matrix/TC-224-checked-package-v2-abstraction-relation-body.md:15-17 |
| FND-006 | high | The lowering paragraph says "merged QSpec FR-451 and FR-353 place the unbound-element refusal at emission onto implementation code". FR-451 says nothing about unbound elements. Only FR-353 (its Description and FR-353-AC-3) places that refusal at emission. Cite FR-353 alone | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:178-181 |
| FND-007 | medium | Several IR readings are written as if they were FR-451's text, with no mark. Only the "another form carrying this body" case is marked. The unmarked readings: the shape refusals' code, "no cause" and loci (from FR-040); the identity locus "at the node"; the order locus "at the array"; non-strict order ("two entries with an equal order key are in order"); "located at the entry unless a resolution refusal fixes its own locus"; and field-name ambiguity or unrecovered owners keeping FR-040's code (FR-451 defers only to FR-342 for operations). Description line 31-33 also says every other body refuses "with the refusal FR-451 fixes". Mark each one, as FR-038-AC-103 does ("the loci ... are an IR reading"), and pin the loci in the ACs as RFC 6901 paths (for example `/semantic_graph/nodes/{n}/body/objects/{i}`) | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:29-33, 96-104, 128-134, 143-168 |
| FND-008 | medium | FR-345 says the reader validates the abstraction body "against this shape alone, as it validates a frame body (FR-040), not against the semantic-term grammar". FR-038 "Frame bodies" still says the reader validates a `state`/`frame` body against its shape "and every other node's `body` against the semantic-term grammar alone". FR-038 "The flat wire" says the grammar walk "covers every node `body`" as strict wire validation, before identity. The PR does not amend FR-038, so the two requirements contradict each other on this node's body. A reader that follows FR-038 refuses every abstraction body at the term walk (`abstraction_relation` is no semantic-term tag). Amend both FR-038 sentences to name the second exception, as merged FR-322 "Body grammar" now lists the `abstraction_relation` body as a Body production | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:86-94; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1955-1960, 576-580 |
| FND-009 | medium | A `correspondence`/`abstraction_relation` node whose body root is an `application` term gets two outcomes. FR-345 says "a `term` other than `abstraction_relation`" refuses `invalid_semantic_graph` at `body`. FR-038-AC-115 (planned) says "a non-`case` application at the body root of a node that its class does not place there refuses `ill_typed`/`operator-ineligible` at the node that holds it". Leaving the case out of AC-2 leaves the conflict open, and the two codes come from different stages. Add the case to AC-2 with the code FR-345 picks, marked as an IR reading as FR-038 does for the frame body | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:96-104, 188; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2238 |
| FND-010 | medium | The abstraction step's place relative to the operation step is not stated. FR-040 "Reader order" fixes frame, then state, then operation. FR-345 says only "after the state step". That could put the abstraction step between the state and operation steps, or after the operation step. Merged FR-451 and FR-322 do not say either. AC-8 has no abstraction-against-operation pair, so either implementation passes. Pin the position as an IR reading pending QSpec, and add the pair to AC-8 | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:138-141, 194; spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:226-232 |
| FND-011 | medium | FR-345 does not reconcile a stale `node_id` with the generic key check. Merged FR-322 says "a node whose retained key differs from the digest of its preimage refuses as `invalid_package` with cause `stale-node-key`, after every graph-shape refusal and before any declaration, frame or operation refusal". FR-038 applies `stale-node-key` to the nominal and application keys. FR-451 gives the abstraction node's stale `node_id` `invalid_semantic_graph` at the abstraction step. FR-345 follows FR-451 but does not say the stale-key stage skips this node. One implementer would refuse a stale abstraction `node_id` as `stale-node-key` before the frame step, and another as `invalid_semantic_graph` after the state step. State that the stale-key stage does not re-derive this node's key, as an IR reading pending the QSpec FR-322/FR-451 reconciliation, and add a stale `node_id` plus a frame-step defect to AC-8 | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:117-124, 143-145, 190 |
| FND-012 | medium | AC-6 and AC-7 require the `conflicting-binding` refusal to name "both entries and the key". IR's `CheckedPackageRefusal` carries one `path`, one `cause` and one `locus`. FR-038 "Locating refusals" says the one refusal with a second pointer is `document_pointer`, which is about model documents. Nothing in the refusal can name a first entry, so a test cannot check "naming both entries" as written. The PR also does not record that a new member is needed. State how the refusal carries the first entry (a new member, a type change recorded in AD-004), or narrow it to the second entry's path and locus as an IR reading | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:193, 192, 155-176 |
| FND-013 | medium | AC-6's "two `rust_parameter` values that are equal" cannot be built as a single defect over the AC-1 fixture. `attemptUpdate(next: Integer)` has one parameter, so a second `parameters` entry also breaks "exactly the operation's declared parameters". Both checks give `invalid_model_binding`/`malformed-declaration` at the entry. The case therefore passes even if the equal-`rust_parameter` check is missing. Use a two-parameter operation for this case | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:192 |
| FND-014 | medium | AC-6 says "a frame `operation` that `ConfigVersion` only inherits", but AC-1's fixture has `ConfigVersion` declare `attemptUpdate` itself. The inherited-only case needs the subtype as `context`, which TC-224's procedure does build ("a subtype that only inherits the operation"). As written, the AC describes an input the fixture cannot produce. Name the subtype as the `context` | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:192; spec/checked_package/matrix/TC-224-checked-package-v2-abstraction-relation-body.md:21-23 |
| FND-015 | low | The lowering paragraph says lowering an admitted abstraction node is "not stated here". FR-038-AC-6 and FR-035 already require every admitted node family to lower with exact source, type and dependency correspondence. The `correspondence` arms in `v2/lower.rs` (`requires_bound`) and `v2/mod.rs` (`declaration_forbidden`) must take a value for the new form under the exhaustive-match rule. Say that the node lowers as the other `correspondence` forms do under FR-038-AC-6, or that it is excluded, so IR-509 does not pick a value without a spec. This does not block IR-509's admission work or QSL AR2 | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:178-181; crates/quire-contract-model/src/checked_package/v2/lower.rs:936-941 |

## Verdict

Not mergeable as written. Six high findings. FND-001 and FND-002 give refusal codes
that contradict merged FR-451, and FND-001 also contradicts FR-345's own Behavior.
FND-003's "admits" example is itself invalid under FR-450. FND-004 leaves two ACs
contradicting each other. FND-005 and FND-006 credit FR-451 with statements it
does not make. The medium findings are unmarked IR readings, the FR-038 grammar
sentences the PR leaves unamended, unpinned stage positions, an unobservable
"naming both entries", and two AC-6 cases the fixture cannot isolate. All are
text fixes inside the PR.

These parts are sound: the form gate, the body shape, the keys, the `node_id`
preimage, the canonical order, the per-node check order, the package-wide
uniqueness scope (object, population and operation keys, within one node or
across two), AC-3's frame-independence case, AC-9's identity propagation, the
matrix and tests rows, the spec.md sentence and the StR-001 core row. The ACs
are direct assertions with the planned marker and no tracker state. No QSpec
text is copied verbatim.

## New findings (disposition pass 1)

Reviewed at f4fb763abfe64d60c2959a717b0829634c7d4096 (delta from f5bb905, which renamed FR-345 to FR-346 and TC-224 to TC-225). Paths and lines below refer to that sha.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-016 | high | The new IR reading in Reader order step 3 swaps two FR-451 ACs. It says "FR-451-AC-3 (`missing_declaration`/`missing-name` for a target missing from `dependencies`) and FR-451-AC-4 (`invalid_semantic_graph` for `dependencies` that omit a body target)". Merged FR-451-AC-3 is the `invalid_semantic_graph` case ("whose `dependencies` omit a body target ... refuses `invalid_semantic_graph`"). FR-451-AC-4 is the `missing_declaration`/`missing-name` case. Swap the two parentheticals | spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md:182-188 |
| FND-017 | medium | The Body shape IR reading says the application body-root refusal and the member-value refusal "Both run in the strict wire stage of FR-038 'The flat wire', before this step". That holds for a nested application (FR-038-AC-114), but not for the body root. FR-038 decides body-root placement in later steps: temporal classes in the temporal step (FR-038 lines 1225-1235: "an application of these classes as the body root of a node of another form" is a placement defect, while a nested one refuses `malformed_wire` "at strict wire validation, ahead of this step"); `quire.op.state.clause` in the state step (FR-040 "Reader order"); `case` and the other classes in the operation step (FR-038 "Path and locus" row for a `case` body root). The abstraction step runs before the operation step. So for an application body root of an ordinary class, the abstraction step's identity, order or shape checks see the node before any placement check could give AC-2's `ill_typed`/`operator-ineligible`. State the stage that refuses an application body root on this node (or that the abstraction step skips such a node), and add the pair to AC-9 | spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md:110-116, 246; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1225-1235, 1406 |
| FND-018 | medium | The new step order "frame, state, abstraction, operation" leaves out FR-038's temporal step. FR-038-AC-102 says "The temporal step runs after the frame and state-clause step and before the operation step". Both the temporal and the abstraction step now sit between state and operation, but nothing orders them against each other. AC-9 has no temporal-against-abstraction pair. Place the abstraction step against the temporal step (IR reading), and add the pair to AC-9 | spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md:162-164, 253; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2229 |
| FND-019 | low | "An `application` standing as a member value inside the body refuses `malformed_wire`" is broader than FR-038 allows. FR-038-AC-115 refuses a nested `case` application `ill_typed`/`operator-ineligible` at its `operator`, never `malformed_wire`. AC-2's "an `application` standing as a `type` value" does not name a class, so a `case` application satisfies its wording but refuses differently. Say "a non-`case` application" in the prose and AC-2, or add the `case` exception | spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md:113-115, 246 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | f4fb763 (FR-346-AC-6: an unexposed `fields` name refuses `invalid_model_binding`/`malformed-declaration`; Behavior step 4 matches FR-451) |
| FND-002 | fixed | f4fb763 (step 4 keeps FR-342's refusals: undeclared or field name as missing-name, ambiguous as ambiguous-name, inherited-only as malformed-declaration; FR-346-AC-7 covers each, checked against FR-342 and FR-342-AC-4) |
| FND-003 | fixed | f4fb763 (AC-1 uses `["config_store", ...]`; AC-6 adds `crate` and `9lives` as refusals, per FR-450-AC-5) |
| FND-004 | fixed | f4fb763 (marked IR reading: the identity step refuses first and the targets-step missing-name branch has no input; AC-5 drops the case; AC-4 states it; QSpec question 1 filed in Dependencies. The citation in the new text is swapped, recorded as FND-016) |
| FND-005 | fixed | f4fb763 (TC-225 Description: "the code, cause and locus FR-346 names, which are FR-451's where FR-451 states them and FR-346's marked IR readings where it does not") |
| FND-006 | fixed | f4fb763 (Lowering cites FR-353 (FR-353-AC-3) alone for the emission refusal) |
| FND-007 | fixed | f4fb763 (Description says every code, cause or locus FR-451 does not state is an IR reading; each such reading is marked; RFC 6901 paths are pinned in Body shape, Canonical order, step 1 and AC-2/AC-4) |
| FND-008 | fixed | f4fb763 (FR-038 "The flat wire" and "Frame bodies" name the abstraction body as a second flat-shape exception; FR-038-AC-114..118 are unchanged and still consistent) |
| FND-009 | fixed | f4fb763 (AC-2 pins an application body root as `ill_typed`/`operator-ineligible` at the node and a member value as `malformed_wire`, consistent with FR-038-AC-115/114. The stage claim added with it is wrong, recorded as FND-017) |
| FND-010 | fixed | f4fb763 (IR reading: abstraction after state and before operation; AC-9 adds the abstraction-against-operation pair. The temporal step is left out, recorded as FND-018) |
| FND-011 | fixed | f4fb763 (Identity IR reading: the stale-key stage does not re-derive this node's key; AC-4 adds a stale `node_id`; AC-9 adds stale `node_id` against a frame defect; QSpec question 2) |
| FND-012 | fixed | f4fb763 (IR reading: path at the second entry, locus the key's node key, first entry not carried; AC-8 states it; QSpec question 4) |
| FND-013 | fixed | f4fb763 (AC-7 isolates the equal `rust_parameter` over `rebase(from, to)` with the declared parameters exact; `from` < `to` keeps the canonical order) |
| FND-014 | fixed | f4fb763 (AC-1's fixture adds `ConfigVersionDraft`, which only inherits `attemptUpdate`; AC-7 uses it as the `context`) |
| FND-015 | deferred | Lowering is now an explicit open question in FR-346 "Lowering" and QSpec question 8, with no outcome picked. This is low and does not block IR-509's admission code; the ruling is QSpec's |
| FND-016 | fixed | 036f501 (step 3 now cites FR-451-AC-3 as the `invalid_semantic_graph` case for `dependencies` that omit a body target, and FR-451-AC-4 as the `missing-name` case, matching merged FR-451) |
| FND-017 | fixed | 036f501 (Body shape names the stage, as an IR reading. Temporal classes are refused in the temporal step and `quire.op.state.clause` in the state step, both before the abstraction step, matching FR-038 lines 1225-1235 and FR-040. Any other class is refused by the shape check that opens the abstraction step, before its identity check, with FR-038-AC-115's code and locus, because the operation step comes later. AC-2 and AC-9 cover each case) |
| FND-018 | fixed | 036f501 (order frame, state, temporal, abstraction, operation, consistent with FR-038-AC-102; AC-9 adds a temporal placement + abstraction pair and an ordinary-class application root + operation-step pair) |
| FND-019 | fixed | 036f501 (prose and AC-2 say "non-`case`" for `malformed_wire` and add the nested `case` → `ill_typed`/`operator-ineligible` at its `operator`, per FR-038-AC-115) |
