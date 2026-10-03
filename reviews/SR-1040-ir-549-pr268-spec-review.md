---
id: SR-1040
title: "spec review of PR 268: the v2 reader admits temporal formulas, fairness and control.case (IR-549)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@ca2a5454fa3b3d9fcf683606909ab5122d4207d1; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md; base feat/artifact-ref-authority-identity e8db0876; read against quire-specification@b1da9c8 FR-370, FR-440, FR-441, FR-322, schema.json, node-identity-preimage.schema.json and fixtures"
review_set: subset
---
# SR-1040: spec review of PR 268: the v2 reader admits temporal formulas, fairness and control.case (IR-549)

## Summary

Ticket: IR-549. PR 268 is spec-only. The diff is
`git diff origin/feat/artifact-ref-authority-identity...ca2a545`, base e8db087
(held PR 253), and it touches four files: FR-038, TC-048, the checked_package
matrix and spec/tests.md. The PR body and the author's report were treated as
data. Every claim was measured.

What I measured:

- QSpec at quire-specification origin/main b1da9c8. I read FR-370, FR-440,
  FR-441, FR-322 and FR-272, schema.json, node-identity-preimage.schema.json
  and every fixture under proposals/checked-package-v2/fixtures/.
  - Temporal applications appear in positive-all-families (nodes 28 and 29:
    `holds`, and `eventually` with {lower "0", upper "3"}) and in
    positive-clause-operations (nodes 18 and 19).
  - `case` appears only in positive-union-nodes (node 13).
  - No positive fixture holds a temporal/fairness node.
  - Every temporal or case application stands at a body root.
  - Both temporal fixtures select the temporal_profile
    quire.temporal.event-position.false-extension/v1.
- Coverage of the positive fixtures. The AC set covers every operator, member
  and form these fixtures contain.
- adverse.json holds no temporal, case or union mutation. Its body-grammar
  mutations refuse any nested application as malformed_wire. That conflicts
  with IR's existing stated deviation, under which nested applications are
  admitted, so it is not new to this PR.
- The author's identity claim holds. I recomputed the application-node
  preimage for positive-union-nodes: it matches only nodes 12 (binary) and 13
  (case). The union and union_value node ids are not application preimages,
  and node-identity-preimage.schema.json defines no preimage for them. So "the
  other three forms carry the node_id every node carries" is right, and no
  union member key is carried (FR-441).
- The four node forms and their shapes agree with schema.json UnionTypeBody,
  UnionValueBody and CaseBody. The fairness member shape agrees with
  OperationMember.
- IR code at e8db087:
  - `is_unsupported` is at vocabulary.rs:451. The nested refusal is at
    common.rs:752 and the body-root refusal at operations.rs:451. Both are as
    described.
  - CompositeTypeForm, ValueForm, ExpressionForm and TemporalForm do not
    decode union, union_value, case or fairness.
  - lower.rs reads no operator. Its exhaustive form matches (requires_bound,
    quantity_chain) and operations.rs operand_family must decide the new forms.
  - The refusal code UnsupportedConstruct (shared.rs:129) is distinct from the
    diagnostic code CheckedDiagnosticCode::UnsupportedConstruct
    (v2/mod.rs:250).
  - IR main 0a889f9 has no UnsupportedConstruct refusal variant.
- Tests the code PR must change:
  - tests/it/checked_package_v2_catalog_words.rs: the three AC-66 tests (lines
    77, 138 and 172).
  - operations.rs unit tests for AC-66 (6079), AC-67 (6130), AC-68 (6302) and
    AC-69 (6382).
  - The AC-87 operand-family unit test.
  - The v2_all_families fixture: its formula node's body is an empty aggregate
    (tests/it/support/checked_package.rs:1256).
- Downstream, read-only:
  - quire-contract-codegen matches none of the changed enums. Its oracles
    refuse the temporal tag as BlockedOnUpstream, and they dispatch on
    operation identity allowlists.
  - quire-driver touches only MalformedWire.
  - quire-spec-language has an exhaustive map_refusal_code and an
    admission-corpus test over CheckedNodeKind::all() (SR-1041 FND-001).
- `make spec` was run in a detached throwaway worktree under
  /home/peter/dev/worktrees, since removed. Results: validate passes, one
  grammar finding (FR-014 ac:vague-response), and --strict 23 unbacked with 0
  contradicted. That is the baseline.
- AC numbering is stable (AC-66 keeps its number, and AC-96 to AC-101 are new
  and contiguous). The new rows are direct assertions. The matrix rows mark
  AC-96 to AC-101 as 🚧 planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The amended order contradicts QSpec FR-370 'Reader order' and FR-370-AC-7. FR-370 runs a temporal step after the state step and before any operation refusal, and it checks the placement of every temporal application first, in ascending node-id digest order. The PR puts a body-root misplacement inside the operation step, after the identity lookup and the class comparison. AC-67 then has the lower-digest node win 'whether its defect is a misplaced temporal_formula application or another operation refusal'. Under QSpec, the placement defect is reported first whatever the digest order, and before unknown-operation and operation-class-mismatch. Either move placement to its own step ahead of the operation step, or state this as a deviation with a ruling. Do not pin it as a rule. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1642, 963-980 |
| FND-002 | high | Placement is stated only in one direction: where an application may stand. The other direction is missing. Nothing requires a temporal/formula node's body root to be a temporal_formula application, a temporal/fairness node's body to be a quire.op.temporal.fair application, or an expression/case node's body to be a case application. The published schema binds exactly those bodies (TemporalFormulaBody, TemporalFairnessBody, CaseBody). FR-370's other placement rule is also missing: a formula node may be referenced only from a clause's formula argument or as a temporal_formula operand, and a fairness node only from a clause's fairness argument. As written, the in-repo all-families formula node with an empty aggregate body (tests/it/support/checked_package.rs:1256-1263) stays admitted, although QSpec's schema refuses it. The PR says the fixture 'gains' nodes, not that this one is replaced. Add the body rule and the reference rule, each with an AC row. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:944-980 |
| FND-003 | medium | All three 'stated deviations' depart from QSpec reader requirements: FR-370 says 'The reader SHALL refuse', FR-370-AC-3 covers profile fit, and FR-370-AC-4 covers fairness resolution with missing_declaration/missing-name. The stated reason, 'profile semantics, which the component that evaluates the profile owns', is not supported. FR-370 gives these checks to the reader's temporal step. The reader already resolves a model-owned operation member on its declaring node (FR-322 'Model-owned members', FR-038-AC-84). AC-97 goes further than a deviation: it asserts that a null interval is admitted under any selected profile. No ticket or QSpec ruling tracks the deviations. Either check them, or record each one as a deviation with a tracking ticket. AC rows should not pin the non-refusal. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:982-990, 1672-1673 |
| FND-004 | medium | 'Decimal string' is ambiguous. QSpec's schemas pin interval bounds as IntegerString, ^(0/-?[1-9][0-9]*)$: '1.5' and '01' are refused and '-1' passes the pattern. The AC's lower > upper oracle is {3, 0}, a single-digit pair. A reader that compares the strings lexicographically passes it, but would wrongly refuse {"9", "10"} and wrongly admit {"10", "9"}. Pin the IntegerString grammar. Add multi-digit cases ({9,10} admits, {10,9} refuses invalid-value) and the '1.5', '01' and negative-bound cases. Also say how a bound past u64 compares. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:900-911, 1672 |
| FND-005 | medium | The prose locates every case defect 'at the case node's arguments'. AC-99 and the TC-048 section say 'at the case node', and QSpec FR-440 reader join 3 says 'located at the case node'. Two implementers would emit different pointers. Pick one and match QSpec. Separately, AC-99 leaves out FR-440-AC-4's repeated-member arm, which the prose lists ('omit or repeat a member'), so a duplicate arm has no test. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:929-942, 1674 |
| FND-006 | medium | The retired AC-66 stays in the AC table with 'Test (TC-048)' as its verification. It is still in TC-048's covered list and in the matrix range 'AC-35 through AC-88'. TC-048's status still says 'AC-65 through AC-69 are implemented' and 'AC-46 through AC-69 ... implemented'. The repo's own precedent for a retired criterion is FR-031-AC-5 (ADR-0056): the row is removed, the id is never reused, and a prose note remains. When the code PR removes the AC-66 tags, a 'Test' row with no traced test is likely to show up as unbacked under quire coverage --strict, which would move the baseline from 23. Follow the FR-031-AC-5 pattern. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1641 |
| FND-007 | medium | The temporal operands are admission-only. No AC row asserts an operand refusal for a temporal entry. FR-370-AC-7 requires a holds operand of a non-Boolean family to refuse ill_typed/operator-ineligible, and the prose says operand count and family refuse 'as for every entry'. A reader that skipped check_operands for the 16 temporal entries would still pass AC-96 to AC-101. Add negative rows: holds over a temporal reference, not over a boolean, and until with one argument. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:920-928, 1671 |
| FND-008 | medium | QSpec FR-440-AC-6 is not covered. It says structural.eq over two Shape values admits, and over values of two different unions refuses same_type. FR-440 places the union family in the any_value and structural_kind groups. The PR fixes the union operand family but says nothing about union equality. It also says nothing about the leaf derivation of structural.eq or structural.ne over a union whose payload reaches text: FR-322's leaf path segments have no union-member segment. Add an AC for FR-440-AC-6. Either define union leaves or state that the gap is open, and route the question to QSpec. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:944-961, 1674 |
| FND-009 | medium | The prose says QSpec's positive fixtures are 'the conformance bar of this reader'. No AC or TC ever reads them: every case is self-built, which is correct under the no-vendoring rule. So conformance is asserted but never measured. The self-built Shape package differs from positive-union-nodes.json in ways the AC does not name: the scrutinee is a value/parameter typed at the union, union_value payloads are references to value/literal nodes, binders carry {name, level}, and the case dependency list differs. Name the fixture features the self-built packages must reproduce, field by field. Also say where the real-fixture evidence lives, for example QSpec TC-217 consumer evidence or QSL A1b. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:185-228 |
| FND-010 | low | Stale matrix prose in rows this PR edited. The FR-038 row still says the formula node is 'never a temporal_formula application, which the reader refuses'. TC-048's status still says 'the formula a clause names is itself refused'. Both become false when IR-549's code lands. Reword them as describing the IR-503 shape. | spec/checked_package/matrix/tests.md:15, 23 |
| FND-011 | low | IR's FR puts a normative 'the evaluating component SHALL refuse ...' on codegen and the runtime, and then calls it out of scope. An IR requirement cannot bind another repository. Make it a non-normative note, and route the rule to a CG/RT ticket. IR-549's own text asks for 'separate ticket if IR has no evaluator'. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:992-1006 |
| FND-012 | low | The new refusal causes are not stated as code-PR additions. FR-440 needs invalid_package/duplicate-member and ill_typed/type-mismatch. CheckedPackageRefusalCause (checked_package/shared.rs) has neither variant today. The earlier text said 'the code PR adds the matching variants and no other pairing', and this text should say the same. Also, QSpec schema.json's cause_tag enum lacks type-mismatch, although FR-272 pairs it with ill_typed. That is the same kind of QSpec gap the earlier text flagged for expression-form (STD-153). Note it and route it to QSpec. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:944-961 |

## Verdict

Changes requested, for two high findings.

- FND-001: the placement order contradicts FR-370 "Reader order" and
  FR-370-AC-7.
- FND-002: there is no body rule binding each new form to its application, and
  no rule on where formula and fairness nodes may be referenced from. As
  written, the reader keeps admitting a schema-invalid formula node.

The medium findings should be fixed in the same round:

- FND-003: the deviation rationale
- FND-004: the interval grammar and its oracle
- FND-005: the case locus
- FND-006: the retired-row pattern
- FND-007: operand negatives
- FND-008: FR-440-AC-6
- FND-009: the unmeasured conformance bar

These are sound and verified:

- Admitting the three operator classes and four forms. This covers every
  operator, member and form the positive fixtures carry.
- Keying a case node by the application preimage, with no new union preimage.
- The interval and fairness member shapes, which match the schema except for
  the bound grammar.
- The union_arms rules against FR-440.
- The decision that IR does not evaluate.
- AC-68, AC-69 and AC-87 as amended.

## New findings (disposition pass 1)

Reviewed at e5da11e53bfe7c4e49c517d4e41a4bd09faa73c6 (delta ca2a545..e5da11e).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-013 | medium | The temporal step's stage order can be read two ways. The prose numbers profile fit and interval bounds as stages 3 and 4, 'within each stage the lowest node_id digest first', while stage 2 runs them per clause. QSpec FR-370 nests them per clause: 'for each temporal/temporal_clause node in ascending node-id digest order: the over ...; then each fairness member ...; then the profile fit ...; then each interval's bounds'. AC-102's 'each adjacent pair ... built in both digest orders' does not say the pair belongs to one clause. Read across clauses, it reports a higher-digest clause's over defect ahead of a lower-digest clause's profile-fit defect, which contradicts FR-370. Fix: say the stages run per clause, and add a two-clause case. Also, the temporal step reads interval and fairness members before the operation step checks their shape, so the outcome is unspecified for, say, {lower: '1.5', upper: '0'} or a wrong-kind member under a bounded profile. Say which refusal wins. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:989-1040, 1753 |
| FND-014 | medium | The fairness rule contradicts itself and departs from FR-370 without saying so. It says the name 'resolves ... as an operation anchor's operation name does (FR-040 "Operation anchor body"), else missing_declaration/missing-name'. FR-040's anchor resolution has more outcomes than that: two or more matches refuse ambiguous_declaration/ambiguous-name, an operation the context only inherits refuses invalid_model_binding/malformed-declaration, and an unrecovered owner has its own refusal. FR-370 names only missing_declaration/missing-name. AC-103 tests none of the extra outcomes. Fix: either list the outcomes and flag them as an IR reading pending a QSpec ruling, or state the FR-370 rule alone (resolve on the declaring node, else missing-name). In both cases add an AC case for an inherited operation and an ambiguous name. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1015-1025 |
| FND-015 | medium | 'A run that cannot read the checkout reports this criterion as not run, never as passed' has no mechanism in this harness. cargo test has no not-run outcome: an early return reports passed, which the AC forbids. #[ignore] does not help either, because `make test` runs `-- --include-ignored`. The AC also leaves open where the checkout is found (env var or path) and what CI does. Fix: name the mechanism. For example, a dedicated make target or test reads QUIRE_SPECIFICATION_DIR, fails loudly when the variable is set but unreadable, and is excluded from `make test`. Say how the matrix row reports a criterion that CI does not run. Measured with a scratch probe at e8db087: the all-families and clause-operations fixtures refuse only at nodes 29 and 19's temporal_formula operator, union-nodes refuses only at the union form gate, and the other three positive fixtures admit. So the AC can be met. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1758 |
| FND-016 | low | make spec at e5da11e still reports strict unbacked 23. It also now prints four dangling traces ('traces to FR-038-AC-66, which matches no declared row'): three tests in tests/it/checked_package_v2_catalog_words.rs and one unit test in operations.rs. They stay until the code PR re-tags those tests. This is acceptable only if IR-549's code lands in the #253 branch before #253 merges to main. Record that in the matrix row so the dangling traces are not mistaken for drift. | spec/checked_package/matrix/tests.md:15 |
| FND-017 | low | Two IR readings are written as QSpec text. (1) A misplaced reference is 'located at the node that holds the misplaced application or reference (QSpec FR-370: "at the node that holds them")', but FR-370 says that only of applications. (2) The case rule refuses a case application at the body root of, say, a function node, and justifies it by 'this reader does not operation-check a nested application'. A body root is operation-checked, so that reason covers only nesting. The real basis is FR-440's producer SHALL, which encodes every case expression as an expression/case node. Label both as readings. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1007-1014, 1041-1056, 1751 |

## Dispositions

Round 1, reviewed at e5da11e53bfe7c4e49c517d4e41a4bd09faa73c6.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | e5da11e. AC-67 now puts a temporal-step placement defect ahead of every operation refusal whatever the digest order. AC-102 pins the stage order. The prose runs the temporal step after the state step and before the operation step, as FR-370 'Reader order' and FR-370-AC-7 require. The within-step ordering is a new finding, FND-013. |
| FND-002 | fixed | e5da11e. Stage 1 binds each temporal form to its body, and the case form is bound by the term walk. FR-370's reference rule is added. AC-100 tests empty-aggregate, literal and other-class bodies and every misplaced reference. The in-repo empty-aggregate formula node is named as refused and replaced. |
| FND-003 | fixed | e5da11e. All three deviations are now checked in the temporal step: over (missing-name / malformed-declaration), fairness resolution (missing-name) and profile fit (operation-member-mismatch). AC-103 and AC-104 test them. The FR-040 reuse wording is a new finding, FND-014. |
| FND-004 | fixed | e5da11e. Bounds are pinned to IntegerString ^(0/-?[1-9][0-9]*)$ and compared as unbounded integers. AC-97 adds {9,10}, {10,9}, the 2^64 pair, and '1.5', '01', '+1', '' and '-1'. invalid-value is now located at the application, as FR-370 says. |
| FND-005 | fixed | e5da11e. The prose now says 'located at the case node', with the pointer /semantic_graph/nodes/{n}, in line with FR-440 join 3, and AC-99 adds the repeated arm. |
| FND-006 | fixed | e5da11e. The AC-66 row is removed and the ID is not reused, citing ADR-0056 and following FR-031-AC-5. The matrix range and TC-048's list exclude AC-66. make spec at e5da11e reports strict unbacked 23. The transient dangling traces are FND-016. |
| FND-007 | fixed | e5da11e. AC-105 adds the negative operand cases: holds over a formula reference, until with one argument, not with two, true with one, and and over a Boolean reference. |
| FND-008 | fixed | e5da11e. AC-106 covers FR-440-AC-6. The union-leaves gap is stated as an open QSpec question that pins no outcome, which is the correct handling and is escalated for a ruling. |
| FND-009 | fixed | e5da11e. AC-107 reads QSpec's three positive fixtures from the checkout and names the fixture features the self-built packages reproduce. A scratch probe against e8db087 shows the other three positive fixtures already admit, and the three targets refuse only at the IR-549 constructs, so AC-107 can be met. Its harness wording is FND-015. |
| FND-010 | fixed | e5da11e. The stale matrix and TC-048 phrases are gone. FR-038 now names the empty-aggregate formula node as refused and replaced. |
| FND-011 | fixed | e5da11e. Now an explicit note: 'note, not a requirement of this specification'. No SHALL remains in the delta. |
| FND-012 | fixed | e5da11e. The two new causes are stated as code-PR additions with FR-440's pairings. The schema cause_tag gap for type-mismatch is flagged as a QSpec question, as was done for STD-153. |

## New findings (disposition pass 2)

Reviewed at f770415a2005bddd77d0f5e9fb2a7977a359c23b (delta e5da11e..f770415).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-018 | medium | The delta writes seven 'QSpec owner' rulings into FR-038 as settled inputs, each marked 'pending QSpec merge': the type-mismatch schema entry, the member:<Ident> leaf segment, the misplaced-reference locus, the ambiguous-name fairness outcome, 'every other known profile is bounded', the list of known temporal profiles, and case placement including details. None has a source I can measure. quire-specification main is still b1da9c8, and no QSpec branch, open PR, or STD/IR ticket carries them (searched). The IR-549 comments hold only review records. AC-106 and the unknown_profile rule turn two of these rulings into concrete outcomes. A ruling relayed by another agent is not a record. Fix: cite the durable record of each ruling (a QSpec PR or a ticket id), or keep each one framed as an IR reading pending a QSpec ruling until that record exists. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:969-972, 1020-1024, 1036-1040, 1064-1066, 1071-1074, 1083-1095 |
| FND-019 | medium | The new rule 'an identity the lock selects that is not a known profile refuses unknown_profile ... and is never read as bounded' has several gaps. (1) No AC row backs it and TC-048 has no case for it. (2) The 'known identities are QSpec's list', but merged QSpec has no such list: it names at least seven quire.temporal.* identities and does not state which are bounded. (3) CheckedPackageRefusalCode has no UnknownProfile variant today (shared.rs:81, checked), so this is one more public-enum change. The 'breaking change' paragraph lists only the four forms and the removed code, yet QSL's exhaustive map_refusal_code (qsl-package/src/checked_v2.rs:493) breaks on an added variant too. (4) The in-repo v2_all_families clause selects 'quire.fixture.temporal-profile/v1' (tests/it/support/checked_package.rs:624-631), which is not a known profile. Under this rule that fixture, and every row built on it, refuses unknown_profile, and the spec does not say the fixture's profile selection changes. Also unstated: the locus, and the rule's place in the order. Fix: add an AC, name the fixture change, add the code to the breaking-change paragraph, and hold the rule until QSpec's list is merged. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1060-1076, 1116-1122 |
| FND-020 | low | The oracle says 'a leaf list missing either, repeating one or holding a leaf for Empty refuses operation-law-missing or operation-law-mismatch at operation.leaves'. That allows either cause for each case. AC-70's convention is stricter: a missing leaf refuses operation-law-missing at operation.leaves, and an extra or repeated leaf refuses operation-law-mismatch at that entry (operation.leaves/<i>). Map each case to its cause and pointer, so a reader that swaps the two causes fails. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1798 |

## Dispositions (round 2)

Reviewed at f770415a2005bddd77d0f5e9fb2a7977a359c23b. FND-001 to FND-012 were fixed in round 1 and are not re-rowed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-013 | fixed | f770415. The prose now runs placement over the whole graph first, then each clause through over, fairness, profile fit and bounds before the next clause, as FR-370 'Reader order' nests them. AC-102 adds the within-clause pairs and the cross-clause case. The shape-skip rule pins {lower:'1.5', upper:'0'} to operation-member-mismatch under either profile. |
| FND-014 | fixed | f770415. Every outcome of the FR-040 resolution is listed, and AC-103 tests each one. The inherited-only outcome is flagged honestly as a reading. One note on that flag: FR-362 resolves a source name to an effective operation, but FR-370's member names 'the operation's declaring model node'. A member that names a node which only inherits the operation is therefore already off FR-370's text, which supports refusing it. Only the cause (malformed-declaration rather than FR-370's missing-name) is truly open. |
| FND-015 | fixed | f770415. AC-107 now names the test, its separate target, `make conformance-qspec`, and QUIRE_SPECIFICATION_DIR, and says the run fails rather than skips when the variable is unset, the path is missing, or a fixture does not admit. It is excluded from make test and make ci, and the matrix says CI does not run it. I checked the claim that there is no precedent: no test in tests/ or crates/ reads an external checkout (output_mapping.rs:533 only uses the string 'agent-ix/quire-specification' as data). The design is sound. The Makefile and Cargo target edits belong to the code PR, and this delta edits only spec files. The exclusion has to survive `make test` = cargo test --workspace --all-targets --include-ignored, which needs `test = false` or required-features on the target. That is code-PR detail, and the AC's outcome is checkable. |
| FND-016 | fixed | f770415. The matrix row now records the four dangling AC-66 traces as not drift, removed inside #253 before it merges. make spec at f770415 shows the same four lines and strict unbacked 23. |
| FND-017 | fixed | f770415. The reference locus is labelled as a ruling, not as FR-370 text. The body-root case basis is now FR-440's producer SHALL, with the nesting deviation as a second reason for the nested position only. Where those rulings come from is FND-018. |

## New findings (disposition pass 3)

Reviewed at 9a5d5b57461dccb1c144144136d7b078393a57db (delta f770415..9a5d5b5).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-021 | high | AC-97 says the reader admits the interval {lower: '-1', upper: '3'} under infinite-trace. Merged QSpec FR-255 says that under every profile except timed 'both ends are closed and the bounds are integers', and its Properties table gives the lower bound as 'Integer, 0 <= a'. FR-091 states the form as 'inclusive integers 0 <= a <= b'. A negative lower bound is therefore outside QSpec's interval domain, although schema.json's IntegerString pattern lets '-1' through. I missed this in round 1, when this row was introduced at e5da11e. Fix: refuse a negative bound (invalid-value, or operation-member-mismatch as a shape refusal), or leave it unpinned pending a QSpec ruling. Do not assert that it admits. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1828 |
| FND-022 | medium | AC-108 pins the cause unknown-profile for an unrecognized temporal profile, on a relayed ruling. Merged QSpec FR-272 already pairs unknown_profile with the closed variants unsupported-selection and wrong-selection-role. FR-271 defines unknown_profile as 'An unrecognized or wrongly-roled profile selection'. So a cause for exactly this case already exists in merged text. FR-272's table does call itself 'proposed, not adopted', which is why this is medium, not high. The spec does not mention the conflict. Fix: use unsupported-selection, or state explicitly that the relayed ruling replaces FR-272's variant and list that among the QSpec-pending items. Separately, the prose attributes unknown_profile to 'QSpec FR-322's closed refusal vocabulary'. FR-322 never names it; FR-271 does. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1093-1104, 1836 |
| FND-023 | low | The timed bullet says it 'refuses {lower, upper: null} (QSpec FR-255: a finite upper bound), also unbounded operators (QSpec FR-250-AC-6)'. That reads as also refusing unbounded operators, which contradicts its own 'admits interval: null' and FR-250-AC-6/FR-090-AC-7, where timed admits the unbounded grammar. Reword to 'and admits unbounded operators'. The same bullet also says a non-integer timed bound 'is refused as a member shape today', while AC-104 'pins no outcome' for it. Pick one. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1078-1086 |

## Dispositions (round 3)

Reviewed at 9a5d5b57461dccb1c144144136d7b078393a57db.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-018 | fixed | 9a5d5b5. Every relayed ruling is now framed as 'a QSpec owner ruling relayed via the IR planner, not yet in merged QSpec text, and an IR reading until it merges'. This covers the type-mismatch entry, the member:<Ident> segment, the reference locus, the fairness outcomes, the per-profile fit, unknown-profile, case placement and details. No unmerged QSpec text is cited by id: FR-250, FR-255, FR-090, FR-362, FR-370 and FR-440 are all on QSpec main b1da9c8. The rulings still have no durable record id. That is now honestly stated, so they are listed as items waiting on QSpec. |
| FND-019 | fixed | 9a5d5b5. AC-108 backs the rule. The known set is QSpec FR-250's merged Values table, and I verified its five identities match the spec text. The breaking-change paragraph now names CheckedPackageRefusalCode::UnknownProfile. The fixture change is stated: v2_all_families moves its law and its profile_selections row to quire.temporal.event-position.false-extension/v1, a bounded profile that fits the closed {0,3} interval (FR-250-AC-6, FR-090-AC-7). The locus and the first-among-clause-checks order are flagged as IR readings. The cause is FND-022. |
| FND-020 | fixed | 9a5d5b5. Each case now maps to its cause and pointer, as in AC-70: a list with only Named, or only Tagged, refuses operation-law-missing at operation.leaves; both leaves plus a repeated Named, or plus [member:Empty], refuse operation-law-mismatch at operation.leaves/2. |

## New findings (disposition pass 4)

Reviewed at bf026d733a074d44e3f47a3fcbeb01303c07c6db (delta 9a5d5b5..bf026d7, through 3f35129).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-024 | low | 'A known definition of another role' is scoped to the package and diagnostic interpretations, quire.package.composed/v1 and quire.native.diagnostics/v1. FR-271 defines unknown_profile as 'an unrecognized or wrongly-roled profile selection', and gives the package/diagnostic case only as an example. Other identities in AD-003's accepted hierarchy are known profiles of another role when named in a temporal_profile law: quire.temporal.bounded-facet/v1, quire.protocol.complete/v1, and the protocol_profile the fixtures themselves select, quire.protocol.finite-global/v1. As written they get unsupported-selection; FR-271's wording points to wrong-selection-role. Name the known set (AD-003's table plus the catalogued profile definitions), or flag the narrowing as an IR reading. Also, the locus and first-check order are 'confirmed by the QSpec owner (relayed via the IR planner)' with no 'not yet in merged QSpec text / IR reading' tag, unlike every other relayed ruling. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1102-1116, 1850 |
| FND-025 | low | The prose does not place the negative-bound refusal in the temporal step's order. {0, -2} carries two defects, a negative upper and lower > upper (0 > -2). AC-97 pins it to the bound (.../interval/upper), so the negative check must run first, but neither the prose nor AC-102 says that. The prose also does not say whether the negative check sits in the 'interval bounds' stage or elsewhere. So it is unclear whether, under a bounded profile, profile fit or the negative bound wins for {-1, null}. State that the negative-bound check is part of interval bounds and runs before lower > upper, and that profile fit runs earlier. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:909-917 |

## Dispositions (round 4)

Reviewed at bf026d733a074d44e3f47a3fcbeb01303c07c6db.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-021 | fixed | bf026d7. AC-97 no longer admits {-1,3}. A negative lower or upper bound now refuses invalid_package/invalid-value at the bound, in line with merged FR-255 and FR-091 (0 <= a). The tightened schema pattern is framed as a relayed ruling, pending QSpec merge. TC-048 adds {-1,3}, {0,-2} and {-5,-2} with their pointers. One gap remains, recorded as FND-025. |
| FND-022 | fixed | bf026d7. No unknown-profile cause any more. The causes are merged FR-272's own pairings: unknown_profile/unsupported-selection for a law naming no FR-250 member, and unknown_profile/wrong-selection-role for a known definition of another role. I checked quire.package.composed/v1 and quire.native.diagnostics/v1 against AD-003's accepted profile hierarchy (row 'Package and diagnostics') and against FR-271's example ('A known package/diagnostic interpretation used as a clause profile is a wrong selection role'). The attribution is corrected to FR-271. I measured that CheckedPackageRefusalCode has no UnknownProfile and CheckedPackageRefusalCause has neither UnsupportedSelection nor WrongSelectionRole (shared.rs at e8db087). The breaking-change paragraph names UnknownProfile; the two causes are additions to an enum no consumer matches exhaustively (checked in round 0). The scope of 'another role' is FND-024. |
| FND-023 | fixed | bf026d7. The timed bullet now says interval: null is 'an unbounded operator, which QSpec FR-250-AC-6 admits under this profile'. The prose and AC-104 now agree that no outcome is pinned for a non-integer bound or an open end. Fairness under timed is admitted as a relayed ruling, and AC-104 says only the three bounded profiles refuse a non-empty fairness argument. These are consistent with FR-250-AC-6 and FR-090-AC-7. |

## New findings (disposition pass 5)

Reviewed at 3f3b00f1eee4104fae1d7b33545cd2e396174ace (delta bf026d7..3f3b00f, through 5fe8022).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-026 | low | Not blocking for this spec PR, but blocking for the code PR's AC-108. The spec says, correctly, that it does not know how the reader obtains QSpec's definition catalog, and that the reader 'holds no list'. Yet AC-108 pins concrete causes that depend on that catalog: wrong-selection-role for four catalogued identities, and unsupported-selection for 'catalogued nowhere' identities such as quire.fixture.temporal-profile/v1. Until QSpec answers the access question, the only ways to implement those clauses are a hardcoded list, which the spec forbids, or reading the QSpec checkout at runtime, which the spec does not state. Mark AC-108's catalog-dependent clauses as pending that answer, as protocol.complete already is, so the code PR does not invent a mechanism. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1121-1142, 1879 |
| FND-027 | low | 'At schema validation' is not a stage of this reader's documented order. The spec places it only 'before placement and every temporal step', not relative to wire decode, graph shape, stale-node-key, declaration or frame/state refusals. There is also an asymmetry: '-1' (a pattern failure under the tightened schema) refuses invalid-value first, while '1.5' (a pattern failure under the current schema) refuses operation-member-mismatch at the operation step, last. Name the stage in IR's order (for example, the body term walk) and say that the asymmetry is deliberate, per the relayed ruling. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:909-921 |

## Dispositions (round 5)

Reviewed at 3f3b00f1eee4104fae1d7b33545cd2e396174ace.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-024 | fixed | 3f3b00f. The hardcoded wrong-role list is gone. 'Known' now means catalogued in QSpec's native Quire v1 definition catalog (the proposals/quire-v1/definitions/README.md table plus the complete-value lock), read at the checkout and not copied. I verified the author's measurement at QSpec b1da9c8: quire.temporal.bounded-facet/v1, quire.package.composed/v1, quire.native.diagnostics/v1 and quire.protocol.finite-global/v1 are in the README table, while quire.protocol.complete/v1 is in neither the README nor complete-value-lock.json (only AD-003 names it). AC-108 pins no outcome for protocol.complete and flags the discrepancy. The relayed locus and order now carry the 'not yet in merged QSpec text' tag. How the reader obtains the catalog is flagged as open and no mechanism is invented. The consequence of that is FND-026. |
| FND-025 | fixed | 3f3b00f. A negative bound is now a schema-pattern failure, refused at the bound before placement and every temporal step, under every profile, first in member order. lower > upper stays in the bounds stage, after profile fit. AC-97, AC-102 and TC-048 pin {0,-2} at upper, {-1,-3} at lower, {-1,null} under bounded, infinite-trace and beside a lower-digest placement defect, and a {3,0} profile-fit-first case. Where 'schema validation' sits in IR's order is FND-027. |

## New findings (disposition pass 6)

Reviewed at dd74358160ab74dcdda001e0d2e07d105c5ef53c (delta 3f3b00f..dd74358, through 01bc74a, b4d2cee, a2a6db1).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-028 | low | Not blocking. The 'Path and locus' table opens by saying its refusals carry a locus 'as the frame-entry refusals do (FR-040)'. FR-040 says 'the locus of an entry's refusal is its declaring node key', and the existing frame code (frame.rs, refused_at with entry.digest) uses the named key even when it names no node. The two no-node rows depart from that: `over` naming no node, and a fairness `declaration` naming no node. Both use the holder's key (the clause's or the fairness node's), and #269's temporal.rs does the same. It is a defensible choice, and the closing note calls the pairing an IR reading, but the table should name it as a departure from the FR-040 convention rather than claim to follow it. Also, the closing note still refers to 'AC-103's and AC-104's "at the clause node"', a phrase neither AC contains any more (AC-103 now names paths, AC-104 says 'at the clause's application'). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1169-1194 |

## Dispositions (round 6)

Reviewed at dd74358160ab74dcdda001e0d2e07d105c5ef53c.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-026 | fixed | dd74358. The catalog dependency is gone. The restated rule decides the cause from the package alone: a temporal_profile law whose definition is not one of FR-250's five refuses unknown_profile; the cause is wrong-selection-role when the same {authority, identity} is a lock.profile_selections row under another role or is the lock.edition row, and unsupported-selection otherwise. A known profile is matched by identity label only, so another authority falls through to AC-57's operation-law-unselected. I checked the reader's field names. CheckedSelection has role and definition, and edition is a separate lock row. CheckedSelectionRole is exactly language, edition, profile, binding_contract, temporal_profile, protocol_profile, which the spec lists. #269 at ddaf828 implements it as stated: temporal.rs selected_under_another_role compares lock.edition.definition and each non-temporal_profile row, and TemporalProfile::from_wire matches identity only. |
| FND-027 | fixed | dd74358. The stage is named: the term walk of the node body, after the strict parse, the canonical-bytes check and the decode, and before the identity checks, the temporal step and the operation step. The '-1' versus '1.5' asymmetry is stated as deliberate in the prose, AC-102 and TC-048. |

## New findings (disposition pass 7)

Reviewed at 312de4505fdd2391a395bad70413ded8b10fd677 (delta dd74358..312de45, through 751ed09, 00af625 and adb764e; read against merged QSpec f39c93f).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-029 | high | Blocking: this contradicts merged QSpec. Merged FR-370 at f39c93f says 'A bound outside its form's pattern, including a negative integer bound ... fails the published schema and refuses invalid_package/invalid-value at that bound's pointer during strict wire validation, before any step of this requirement runs ... never operation-member-mismatch'. The merged schema binds integer-form bounds to NonNegativeIntegerString ^(0/[1-9][0-9]*)$, so '1.5', '01', '+1', '' and '3x' are bounds outside their form's pattern, and so is a JSON integer 0 in place of the string. AC-97 still refuses each of these operation-member-mismatch at operation.member. AC-102 pins {lower: '1.5', upper: '0'} to operation-member-mismatch, 'never invalid-value', and attributes 'the asymmetry is deliberate' to FR-370-AC-9, which says the opposite. Fix: refuse every bound outside its form's pattern invalid-value at the bound in the term walk, as negatives already are, and drop the asymmetry. Otherwise list it as a sixth unreconciled difference, without citing AC-9 for it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:928-931, 2016, 2021 |
| FND-030 | medium | Blocking. The reader order for case placement contradicts merged text and is not in the PR's list of differences. Merged FR-440 says 'The reader checks these joins at FR-322's operation step, in ascending node-id digest order', and join 1 is case placement. This spec checks a misplaced case application in the temporal step's placement pass, before the operation step, and calls that 'an IR reading'. As a result a misplaced case is reported ahead of a temporal clause defect, where merged order reports the clause defect first. Either move case placement to the operation step, or list the order as a difference QSL rules on. (Difference (3) itself, an expression-form contradiction refusing invalid_semantic_graph at the body, is applied at 312de45.) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1255-1273 |
| FND-031 | medium | The locus of a null member contradicts merged text. Merged FR-370 says 'An interval-capable operator whose member is null refuses invalid_package/operation-member-mismatch at the application' (/semantic_graph/nodes/{n}/body). AC-97 refuses a null member at operation.member, and AC-102 says a null member under a bounded profile 'refuses the same way' (at operation.member). Move the null-member locus to the application, or list it as a difference. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2016 |
| FND-032 | medium | 'Merged text does not say what a declaration that names no node at all refuses' is doubtful. Merged FR-370 'Fairness resolution' step 1 reads: 'The declaration target is a model node with semantic_form object_type, else invalid_model_binding/malformed-declaration, located at the target.' A declaration naming no node falls under that 'else', which gives malformed-declaration, not the spec's missing-name. The PR lists missing-name as 'merged QSpec does not state'. Either follow step 1, or record this as a reading that departs from step 1's literal text and put it in the differences list. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1081-1099 |
| FND-033 | low | Not blocking for this spec, but it needs routing to QSL. The PR body now records difference (4) as resolved by the QSL ruling relayed 2026-10-03: the recursion:<d> entry stays, as for records. The spec prose still calls it 'an unreconciled difference, listed', and still says merged text 'does not address unions with text'. In fact merged FR-322 'Structural leaf walk' says of every recursion leaf, 'through a union as through a record or tuple', that the walk 'lists no entry for it'. That also contradicts this repository's implemented record rule (FR-038-AC-70 through AC-72). Fix: relabel the prose as the 2026-10-03 ruling. QSL's FR-322 correction must restore the recursion entry for records and tuples as well as unions, or AC-70 to AC-72 stay in conflict with merged text. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1659-1673 |
| FND-034 | low | Not blocking. Three wording defects. (1) AC-103 still calls the ambiguous, inherited and unrecovered-owner outcomes 'QSpec owner rulings relayed via the IR planner and not yet in merged QSpec text'. Merged FR-370 'Fairness resolution' and FR-370-AC-11 now state all three. (2) The nested-case ruling says the nested case refuses at its own operator 'as the misplaced-application rule does', but that rule (FR-370, and the table's first row) refuses at the node that holds the application, not at an operator. (3) The details bullet says a reference to an expression/case node 'is not named by the ruling' and pins no outcome. It should also say that merged FR-370-AC-12 does refuse that reference, so the gap is visible. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2022, 1261-1292 |

## Dispositions (round 7)

Reviewed at 312de4505fdd2391a395bad70413ded8b10fd677.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-028 | fixed | 312de45. The table now cites QSpec FR-047 and this repository's FR-040, quotes the loci merged FR-370 states (the target, the referencing node, the clause node for the law), and names the two no-node rows as a deliberate departure from FR-040's named-key locus. The stale 'at the clause node' reference is gone. It also adds rows for references and applications in `details`, with the entry pointer as locus. |

## New findings (disposition pass 8)

Reviewed at 3c4821868cab2cbecd1da21c320ba35e68c91ae8 (delta 312de45..3c48218; read against merged QSpec f39c93f).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-035 | low | Not blocking: stale wording. (1) The TC-048 matrix row still lists 'a fairness declaration naming no node' among IR readings that merged QSpec does not state, but FR-038 now follows merged FR-370 step 1 for it. (2) The IR-551 note under timed/v1 describes today's reader (ddaf828) as refusing a timed-form interval 'operation-member-mismatch at operation.member as any member shape that does not decode'. Under the new term-walk rule, a timed-form bound is a non-string outside the integer pattern, so it would refuse invalid-value at the bound first. The note is explicitly 'not a requirement' and IR-551 owns the form, so only the description is off. (3) FR-038 still calls the expression/case details reference and the recursion-leaf contradiction 'open QSL items'; if QSL has confirmed both, as relayed, relabel them when the QSpec follow-up lands. | spec/checked_package/matrix/tests.md:23 |

## Dispositions (round 8)

Reviewed at 3c4821868cab2cbecd1da21c320ba35e68c91ae8.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-029 | fixed | 3c48218. Every bound outside merged FR-370's pattern, negative or malformed ("1.5", "01", "+1", "", "3x", a non-string), now refuses invalid_package/invalid-value at that bound in the term walk, first in member order, never operation-member-mismatch. The pattern is ^(0/[1-9][0-9]*)$, matching merged schema.json NonNegativeIntegerString. The asymmetry text and its AC-9 citation are gone, and AC-97, AC-102 and TC-048 are consistent. A fairness-kind member and an interval with a third member still refuse operation-member-mismatch at operation.member, labelled 'an IR reading, merged text silent'. |
| FND-030 | fixed | 3c48218. Case placement now runs at the operation step, as merged FR-440 'Reader joins' places it, in ascending node_id order and as the first join of its node. The temporal placement pass covers the three temporal classes only. The order consequence is stated: every temporal-step defect is reported before a case placement defect. AC-100 says the same. The nested case stays in the term walk at the nested operator, by the 2026-10-03 ruling. |
| FND-031 | fixed | 3c48218. A null member on an interval-capable operator now refuses invalid_package/operation-member-mismatch at the application (/semantic_graph/nodes/{n}/body), as merged FR-370 says, in the prose, AC-97, AC-102, TC-048 and a new table row. |
| FND-032 | fixed | 3c48218. This follows merged FR-370 'Fairness resolution' step 1: a declaration whose target is not a model/object_type node, including a key that names no node, refuses invalid_model_binding/malformed-declaration located at the target, with the named key as locus. missing-name is used only for an unmatched name (step 3). AC-103 and the table agree, and AC-103 now cites FR-370-AC-11 for the outcomes. |
| FND-033 | fixed | 3c48218. Relabelled. Merged FR-322 'lists no entry' is stated for every recursion leaf, records and tuples included, as contradicting the implemented FR-038-AC-70 to AC-72. The entries stay for records and tuples, and for unions by the 2026-10-03 ruling. QSL's FR-322 follow-up must restore the record entry. It is no longer called 'not addressed for unions'. |
| FND-034 | fixed | 3c48218. All three parts are fixed. (1) AC-103 now cites merged FR-370-AC-11 and 'Fairness resolution' for the ambiguous, inherited-admitted, unrecovered-owner and malformed-declaration outcomes. (2) The nested-case text now says the ruling's operator pointer differs from FR-370's holding-node rule. (3) A details reference to an expression/case node refuses at .../details/{d}, as merged FR-370-AC-12 says. |

Final status at 3c48218: every finding from FND-001 to FND-034 has a latest outcome of fixed. The one open finding, FND-035, is low and non-blocking. #268 is mergeable into the #253 branch.

## New findings (disposition pass 9)

Reviewed at 6f40ae4d639c18e94d60e11acc2ace767001e2ef (delta 3c48218..6f40ae4; the #269 reader measured at ddaf828 and 0209033).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-036 | low | Not blocking. The IR-551 note claims, as what 'the IR-549 code (the held code change, head ddaf828…)' does today, that a timed-form interval's non-string bounds are refused invalid-value at that bound in the term walk. I measured that this is false at ddaf828 and at the current #269 head 0209033. In temporal.rs, negative_interval_bound refuses only negative integer strings (as_str then IntegerString::parse then negative), so an object bound passes the term walk. read_interval_member then returns None for it, and the operation step refuses operation-member-mismatch at operation.member. The note before 6f40ae4 was the accurate one; I caused this by misreading in FND-035 (2). Once #269 implements AC-97's every-bound-outside-the-pattern rule (it still does not: '1.5' is also member-mismatch at 0209033), the new sentence becomes true. Fix: attribute it to the reader after the code change, or restore the ddaf828 description. Separately, and for the #269 review: the code must implement AC-97's malformed and non-string bound refusal, AC-103's no-node fairness locus at the named key, and AC-100's case placement at the operation step. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1142-1160 |

## Dispositions (round 9)

Reviewed at 6f40ae4d639c18e94d60e11acc2ace767001e2ef.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-035 | fixed | 6f40ae4. Parts (1) and (3) are fixed. The TC-048 matrix row no longer lists the no-node fairness declaration among IR readings. The expression/case details reference, the recursion:<d> entry and the nested-case items are relabelled 'QSL confirmed 2026-10-03; QSpec text follow-up pending'; each still states the merged-text position and that the QSpec text follow-up is pending, so no pending claim was weakened. Part (2) was my own error. I described what the spec now requires, not what the ddaf828 reader does. The author followed my wording, and the note is now wrong in its turn (FND-036). I measured that 6f40ae4 changes only these wording spots: AC-100 differs only in its label, no other AC row changed, and no AC was added. |

Final status at 6f40ae4: every finding from FND-001 to FND-035 has a latest outcome of fixed. FND-036 is low and non-blocking, a misattributed descriptive note that is not a requirement. #268 is mergeable into the #253 branch. This file is copied into reviews/ next.

## Dispositions (round 10)

Reviewed at 5774b4c2001cbcf5c8829915713910b4eab325c8.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-036 | fixed | 5774b4c. The IR-551 note is now accurate whichever lands first. Before AC-97's rule is implemented, the reader at ddaf828 (and #269's 0209033) catches only negative integer strings in the term walk (negative_interval_bound: as_str, then IntegerString::parse, then negative). So an object bound passes the term walk, read_interval_member returns None, and the operation step refuses operation-member-mismatch at operation.member. After the rule is implemented, AC-97 refuses every bound outside the pattern, non-strings included, invalid-value at the bound in the term walk. Either way a timed-form interval under timed/v1 is refused until IR-551. The note is still labelled 'not a requirement'. 6f40ae4..5774b4c changes only this note (one file, one hunk). |

Final status at 5774b4c (final): every finding from FND-001 to FND-036 has a latest outcome of fixed, and none is open. #268 is mergeable into the #253 branch. This file is final and is copied into reviews/.
