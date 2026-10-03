---
id: SR-1041
title: "gap analysis of PR 268 against IR code, QSpec fixtures and downstream consumers (IR-549)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@ca2a5454fa3b3d9fcf683606909ab5122d4207d1; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md; base feat/artifact-ref-authority-identity e8db0876; read against quire-specification@b1da9c8 FR-370, FR-440, FR-441, FR-322, schema.json, node-identity-preimage.schema.json and fixtures"
review_set: subset
---
# SR-1041: gap analysis of PR 268 against IR code, QSpec fixtures and downstream consumers (IR-549)

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
| FND-001 | medium | The code change breaks a downstream test, and the spec does not record it. quire-spec-language qsl-package/src/emit/tests/admission_corpus.rs classifies every CheckedNodeKind::all() and says 'A new IR kind fails until it is classified'. The four new forms (composite_type/union, value/union_value, expression/case, temporal/fairness) will fail that test on QSL's next IR bump. Separately, qsl-package/src/checked_v2.rs:493 map_refusal_code matches CheckedPackageRefusalCode exhaustively, and the enum is not non_exhaustive. UnsupportedConstruct exists only on #253's branch (IR main at 0a889f9 has none), so removing it inside #253 before #253 merges is safe. If #253 merges first, QSL breaks on adding the variant and again on removing it. Route this to QSL, and keep IR-549's code in #253 as the ticket says. CG matches none of these enums (checked). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:944-961 |
| FND-002 | low | 'The code change removes the variants' should name the refusal types only: CheckedPackageRefusalCode::UnsupportedConstruct (shared.rs:129) and CheckedPackageRefusalCause::ExpressionForm (shared.rs:207). The diagnostics wire vocabulary CheckedDiagnosticCode::UnsupportedConstruct (v2/mod.rs:250) must stay, because QSpec schema.json's Diagnostic.code enumerates unsupported_construct. Removing it would refuse QSpec-valid diagnostics entries. Say so in AC-101. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1676 |

## Verdict

Changes requested, medium. IR's code claims in the PR hold at e8db087. The
dropped refusal code and cause variants exist only on PR 253's branch, so no
merged consumer depends on them.

FND-001 records a real downstream break. The four new node kinds will fail
QSL's admission-corpus test on QSL's next IR bump. Route that to QSL, and keep
IR-549's code inside PR 253, as the ticket says.

FND-002 is a wording guard: the diagnostics code UnsupportedConstruct must
survive the code change.

The rest of the gap work is recorded in SR-1040 and in the Summary list of
tests the code PR must change.

## Dispositions

Round 1, reviewed at e5da11e53bfe7c4e49c517d4e41a4bd09faa73c6.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | e5da11e. The spec now records the consumer break (CheckedNodeKind classification, exhaustive CheckedPackageRefusalCode) and the shipping rule: the change ships inside #253 before it merges. Opening the QSL ticket is still the coordinator's job; the spec side is done. |
| FND-002 | fixed | e5da11e. AC-101 names the refusal types and keeps CheckedDiagnosticCode::UnsupportedConstruct, and TC-048 reads a diagnostics entry whose code is unsupported_construct. |
