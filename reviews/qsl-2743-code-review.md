---
id: SR-967175026302197768900474181941935084514
title: "QSL-2743 PR 1117 code-review (Rust lane) of the model-union reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-contract-ir; QSL-2743; PR 1117 head 31a74a3c6d48f8f272f82092cd28ba9e377d2516; crates/filament-contract-model/src/checked_package/v2/{exact_projection.rs,exact_projection/*,native_join_storage.rs,runtime_type_projection.rs,model_members.rs,monitor_plan/*}, tests/conformance_fspec/model_union_witness.rs, tests/conformance_fspec/main.rs, tests/conformance_fsl_ir1384/tests/receipt.rs, crates/filament-contract-model/tests/monitor_plan_public.rs, spec rows"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-contract-ir/FR-352", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-358", type: references }
  - { target: "ix://agent-ix/filament-contract-ir/FR-069", type: references }
---

## Summary

Code review of PR 1117 (agent-ix/filament-contract-ir), the FR-830 model-union reader, diff origin/main...31a74a3. Ticket: QSL-2743. The core join is sound, but refusal loci, budget vectors, retention, row-index loci, payload coverage and storage precharge fall short of the FR-830 contract, and the tests cannot detect several of these gaps.

## Verdict

**FAIL** — two high findings (FND-001 still open after a partial fix, FND-002) plus medium findings on payload forms, precharge, loci and coverage. MERGEABLE=FALSE.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Refusal loci do not match the published vectors and no test checks them. The refusal test compares only expected.code and ignores expected.locus, while FR-830 AC-7/8/9/13/14 and FR-352-AC-31 require code and locus. Wrong pointers as found: model_union.rs:672 emits /member (MUW-R-17/18 expect /member/declaration and /member/identifier); 860/876/896/918/940/962 emit a bare /payload/node with no /model_unions/{k}/cases/{j} prefix (R-43/46/51/54/55, TC-02/03/05); 497 (R-01) and every tuple refusal from with_admitted_type_rows (TR-01..07) carry form selected_identity, not a /model_unions or /model_tuples/{k} pointer. A mutant that swaps /rank and /position passes. | tests/conformance_fspec/model_union_witness.rs:393-401 (e8ba885f7); crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:497,672,860-962 (e8ba885f7) |
| FND-002 | high | The published budget_cases MUW-B-01..B-08 are never executed. The budget test asserts only outcome codes for baseline, one-below and zero limits on three positives; it never asserts the first unpaid kind, full attempted amount or locus, and omits B-02 (closure-edge boundary Ec-1/Ec), B-03 (cumulative across joins), B-04/B-06/B-07 (charges kept after a refusal, no rows), B-05 (tuple/union channel split at Wt) and B-08 (reached with an unreached row: surplus only common work). FR-830-AC-11 and FR-358-AC-15 are largely unverified. | tests/conformance_fspec/model_union_witness.rs:426-462 (e8ba885f7) |
| FND-003 | medium | The positive test only checks each returned row is contained in the input. A change that retains out-of-closure rows under reached scope (MUW-03, MUW-16) or returns extra rows (MUW-19 returns only the model row) passes; dropping all retention passes for every reached vector. The published closure_edges counts (AC-3: each node expanded once) are never asserted. Only the MUW-01 full-scope projection test asserts an exact count. | tests/conformance_fspec/model_union_witness.rs:330-336 (e8ba885f7) |
| FND-004 | medium | Native denials carry no sidecar row-index locus. The union phase never sets a locus source; in the reached monitor path with_admitted_type_rows sets TupleRow(index) over the scope-filtered joined_tuples, not the original sidecar index, and that source stays active, so a later union-phase denial is attributed to the last tuple row. FR-830 Work and storage requires the denial's compact locus to name the channel (model_tuples vs model_unions), the sidecar row index and the form. | crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs:85; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:456,487-890 |
| FND-005 | medium | Admitted non-Clock payload forms are refused. model_union_payload_node returns None for Option, Collection, IntRange, RationalRange, TextBounds, CarriedRational and ClockReference, and model_union_expected_type refuses an enum-family variant payload; all become UnsupportedExactType. FR-830 says a data payload is any admitted type but Clock and the original value-type grammar includes option, collection, map, bounded and carried-quantity forms. A full join over a package with a Rational-range payload (TK-36, FR-069-AC-8) cannot succeed. No vector covers these forms; that does not narrow the contract. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:247-263, 871-877 (e8ba885f7 numbering 247-263, 870-878) |
| FND-006 | medium | The monitor and runtime threading has no test with real rows. Every ReachedProducerRows and SelectedMonitorClosureRows site passes model_unions: &[] (reached_model_read.rs tests, monitor_plan_public.rs:70, receipt.rs:284), so with_model_union_rows_over and SelectedTypeDeclarations::unions() never run with a row. FR-352-AC-31 also claims a record or object field of an admitted union returns Composite(type_handle) (exact_projection.rs:3429); no test exercises it, nor a tuple position or optional field of union type. | crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs:708-719; crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs:1211-1220; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3429-3437 |
| FND-007 | medium | The preflight handle index covers only rows already retained by the receiver, not the supplied rows. FR-830 Work and storage puts the handles of every supplied tuple and union witness, across both channels, in the preflight index of the common pass. Under reached scope a joined row whose type_handle or settled_key collides with an unreached supplied row's handle is not detected, and the B-08 claim that the surplus is preflight-index work is not what the code does. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:281-345 |
| FND-008 | low | Misleading comment: it says the fixture-C source-row non-substitution controls are asserted by the native tests of the crate, but no such native tests exist in the diff or the crate. | tests/conformance_fspec/model_union_witness.rs:356-357 (e8ba885f7) |
| FND-009 | low | One clone is charged twice and one check is duplicated. selected_identity_handles charges storage::selected_clone() at 405-411 while with_admitted_type_rows charges it again for the only clone (complete_witness.rs:73-80). The tuple-vs-union key check at 457-469 duplicates the model_unions comparison check_handle already makes (complete_witness.rs:302-305) and uses the unbudgeted witness_error with form selected_identity. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:405-411, 457-469 |
| FND-010 | low | validate_and_rebind ignores model unions. validate_contents does not re-check model_unions rows and validate_completeness requires no row for a union-family variant_type node, while it does require admitted_tuples rows. FR-352-AC-28 says missing emitted declarations or unsupported identity channels refuse rather than yield a partial complete witness, and FR-830 full scope requires one row per union-family node; a rebound witness can be complete without the rows direct projection needs. Matching the older model-enum omission is not contract scope. | crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs:652-740; 409 |
| FND-011 | low | A shared helper changed behaviour for every caller. selected_declaration_error and its budgeted twin now return UnknownNode instead of ForeignNode for any node whose package equals a selected model document identity. This affects all joins and projections that use the helper, not just the model-union join, and only the FR-830 vectors exercise it. | crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:2976-2990, 2999 |
| FND-012 | low | Spec-row wording. FR-352-AC-31 says a foreign selection refuses UnknownNode, which is ambiguous beside FR-352-AC-30 (identities from an independent admission refuse ForeignNode). FR-358-AC-15 says every row is charged in the common pass before allocation, but FR-830 and the code allocate the preflight handle index first. None of the three new rows carried the status prefix sibling rows use. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:627 (e8ba885f7) |
| FND-013 | medium | The reached closure clones before it pays. The start node is cloned twice (pending and visited) under a flat charge of 3 storage units, not its node size; each new target is cloned twice (visited.insert and pending.push) under one node_units+2 charge; the per-node edges and members Vecs grow with no storage charge. A long start or target CheckedNodeId under a tight storage limit allocates beyond what the budget admitted, violating precharge-before-clone. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:150-157, 165-166, 221-233 |
| FND-014 | medium | Staged and index storage is under-charged. staged.insert clones the structural node as the map key, but row_units (native_join_storage::model_union) counts the structural node once, for the row copy; the tuple path charges the key separately (complete_witness.rs:121-124, storage::node(id)). The handle-set inserts at 470 and 775-776 and the joined_handles BTreeMap at 472-478 allocate with no storage charge. | crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs:605-612; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:470,477,774-777 |
| FND-015 | medium | Uncharged scans. The coverage loop walks every selected declaration and calls variant_family (which scans the variant's cases) before any work charge, charging only matching tuple and union nodes; the tuple-vs-prior-union check at 457-469 scans every retained union for every joined tuple with no charge (T x U). These are whole-model and quadratic blind spots in the shared work budget. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:457-469, 780-787 |
| FND-016 | medium | The typed source non-substitution controls are skipped as if producer-only. MUW-19, MUW-20 and MUW-R-59..R-62 need only typed SelectedSourceUnionRow and SelectedSourceTupleRow rows and the existing native source joins, which IR already owns; the FSL parser boundary covers the closed format only. Nothing shows a valid source_unions or source_tuples row never supplies a missing model row, or that MUW-20 admits beside a source tuple. | tests/conformance_fspec/model_union_witness.rs:377-381 |
| FND-017 | medium | The -TR- locus exemption drops required coverage. IR owns the typed model_tuples join and knows each row's sidecar index before filtering, so /model_tuples/{k}/... is IR's coordinate to report, not an FSL adapter pointer. The substring match also exempts MUW-TR-08, whose expected locus is the union pointer /model_unions/1/type_handle. Coverage vectors' expected.node is never compared with error.at. | tests/conformance_fspec/model_union_witness.rs:420-428 |
| FND-018 | low | FR-019 inventory omits part of the public change: the new public field SelectedMonitorClosureRows.model_unions is a breaking change for struct-literal consumers (receipt.rs and monitor_plan_public.rs had to change) but is not in the breaking-change prose, and the associated with_model_union_rows join, model_union_rows/model_union_row accessors and SelectedTypeDeclarations::unions() are not named. | spec/model/functional/FR-019-rust-library-interface.md:311, 316, 318 |

## Scope and evidence

Rust lane (rust-review, rust-style baseline) folded into this artifact. FND-001 to FND-012 are the original review findings as found at e8ba885f7 (their lines and excerpts cite e8ba885f7). FND-013 to FND-018 were added in this formal pass at frozen head 31a74a3, covering the lead verification items: closure precharge, staged and index storage, uncharged scans, typed source controls, the -TR- exemption and the public inventory. Round-1 dispositions cover only the partial fixes in 31a74a3: locus (FND-001), the misleading comment (FND-008) and spec wording (FND-012). Every other finding is open.

Held clean (examined, no finding): atomic tuple-first join and single copy-on-write publish (model_union.rs:456, 824-846); prior-Arc copy charged before Arc::make_mut; union rows on source-union, enum-family or tuple nodes refuse Mismatched; union projection branches precede the enum-shape lookup (exact_projection.rs:3429, 4284); type_handle and settled_key kept in separate roles; canonical rank and authored position checked apart; UnionMember never an enum VariantId; nullary, Clock and data payloads never substitute; the closure keeps a visited set and expands each node once; the FR-531-AC-13 authority wording is verbatim at model_union.rs:5-6 and 357-358; the model-enum test supersession only replaces the old unsupported refusal with the missing-witness refusal, and an enum-only mask is still refused; source-union, source-tuple and dimensional APIs are unchanged.

Gates: no cargo was run by this reviewer (host thermal rule; no aggregate, workspace or clippy gates). Coder receipts read, not re-executed: /tmp/ir2743-t4.log (conformance_fspec model_union_witness, 4 passed) and /tmp/ir2743-t5.log (exact_projection, 16 passed). t4 is timestamped 06:51 and head 31a74a3 was committed 06:55, so the receipt may predate the final commit by one edit.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-019 | high | Two implementations of one rule (introduced in 758a2cc4d/1be399aa5). model_union_payload_node re-implements MemberType::node_id (the FR-322 step 4 structural-id derivation) form for form, so that it can charge before allocating. payload_nodes_are_the_structural_ids_the_member_types_own is a drift detector over that copy. model_union_expected_type re-implements the leaf and Option/Collection wrapper projection beside project_record_field_type, and the two now disagree: a Float32/Float64, Bytes, JsonObject, RationalRange, TextBounds or CarriedRational payload projects to an Exact type, while the same member type as a record field refuses UnsupportedExactType (exact_projection.rs:3448-3463). A fix to either derivation must be made twice. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:1402-1660, 1667-1831; tests 1998-2038, 2165-2199 |
| FND-020 | medium | Closure-edge charges never carry the row locus FR-358-AC-17 states. model_union_closure sets the active locus to NativeJoinLocusSource::Package with form model_union_closure or model_union_closure_type, and model_union_follow charges each edge under that locus, so an Ec-1 denial names Package, not the ModelUnionRow being closed or the edge (FR-830 B-02: its locus names that edge). budget_case_b02 only compares the denial with the baseline event's own locus, so it cannot fail on this. | crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:333-337, 366-370, 391-395, 321 |
| FND-021 | low | The test name and doc overclaim: 'every field form' covers only a direct and an optional record field. There is no object-type field, tuple position or collection field, so the name hides the gap recorded in FND-006. | crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs:4374-4377 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Partial fix at 31a74a3: member/identifier vs member/declaration, the /model_unions/{k}/cases/{j} payload prefix and the R-01 structural_node pointer are corrected, and the refusal test now compares error.form with expected.locus. Still missing: every -TR- vector is exempted from the locus check, so tuple refusals keep the tuple-stage form selected_identity (TR-01..07) and TR-08 is unasserted; coverage vectors' expected.node is not compared with error.at. |
| FND-008 | fixed | Fixed at 31a74a3; after_excerpt in the round-1 tracker comment. |
| FND-012 | still-open | Partial fix at 31a74a3: the three rows now carry the Implemented (QSL-2743) status prefix. Still unchanged: FR-352-AC-31 still says a foreign selection refuses UnknownNode beside FR-352-AC-30's ForeignNode, and FR-358-AC-15 still says the common pass is charged before allocation although the preflight handle index is allocated first. |
| FND-001 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-002 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-003 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-004 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-005 | still-open | Round 2 (head 16a686c). 758a2cc4d/1be399aa5/dc266979e implement node derivation and Exact types for Option, Collection, IntRange, RationalRange, TextBounds and CarriedRational, and ClockReference refusing unsupported_exact_type is contract-backed (filament-exact b26a8e4 ValueType has no clock-reference variant; FR-830 step 6: no faithful Exact representation is unsupported_exact_type). Still missing: no strict-admitted end-to-end join for any non-scalar admitted payload. any_admitted_non_clock_payload_is_joined_and_each_wrong_witness_is_refused covers seven scalars only; Option, IntRange, RationalRange, TextBounds, CarriedRational, Duration (Quantity of the duration unit) and enum-family payloads are reached only by unit calls on SelectedExactIdentities::empty(), and CarriedRational and enum payloads, which need unit or enum-shape witnesses, have no expected-type assertion. IR already strict-admits a Rational-range variant payload (TK-36, tests/conformance_fspec/model_tuple_union.rs:360), and the harness's document_patch can add such cases, so the read-only FSpec worktree is no blocker. |
| FND-006 | still-open | Round 2 (head 16a686c). Fixed in part: runtime SelectedTypeDeclarations::unions() over real rows (runtime_selected_types_declare_each_joined_union_by_its_type_handle) and the reached monitor read with a real union row, direct and optional record fields, exact and one-below budget and the no-row refusal (reached_model_read.rs:4377). Still missing: no test projects an object-type field or a tuple position whose type is a model union. The object-field case was removed from the monitor fixture, and FR-352-AC-35 was narrowed to record fields instead of the obligation being evidenced. |
| FND-007 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-009 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-010 | still-open | Round 2 (head 16a686c). Code changed (complete_witness.rs:626-640 re-validates every retained union row through validate_union_row; :755-764 requires a row per union-family node), but nothing evidences it. The only test, rebinding_a_reached_only_witness_is_refused_and_never_mutates_the_receiver, rebinds the MUW-03 witness, which holds only the Pair tuple row of fixture A's four (Cell, Nest, Pair, Swap). validate_completeness requires tuple rows at complete_witness.rs:751 before it reaches the union requirement at :762, so that refusal is selected_model_tuple and the union-completeness stage is never reached. There is no positive full rebind, no isolated missing-union rebind and no content re-validation refusal. FR-352-AC-37 was narrowed to the refusal half. |
| FND-011 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-012 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-013 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-014 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-015 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-016 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-017 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-018 | fixed | Round 2 (head 16a686c). Fixed; the verbatim after_excerpt is in the round-2 data block below and in the round-2 tracker comment. |
| FND-005 | still-open | Round 3 (head 70b2445). Fixed in part (34969ef2d, 8f8bd1c9e): integer range, profiled text, rational range and option payloads now join end to end through strict-admitted packages (checked_package_v2_model_fields.rs every_strict_admitted_non_scalar_payload_form_joins_and_each_wrong_witness_is_refused). Unit quantity, object Reference and record/tuple/union composites join with real witnesses (MUW-11), and enum-family payloads join with a real enum shape. A sequence payload is refused at intake, and ClockReference stays contract-backed unsupported. Still missing: a carried-quantity payload (value type Rational with a unit, admitted at intake through value_types) and a Duration payload (MemberType::Quantity of the duration unit, admitted at intake) have no end-to-end join. No test anywhere asserts a positive ValueType::CarriedQuantity projection, on the payload or the record-field path; payload_and_record_field_project_every_admitted_type_identically compares CarriedRational only as the same refusal on empty identities. |
| FND-006 | fixed | Round 3 (head 70b2445). Fixed; the verbatim after_excerpt is in the round-3 data block below and in the round-3 tracker comment. |
| FND-010 | fixed | Round 3 (head 70b2445). Fixed; the verbatim after_excerpt is in the round-3 data block below and in the round-3 tracker comment. |
| FND-019 | fixed | Round 3 (head 70b2445). Fixed; the verbatim after_excerpt is in the round-3 data block below and in the round-3 tracker comment. |
| FND-020 | fixed | Round 3 (head 70b2445). Fixed; the verbatim after_excerpt is in the round-3 data block below and in the round-3 tracker comment. |
| FND-021 | fixed | Round 3 (head 70b2445). Fixed; the verbatim after_excerpt is in the round-3 data block below and in the round-3 tracker comment. |

## Disposition round 2 verdict

**FAIL — MERGEABLE=FALSE.** Round 2 (head 16a686c): 14 of the 17 open findings are fixed. Still open: FND-005 (no strict-admitted end-to-end join for non-scalar admitted payloads), FND-006 (no object-field or tuple-position union projection), FND-010 (validate_and_rebind union stage and positive rebind unevidenced). New: FND-019 high (duplicated id derivation and projection, already divergent from record fields), FND-020 medium (closure-edge locus is Package, not the row FR-358-AC-17 names), FND-021 low. The tc_058 public_interface failure fails identically on origin/main 743b5e5f1, so it is not attributed to this PR.

## Disposition round 3 verdict

**FAIL — MERGEABLE=FALSE.** Round 3 (head 70b24456c): 5 of the 6 open code-review findings are fixed (FND-006, FND-010, FND-019, FND-020, FND-021). FND-005 stays open: no carried-quantity or Duration payload joins end to end, and no positive CarriedQuantity projection is asserted on either path. The FR-358-AC-17 wording contradiction and the unspecified record-field widening are recorded in the base SR (FND-008, FND-009).

## Machine-readable data

```yaml
{
  "clean": false,
  "scope": [
    {
      "id": "FR-352-AC-31",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a "
    },
    {
      "id": "FR-358-AC-15",
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). The model tuple and model union row join charges one shared `NativeJoinBudget`: every row of both channels is charged in the common pass before allocation, then the selected clone and the prior-copy publish. The measured exact cumulative work and storage succeed and one unit below either refuses at the first unpaid kind and locus with no partial publish; a zero budget refuses before any row is retained."
    },
    {
      "id": "FR-069-AC-11",
      "path": "spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md",
      "role": "examined",
      "excerpt": "Implemented (QSL-2743). A model union variant payload naming an object type of the selected package is admitted as a `Reference` payload, so a union such as `Mix.Held(Holder)` passes intake instead of being refused for an undeclared payload type."
    },
    {
      "id": "FR-352-AC-28",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "context_only",
      "excerpt": "Implemented (IR-1360). A default-feature external consumer calls the checked public `with_admitted_type_rows` and single `validate_and_rebind` signatures with `&mut NativeJoinBudget`. The selected-type declaration projection uses the same checked admitted-type join; the old unchecked copy path is absent. The public surface exposes no private-map getter, unchecked rebind, completeness flag, wrapper or second validator. Full finite package-local checks use native admitted facts, terminate iteratively, perform no source parse, and acquire no FSL/runtime dependency or pin. Missing emitted declarat"
    },
    {
      "id": "FR-352-AC-30",
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "role": "context_only",
      "excerpt": "Direct projection of an admitted selected `model/variant_type` with its complete installed model enum shape returns `ValueType::Enum` equal to that shape, retaining ordering and every opaque variant identity. A package clone agrees. The reached-model join and following projection share one `NativeJoinBudget`: their measured exact cumulative work and storage succeed, and one unit below either refuses at the first unpaid kind and locus without a partial projection. An absent shape returns `UnavailableIdentityWitness`, and identities installed for an independent admission refuse `ForeignNode`. A "
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "role": "examined",
      "excerpt": "Atomic tuple-first join: tuples via with_admitted_type_rows, union staging, coverage, single COW publish; type_handle/settled_key roles; rank vs position; UnionMember not VariantId; Clock vs nullary; cycle-aware closure."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs",
      "role": "examined",
      "excerpt": "Union branches of direct projection (4284) and record-field projection (3429) ahead of the enum-shape lookup; selected_declaration_error classification."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "role": "examined",
      "excerpt": "check_handle and check_admitted_handle cover model_unions in both directions; with_admitted_type_rows locus source; validate_and_rebind."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs",
      "role": "examined",
      "excerpt": "storage::model_union per-row units."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs",
      "role": "examined",
      "excerpt": "with_model_union_rows_over threaded after the enum-shape join under one budget."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs",
      "role": "examined",
      "excerpt": "selected_union requirement and unions() output with storage precharge."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/model_members.rs",
      "role": "examined",
      "excerpt": "scope.object_types admitted as a declared payload name (FR-322 Reference<O>)."
    },
    {
      "id": "tests/conformance_fspec/model_union_witness.rs",
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "role": "examined",
      "excerpt": "Conformance harness: strict read of complete published inputs, production with_model_union_rows, positives, refusals with code and locus, budget baseline, projection."
    },
    {
      "id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_enum_tests.rs",
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_enum_tests.rs",
      "role": "examined",
      "excerpt": "Supersession: absent union row now UnavailableIdentityWitness at selected_union; enum-only mask still refused."
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "high",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::published_reader_refusals_keep_their_codes",
      "related_ids": [
        "FR-352-AC-31",
        "FR-830-AC-7",
        "FR-830-AC-8",
        "FR-830-AC-9",
        "FR-830-AC-13",
        "FR-830-AC-14"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "393-401 (e8ba885f7); crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:497,672,860-962 (e8ba885f7)",
      "excerpt": "        let expected = row[\"expected\"][\"code\"].as_str().expect(\"code\");\n        if expected == \"malformed\" {\n            // The closed-format codec is the FSL identity-file parser's.\n            continue;\n        }\n        let actual = run(&vectors, id, None);\n        if actual != expected {\n            mismatches.push(format!(\"{id}: expected {expected}, got {actual}\"));\n        }",
      "finding": "Refusal loci do not match the published vectors and no test checks them. The refusal test compares only expected.code and ignores expected.locus, while FR-830 AC-7/8/9/13/14 and FR-352-AC-31 require code and locus. Wrong pointers as found: model_union.rs:672 emits /member (MUW-R-17/18 expect /member/declaration and /member/identifier); 860/876/896/918/940/962 emit a bare /payload/node with no /model_unions/{k}/cases/{j} prefix (R-43/46/51/54/55, TC-02/03/05); 497 (R-01) and every tuple refusal from with_admitted_type_rows (TR-01..07) carry form selected_identity, not a /model_unions or /model_tuples/{k} pointer. A mutant that swaps /rank and /position passes."
    },
    {
      "fnd": "FND-002",
      "severity": "high",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::the_measured_baseline_admits_and_one_unit_less_exhausts_each_counter",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-11"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "426-462 (e8ba885f7)",
      "excerpt": "        assert_eq!(\n            run(&vectors, id, Some((work, storage))),\n            \"admitted\",\n            \"{id}: baseline\"\n        );\n        assert_eq!(\n            run(&vectors, id, Some((work - 1, storage))),\n            \"exhausted_work\",\n            \"{id}: one unit less work\"\n        );\n        assert_eq!(",
      "finding": "The published budget_cases MUW-B-01..B-08 are never executed. The budget test asserts only outcome codes for baseline, one-below and zero limits on three positives; it never asserts the first unpaid kind, full attempted amount or locus, and omits B-02 (closure-edge boundary Ec-1/Ec), B-03 (cumulative across joins), B-04/B-06/B-07 (charges kept after a refusal, no rows), B-05 (tuple/union channel split at Wt) and B-08 (reached with an unreached row: surplus only common work). FR-830-AC-11 and FR-358-AC-15 are largely unverified."
    },
    {
      "fnd": "FND-003",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::published_positive_vectors_are_admitted_with_their_rows_returned",
      "related_ids": [
        "FR-830-AC-2",
        "FR-830-AC-3",
        "FR-830-AC-10",
        "FR-352-AC-31"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "330-336 (e8ba885f7)",
      "excerpt": "        match identities.with_model_union_rows(&admitted, &tuples, &unions, scope, &mut budget) {\n            Ok(joined) => {\n                // Every joined union row is returned unchanged.\n                for row in joined.model_union_rows() {\n                    assert!(\n                        unions.contains(row),\n                        \"{id}: a returned row differs from its input\"\n                    );\n                }\n                \"admitted\".into()",
      "finding": "The positive test only checks each returned row is contained in the input. A change that retains out-of-closure rows under reached scope (MUW-03, MUW-16) or returns extra rows (MUW-19 returns only the model row) passes; dropping all retention passes for every reached vector. The published closure_edges counts (AC-3: each node expanded once) are never asserted. Only the MUW-01 full-scope projection test asserts an exact count."
    },
    {
      "fnd": "FND-004",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs::with_admitted_type_rows",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-11",
        "FR-352-AC-31"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "lines": "85; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:456,487-890",
      "excerpt": "            for (index, row) in tuple_rows.iter().enumerate() {\n                budget.set_locus_source(NativeJoinLocusSource::TupleRow(index));\n                validate_admitted_tuple(package, row, budget)?;",
      "finding": "Native denials carry no sidecar row-index locus. The union phase never sets a locus source; in the reached monitor path with_admitted_type_rows sets TupleRow(index) over the scope-filtered joined_tuples, not the original sidecar index, and that source stays active, so a later union-phase denial is attributed to the last tuple row. FR-830 Work and storage requires the denial's compact locus to name the channel (model_tuples vs model_unions), the sidecar row index and the form."
    },
    {
      "fnd": "FND-005",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::model_union_payload_node",
      "related_ids": [
        "FR-352-AC-31",
        "FR-830-AC-9",
        "FR-069-AC-8"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "247-263, 871-877 (e8ba885f7 numbering 247-263, 870-878)",
      "excerpt": "            | MemberType::Reference(id) => return Ok(Some(Cow::Borrowed(id))),\n            MemberType::Boolean => \"boolean\",\n            MemberType::Integer => \"integer\",\n            MemberType::Float32 => \"float32\",\n            MemberType::Float64 => \"float64\",\n            MemberType::String => \"string\",\n            MemberType::Uuid => \"uuid\",\n            MemberType::Timestamp => \"timestamp\",\n            MemberType::Bytes => \"bytes\",\n            MemberType::JsonObject => \"json_object\",\n            _ => return Ok(None),",
      "finding": "Admitted non-Clock payload forms are refused. model_union_payload_node returns None for Option, Collection, IntRange, RationalRange, TextBounds, CarriedRational and ClockReference, and model_union_expected_type refuses an enum-family variant payload; all become UnsupportedExactType. FR-830 says a data payload is any admitted type but Clock and the original value-type grammar includes option, collection, map, bounded and carried-quantity forms. A full join over a package with a Rational-range payload (TK-36, FR-069-AC-8) cannot succeed. No vector covers these forms; that does not narrow the contract."
    },
    {
      "fnd": "FND-006",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "coverage",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs::with_model_union_rows_over call",
      "related_ids": [
        "FR-352-AC-31",
        "FR-019-AC-27"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs",
      "lines": "708-719; crates/filament-contract-model/src/checked_package/v2/runtime_type_projection.rs:1211-1220; crates/filament-contract-model/src/checked_package/v2/exact_projection.rs:3429-3437",
      "excerpt": "            let identities = identities\n                .with_model_union_rows_over(\n                    self.package,\n                    rows.tuples,\n                    rows.model_unions,\n                    &reached,\n                    budget,\n                )",
      "finding": "The monitor and runtime threading has no test with real rows. Every ReachedProducerRows and SelectedMonitorClosureRows site passes model_unions: &[] (reached_model_read.rs tests, monitor_plan_public.rs:70, receipt.rs:284), so with_model_union_rows_over and SelectedTypeDeclarations::unions() never run with a row. FR-352-AC-31 also claims a record or object field of an admitted union returns Composite(type_handle) (exact_projection.rs:3429); no test exercises it, nor a tuple position or optional field of union type."
    },
    {
      "fnd": "FND-007",
      "severity": "medium",
      "confidence": "medium",
      "method": "code-review",
      "check": "soundness",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::model_union_handles",
      "related_ids": [
        "FR-830-AC-2",
        "FR-830-AC-7",
        "FR-830-AC-14"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "281-345",
      "excerpt": "    fn model_union_handles(\n        &self,\n        first: &CheckedNodeId,\n        budget: &mut NativeJoinBudget,\n    ) -> Result<BTreeSet<NodeKey>, ExactTypeProjectionError> {\n        charge_native_storage(budget, first, first, \"model_union_handles\", 2)?;\n        let mut handles = BTreeSet::new();\n        macro_rules! take {\n            ($id:expr, $key:expr) => {{\n                charge_native_work(budget, first, $id, \"model_union_handle\", 1)?;\n                if !handles.contains($key) {\n                    charge_native_storage(budget, first, $id, \"model_union_handle\", 2)?;",
      "finding": "The preflight handle index covers only rows already retained by the receiver, not the supplied rows. FR-830 Work and storage puts the handles of every supplied tuple and union witness, across both channels, in the preflight index of the common pass. Under reached scope a joined row whose type_handle or settled_key collides with an unreached supplied row's handle is not detected, and the B-08 claim that the surplus is preflight-index work is not what the code does."
    },
    {
      "fnd": "FND-008",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "other",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::uses_source_rows",
      "related_ids": [],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "356-357 (e8ba885f7)",
      "excerpt": "/// Fixture C holds source rows this adapter does not build; the source-row\n/// non-substitution controls are asserted by the native tests of the crate.\nfn uses_source_rows(vectors: &Value, id: &str) -> bool {\n    find(vectors, id).to_string().contains(\"/source_\") || resolve_fixtures(vectors, id) == \"C\"\n}",
      "finding": "Misleading comment: it says the fixture-C source-row non-substitution controls are asserted by the native tests of the crate, but no such native tests exist in the diff or the crate."
    },
    {
      "fnd": "FND-009",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::join_model_unions",
      "related_ids": [
        "FR-358-AC-15"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "405-411, 457-469",
      "excerpt": "        for row in &joined_tuples {\n            // A model tuple's key must not collide with a union handle.\n            if selected\n                .model_unions\n                .values()\n                .any(|union| union.type_handle == row.node_key || union.settled_key == row.node_key)\n            {\n                return Err(witness_error(\n                    &row.structural_node,\n                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,\n                ));\n            }\n        }",
      "finding": "One clone is charged twice and one check is duplicated. selected_identity_handles charges storage::selected_clone() at 405-411 while with_admitted_type_rows charges it again for the only clone (complete_witness.rs:73-80). The tuple-vs-union key check at 457-469 duplicates the model_unions comparison check_handle already makes (complete_witness.rs:302-305) and uses the unbudgeted witness_error with form selected_identity."
    },
    {
      "fnd": "FND-010",
      "severity": "low",
      "confidence": "medium",
      "method": "code-review",
      "check": "soundness",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs::validate_completeness",
      "related_ids": [
        "FR-352-AC-28",
        "FR-830-AC-10"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs",
      "lines": "652-740; 409",
      "excerpt": "    fn validate_completeness(\n        &self,\n        package: &CheckedPackageV2,\n        budget: &mut NativeJoinBudget,\n    ) -> Result<(), ExactTypeProjectionError> {\n        // Required rows are derived below from admitted tables, never from the\n        // number of rows retained by this witness.\n        self.complete_selected_documents(package, budget)?;\n        // Declarations, including model-owned units, share one ascending",
      "finding": "validate_and_rebind ignores model unions. validate_contents does not re-check model_unions rows and validate_completeness requires no row for a union-family variant_type node, while it does require admitted_tuples rows. FR-352-AC-28 says missing emitted declarations or unsupported identity channels refuse rather than yield a partial complete witness, and FR-830 full scope requires one row per union-family node; a rebound witness can be complete without the rows direct projection needs. Matching the older model-enum omission is not contract scope."
    },
    {
      "fnd": "FND-011",
      "severity": "low",
      "confidence": "medium",
      "method": "code-review",
      "check": "soundness",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs::selected_declaration_error",
      "related_ids": [
        "FR-352-AC-31",
        "FR-352-AC-30"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection.rs",
      "lines": "2976-2990, 2999",
      "excerpt": "\nfn selected_declaration_error(\n    package: &CheckedPackageV2,\n    node: &CheckedNodeId,\n) -> ExactTypeProjectionError {\n    let code = if package.node(node).is_some() {\n        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration\n    } else if node.package == package.wire.name\n        || package.models.selected_model(&node.package).is_some()\n    {\n        ExactTypeProjectionErrorCode::UnknownNode\n    } else {\n        ExactTypeProjectionErrorCode::ForeignNode\n    };\n    witness_error(node, code)",
      "finding": "A shared helper changed behaviour for every caller. selected_declaration_error and its budgeted twin now return UnknownNode instead of ForeignNode for any node whose package equals a selected model document identity. This affects all joins and projections that use the helper, not just the model-union join, and only the FR-830 vectors exercise it."
    },
    {
      "fnd": "FND-012",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "other",
      "artifact_id": "FR-352-AC-31",
      "related_ids": [
        "FR-358-AC-15",
        "FR-069-AC-11"
      ],
      "path": "spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md",
      "lines": "627 (e8ba885f7)",
      "excerpt": "Joining the selected model tuple and model union witness rows (FSpec FR-830) returns the complete rows keyed by structural node, retaining each type handle, settled key, case rank, position, name, original case, member and payload as the producer's opaque identities. Tuple rows are admitted first, then union identity, case census, per-case checks, payload original type against the side-input witness table, and coverage: a full scope refuses an uncovered union with `IncompleteIdentityWitness` and a reached scope with `UnavailableIdentityWitness`. Direct projection and a record or object field o",
      "finding": "Spec-row wording. FR-352-AC-31 says a foreign selection refuses UnknownNode, which is ambiguous beside FR-352-AC-30 (identities from an independent admission refuse ForeignNode). FR-358-AC-15 says every row is charged in the common pass before allocation, but FR-830 and the code allocate the preflight handle index first. None of the three new rows carried the status prefix sibling rows use."
    },
    {
      "fnd": "FND-013",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::model_union_closure",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-3"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "150-157, 165-166, 221-233",
      "excerpt": "        if package.node(start).is_none() {\n            return Err(selected_declaration_error(package, start));\n        }\n        charge_native_storage(budget, start, start, \"model_union_closure\", 3)?;\n        let mut visited = BTreeSet::new();\n        let mut pending = vec![start.clone()];\n        visited.insert(start.clone());\n        while let Some(id) = pending.pop() {",
      "finding": "The reached closure clones before it pays. The start node is cloned twice (pending and visited) under a flat charge of 3 storage units, not its node size; each new target is cloned twice (visited.insert and pending.push) under one node_units+2 charge; the per-node edges and members Vecs grow with no storage charge. A long start or target CheckedNodeId under a tight storage limit allocates beyond what the budget admitted, violating precharge-before-clone."
    },
    {
      "fnd": "FND-014",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs::model_union",
      "related_ids": [
        "FR-358-AC-15"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/native_join_storage.rs",
      "lines": "605-612; crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs:470,477,774-777",
      "excerpt": "            }\n            // Retention: precharge every copy before it is made.\n            charge_native_storage(budget, id, id, \"model_union_row\", row_units(row))?;\n            handles.insert(row.type_handle.clone());\n            handles.insert(row.settled_key.clone());\n            staged.insert(id.clone(), Arc::new(row.clone()));",
      "finding": "Staged and index storage is under-charged. staged.insert clones the structural node as the map key, but row_units (native_join_storage::model_union) counts the structural node once, for the row copy; the tuple path charges the key separately (complete_witness.rs:121-124, storage::node(id)). The handle-set inserts at 470 and 775-776 and the joined_handles BTreeMap at 472-478 allocate with no storage charge."
    },
    {
      "fnd": "FND-015",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::join_model_unions coverage",
      "related_ids": [
        "FR-358-AC-15",
        "FR-830-AC-11"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "457-469, 780-787",
      "excerpt": "        for (id, entry) in package.models.selected_declarations() {\n            let needs_tuple = entry.form == DeclarationForm::TupleValueType;\n            let needs_union = entry.form == DeclarationForm::VariantType\n                && package.models.variant_family(id) == Some(\"union\");\n            if !(needs_tuple || needs_union) {\n                continue;\n            }\n            charge_native_work(budget, first, id, \"model_union_coverage\", 1)?;",
      "finding": "Uncharged scans. The coverage loop walks every selected declaration and calls variant_family (which scans the variant's cases) before any work charge, charging only matching tuple and union nodes; the tuple-vs-prior-union check at 457-469 scans every retained union for every joined tuple with no charge (T x U). These are whole-model and quadratic blind spots in the shared work budget."
    },
    {
      "fnd": "FND-016",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "coverage",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::uses_source_rows",
      "related_ids": [
        "FR-830-AC-10",
        "FR-830-AC-15",
        "FR-352-AC-31"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "377-381",
      "excerpt": "/// Fixture C holds source rows this adapter does not build; the source-row\n/// non-substitution controls are not executed here and remain an open boundary.\nfn uses_source_rows(vectors: &Value, id: &str) -> bool {\n    find(vectors, id).to_string().contains(\"/source_\") || resolve_fixtures(vectors, id) == \"C\"\n}",
      "finding": "The typed source non-substitution controls are skipped as if producer-only. MUW-19, MUW-20 and MUW-R-59..R-62 need only typed SelectedSourceUnionRow and SelectedSourceTupleRow rows and the existing native source joins, which IR already owns; the FSL parser boundary covers the closed format only. Nothing shows a valid source_unions or source_tuples row never supplies a missing model row, or that MUW-20 admits beside a source tuple."
    },
    {
      "fnd": "FND-017",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "test-intent",
      "artifact_id": "tests/conformance_fspec/model_union_witness.rs::published_reader_refusals_keep_their_codes",
      "related_ids": [
        "FR-830-AC-14",
        "FR-830-AC-7",
        "FR-352-AC-31"
      ],
      "path": "tests/conformance_fspec/model_union_witness.rs",
      "lines": "420-428",
      "excerpt": "        let want = row[\"expected\"][\"locus\"].as_str().unwrap_or(\"\");\n        // A tuple-channel refusal keeps the existing tuple stage form; the\n        // model-tuple row pointers are reported as a boundary, not asserted.\n        let tuple_stage = id.contains(\"-TR-\");\n        if actual != expected || (!tuple_stage && locus.as_deref().unwrap_or(\"\") != want) {\n            mismatches.push(format!(\n                \"{id}: expected {expected} at {want}, got {actual} at {locus:?}\"\n            ));\n        }",
      "finding": "The -TR- locus exemption drops required coverage. IR owns the typed model_tuples join and knows each row's sidecar index before filtering, so /model_tuples/{k}/... is IR's coordinate to report, not an FSL adapter pointer. The substring match also exempts MUW-TR-08, whose expected locus is the union pointer /model_unions/1/type_handle. Coverage vectors' expected.node is never compared with error.at."
    },
    {
      "fnd": "FND-018",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "other",
      "artifact_id": "spec/model/functional/FR-019-rust-library-interface.md",
      "related_ids": [
        "FR-019-AC-27"
      ],
      "path": "spec/model/functional/FR-019-rust-library-interface.md",
      "lines": "311, 316, 318",
      "excerpt": "| `checked_package` (V2 selected monitor type closure) | types `SelectedMonitorClosureRows`, `SelectedMonitorTypeClosure`, `SelectedMonitorTypeClosureError`, `SelectedMonitorTypeClosureErrorCode`; associated bounded project_selected_type_closure read over one admitted root, selected producer rows and shared native budget | FR-357-AC-1 through FR-357-AC-4 |",
      "finding": "FR-019 inventory omits part of the public change: the new public field SelectedMonitorClosureRows.model_unions is a breaking change for struct-literal consumers (receipt.rs and monitor_plan_public.rs had to change) but is not in the breaking-change prose, and the associated with_model_union_rows join, model_union_rows/model_union_row accessors and SelectedTypeDeclarations::unions() are not named."
    }
  ],
  "bindings": []
}
```

## Disposition round 2 data

```yaml
{
  "round": 2,
  "head": "16a686c7641e9b07841c130d28ad26ebf1a198a1",
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "after_excerpt": "        let want = row[\"expected\"][\"locus\"].as_str().expect(\"locus\");\n        let (code, locus, at) = match prepare(&vectors, id) {\n            Err((code, locus)) => (code, locus.unwrap_or_default(), None),\n            Ok(prepared) => match join(&prepared, &mut ample()).0 {\n                Ok(_) => (\"admitted\".to_owned(), String::new(), None),\n                Err(error) => (\n                    error.code.as_str().to_owned(),\n                    error.form.as_deref().unwrap_or_default().to_owned(),\n                    error.at.map(|node| *node),\n                ),\n            },\n        };"
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "    // Ec-1: the denied charge is the last closure edge, before any row joins.\n    let denied = probe(&prepared, ec - 1, u64::MAX);\n    assert_denied_at(&denied, &base.events, ec_at, \"Ec-1\");\n    assert!(\n        denied.events.iter().all(|event| !matches!(\n            form_of(event),\n            \"model_tuple_row\" | \"model_union_row\" | \"model_union_retain\"\n        )),\n        \"Ec-1: no row has been joined\"\n    );"
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "after_excerpt": "        assert_eq!(\n            sorted(retained_unions(&joined)),\n            sorted(want_unions),\n            \"{id}: union set\"\n        );\n        assert_eq!(\n            sorted(retained_tuples(&joined)),\n            sorted(want_tuples),\n            \"{id}: tuple set\"\n        );\n        assert!(\n            !retained_unions(&joined).is_empty(),\n            \"{id}: a positive retains union rows, never nothing\"\n        );"
    },
    {
      "fnd": "FND-004",
      "outcome": "fixed",
      "after_excerpt": "        for (k, row) in tuple_rows.iter().enumerate() {\n            at_locus(\n                budget,\n                NativeJoinLocusSource::ModelTupleRow(k),\n                \"model_tuple_handle\",\n            );\n            let owner = Owner::Row(Channel::Tuple, k);\n            let node = &row.structural_node;\n            index.add(\n                first,\n                node,\n                &row.node_key,\n                owner,\n                \"model_tuple_handle\",\n                budget,\n            )?;\n        }"
    },
    {
      "fnd": "FND-005",
      "outcome": "still-open",
      "reason": "758a2cc4d/1be399aa5/dc266979e implement node derivation and Exact types for Option, Collection, IntRange, RationalRange, TextBounds and CarriedRational, and ClockReference refusing unsupported_exact_type is contract-backed (filament-exact b26a8e4 ValueType has no clock-reference variant; FR-830 step 6: no faithful Exact representation is unsupported_exact_type). Still missing: no strict-admitted end-to-end join for any non-scalar admitted payload. any_admitted_non_clock_payload_is_joined_and_each_wrong_witness_is_refused covers seven scalars only; Option, IntRange, RationalRange, TextBounds, CarriedRational, Duration (Quantity of the duration unit) and enum-family payloads are reached only by unit calls on SelectedExactIdentities::empty(), and CarriedRational and enum payloads, which need unit or enum-shape witnesses, have no expected-type assertion. IR already strict-admits a Rational-range variant payload (TK-36, tests/conformance_fspec/model_tuple_union.rs:360), and the harness's document_patch can add such cases, so the read-only FSpec worktree is no blocker."
    },
    {
      "fnd": "FND-006",
      "outcome": "still-open",
      "reason": "Fixed in part: runtime SelectedTypeDeclarations::unions() over real rows (runtime_selected_types_declare_each_joined_union_by_its_type_handle) and the reached monitor read with a real union row, direct and optional record fields, exact and one-below budget and the no-row refusal (reached_model_read.rs:4377). Still missing: no test projects an object-type field or a tuple position whose type is a model union. The object-field case was removed from the monitor fixture, and FR-352-AC-35 was narrowed to record fields instead of the obligation being evidenced."
    },
    {
      "fnd": "FND-007",
      "outcome": "fixed",
      "after_excerpt": "        for (k, row) in union_rows.iter().enumerate() {\n            at_locus(\n                budget,\n                NativeJoinLocusSource::ModelUnionRow(k),\n                \"model_union_handle\",\n            );\n            let owner = Owner::Row(Channel::Union, k);\n            let node = &row.structural_node;\n            index.add(first, node, &row.type_handle, owner, form, budget)?;\n            if row.settled_key != row.type_handle {\n                index.add(first, node, &row.settled_key, owner, form, budget)?;\n            }\n        }"
    },
    {
      "fnd": "FND-009",
      "outcome": "fixed",
      "after_excerpt": "        // The one clone of the identities this join returns, paid once.\n        charge_native_storage(\n            budget,\n            first,\n            first,\n            \"selected_identity_handles\",\n            storage::selected_clone(),\n        )?;"
    },
    {
      "fnd": "FND-010",
      "outcome": "still-open",
      "reason": "Code changed (complete_witness.rs:626-640 re-validates every retained union row through validate_union_row; :755-764 requires a row per union-family node), but nothing evidences it. The only test, rebinding_a_reached_only_witness_is_refused_and_never_mutates_the_receiver, rebinds the MUW-03 witness, which holds only the Pair tuple row of fixture A's four (Cell, Nest, Pair, Swap). validate_completeness requires tuple rows at complete_witness.rs:751 before it reaches the union requirement at :762, so that refusal is selected_model_tuple and the union-completeness stage is never reached. There is no positive full rebind, no isolated missing-union rebind and no content re-validation refusal. FR-352-AC-37 was narrowed to the refusal half."
    },
    {
      "fnd": "FND-011",
      "outcome": "fixed",
      "after_excerpt": "fn absent_node(\n    package: &CheckedPackageV2,\n    budget: &mut NativeJoinBudget,\n    node: &CheckedNodeId,\n    pointer: &str,\n) -> ExactTypeProjectionError {\n    let code = if node.package == package.wire.name\n        || package.models.selected_model(&node.package).is_some()\n    {\n        ExactTypeProjectionErrorCode::UnknownNode\n    } else {\n        ExactTypeProjectionErrorCode::ForeignNode\n    };\n    fail(budget, code, node, pointer)\n}"
    },
    {
      "fnd": "FND-012",
      "outcome": "fixed",
      "after_excerpt": "The model tuple and model union row join charges one shared `NativeJoinBudget` in the order: handle index over every supplied and retained handle of both channels, the common pass over every row, the reached closure edges, then the row join with retention for in-scope rows. A denied charge consumes nothing."
    },
    {
      "fnd": "FND-013",
      "outcome": "fixed",
      "after_excerpt": "        let units = Self::model_union_scan_units(package, target);\n        charge_native_work(budget, start, target, \"model_union_closure_node\", units)?;\n        charge_native_storage(\n            budget,\n            start,\n            target,\n            \"model_union_closure\",\n            units.saturating_add(3),\n        )?;\n        visited.insert(target);\n        pending.push(target);\n        Ok(())"
    },
    {
      "fnd": "FND-014",
      "outcome": "fixed",
      "after_excerpt": "            charge_native_storage(\n                budget,\n                id,\n                id,\n                \"model_union_row\",\n                row_units(row)\n                    .saturating_add(storage::node(id))\n                    .saturating_add(2),\n            )?;\n            Arc::make_mut(&mut self.model_unions).insert(id.clone(), Arc::new(row.clone()));"
    },
    {
      "fnd": "FND-015",
      "outcome": "fixed",
      "after_excerpt": "                    let cases = package\n                        .models\n                        .selected_models()\n                        .get(entry.selection)\n                        .and_then(|model| model.variant_types.get(&entry.node))\n                        .map_or(0, |cases| cases.len());\n                    charge_native_work(\n                        budget,\n                        first,\n                        id,\n                        \"model_union_coverage_family\",\n                        u128::try_from(cases).unwrap_or(u128::MAX),\n                    )?;"
    },
    {
      "fnd": "FND-016",
      "outcome": "fixed",
      "after_excerpt": "fn source_rows_stay_beside_model_rows_and_are_never_read_as_model_rows() {\n    let vectors = vectors();\n    let prepared = prepare(&vectors, \"MUW-19-source-union-row-beside-model-rows\").unwrap();\n    let (result, _) = join(&prepared, &mut ample());\n    let joined = unwrap_join(\"MUW-19\", result);\n    assert_eq!(\n        joined.source_union_rows().count(),\n        1,\n        \"the source union row stays\"\n    );\n    assert_eq!(\n        retained_unions(&joined),\n        vec![prepared.unions[0].structural_node.clone()],\n        \"only the model row of Link is retained\"\n    );"
    },
    {
      "fnd": "FND-017",
      "outcome": "fixed",
      "after_excerpt": "        let want_node = row[\"expected\"]\n            .get(\"node\")\n            .filter(|node| !node.is_null())\n            .map(node_id);\n        if code != expected || locus != want || (want_node.is_some() && at != want_node) {\n            mismatches.push(format!(\n                \"{id}: expected {expected} at {want} node {want_node:?}, got {code} at {locus} node {at:?}\"\n            ));\n        }"
    },
    {
      "fnd": "FND-018",
      "outcome": "fixed",
      "after_excerpt": "QSL-2743 adds the model-union row join (with_model_union_rows, model_tuple_rows, model_union_rows, model_union_row, SelectedTypeDeclarations::unions), the NativeJoinLocusSource variants ModelTupleRow and ModelUnionRow, and the required ReachedProducerRows/SelectedMonitorClosureRows field model_unions (breaking for literal constructors; pass an empty slice for a selection without model unions)"
    }
  ],
  "new_findings": [
    {
      "fnd": "FND-019",
      "severity": "high",
      "confidence": "high",
      "method": "code-review",
      "check": "code-bug",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs::model_union_payload_node",
      "related_ids": [
        "FR-352-AC-32",
        "FR-352-AC-35"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "1402-1660, 1667-1831; tests 1998-2038, 2165-2199",
      "excerpt": "        let leaf_id = match leaf {\n            MemberType::Clock => return Ok(None),\n            MemberType::ClockReference => {\n                derived(budget, \"scalar_type\", \"clock_reference\", Vec::new(), 0, 0)?\n            }\n            MemberType::Record(id)\n            | MemberType::Tuple(id)\n            | MemberType::Variant(id)\n            | MemberType::Quantity(id) => {\n                charge_native_work(budget, at, at, \"model_union_payload_node\", node_units(id))?;\n                charge_native_storage(budget, at, at, \"model_union_payload_node\", node_units(id))?;",
      "finding": "Two implementations of one rule (introduced in 758a2cc4d/1be399aa5). model_union_payload_node re-implements MemberType::node_id (the FR-322 step 4 structural-id derivation) form for form, so that it can charge before allocating. payload_nodes_are_the_structural_ids_the_member_types_own is a drift detector over that copy. model_union_expected_type re-implements the leaf and Option/Collection wrapper projection beside project_record_field_type, and the two now disagree: a Float32/Float64, Bytes, JsonObject, RationalRange, TextBounds or CarriedRational payload projects to an Exact type, while the same member type as a record field refuses UnsupportedExactType (exact_projection.rs:3448-3463). A fix to either derivation must be made twice."
    },
    {
      "fnd": "FND-020",
      "severity": "medium",
      "confidence": "high",
      "method": "code-review",
      "check": "soundness",
      "artifact_id": "FR-358-AC-17",
      "related_ids": [
        "FR-830-AC-11",
        "FR-358-AC-19"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/exact_projection/model_union.rs",
      "lines": "333-337, 366-370, 391-395, 321",
      "excerpt": "The reached closure charges one edge per typed member of every reached union (scalars included, `Clock` none); a budget one unit below the measured edges refuses at the `ModelUnionRow` locus of the row being closed.",
      "finding": "Closure-edge charges never carry the row locus FR-358-AC-17 states. model_union_closure sets the active locus to NativeJoinLocusSource::Package with form model_union_closure or model_union_closure_type, and model_union_follow charges each edge under that locus, so an Ec-1 denial names Package, not the ModelUnionRow being closed or the edge (FR-830 B-02: its locus names that edge). budget_case_b02 only compares the denial with the baseline event's own locus, so it cannot fail on this."
    },
    {
      "fnd": "FND-021",
      "severity": "low",
      "confidence": "high",
      "method": "code-review",
      "check": "other",
      "artifact_id": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs::reached_read_joins_model_union_rows_and_projects_every_field_form",
      "related_ids": [
        "FR-352-AC-35"
      ],
      "path": "crates/filament-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs",
      "lines": "4374-4377",
      "excerpt": "    /// as the composite its own `type_handle` names.\n    #[trace(\"FR-352-AC-35\", \"FR-358-AC-17\")]\n    #[test]\n    fn reached_read_joins_model_union_rows_and_projects_every_field_form() {",
      "finding": "The test name and doc overclaim: 'every field form' covers only a direct and an optional record field. There is no object-type field, tuple position or collection field, so the name hides the gap recorded in FND-006."
    }
  ]
}
```

## Disposition round 3 data

```yaml
{
  "round": 3,
  "head": "70b24456c7132d4603f0ddcaa5f3ba87aba3befc",
  "dispositions": [
    {
      "fnd": "FND-005",
      "outcome": "still-open",
      "reason": "Fixed in part (34969ef2d, 8f8bd1c9e): integer range, profiled text, rational range and option payloads now join end to end through strict-admitted packages (checked_package_v2_model_fields.rs every_strict_admitted_non_scalar_payload_form_joins_and_each_wrong_witness_is_refused). Unit quantity, object Reference and record/tuple/union composites join with real witnesses (MUW-11), and enum-family payloads join with a real enum shape. A sequence payload is refused at intake, and ClockReference stays contract-backed unsupported. Still missing: a carried-quantity payload (value type Rational with a unit, admitted at intake through value_types) and a Duration payload (MemberType::Quantity of the duration unit, admitted at intake) have no end-to-end join. No test anywhere asserts a positive ValueType::CarriedQuantity projection, on the payload or the record-field path; payload_and_record_field_project_every_admitted_type_identically compares CarriedRational only as the same refusal on empty identities."
    },
    {
      "fnd": "FND-006",
      "outcome": "fixed",
      "after_excerpt": "    let held = object\n        .fields()\n        .iter()\n        .find(|field| field.name() == \"held\")\n        .expect(\"the object's union field\");\n    assert_eq!(held.original_identity(), format!(\"{}/held\", orders(\"Node\")));\n    assert_eq!(held.value_type(), &handle);\n    assert_eq!(held.presence(), Presence::Required);\n    // The tuple position, by its author order.\n    let pair = view\n        .tuples()\n        .iter()\n        .find(|tuple| tuple.original_identity() == orders(\"Pair\"))\n        .expect(\"the selected tuple\");\n    assert_eq!(pair.positions(), [handle.clone()]);"
    },
    {
      "fnd": "FND-010",
      "outcome": "fixed",
      "after_excerpt": "        .validate_and_rebind(&package, &mut budget)\n        .expect_err(\"a witness without the union row is incomplete\");\n    let choice: CheckedNodeId =\n        serde_json::from_value(node_id(&declaration_key(\"Choice\", \"variant_type\")))\n            .expect(\"the union node\");\n    assert_eq!(\n        refusal.code,\n        ExactTypeProjectionErrorCode::IncompleteIdentityWitness\n    );\n    assert_eq!(refusal.at.as_deref(), Some(&choice));\n    assert_eq!(refusal.requested.as_deref(), Some(&choice));\n    assert_eq!(refusal.form.as_deref(), Some(\"selected_model_union\"));"
    },
    {
      "fnd": "FND-019",
      "outcome": "fixed",
      "after_excerpt": "    fn model_union_expected_type(\n        &self,\n        package: &CheckedPackageV2,\n        member: &MemberType,\n        node: &CheckedNodeId,\n        locus: &str,\n        budget: &mut NativeJoinBudget,\n    ) -> Res<ValueType> {\n        super::project_wrapped_record_field_type(\n            package,\n            self,\n            node,\n            node,\n            member,\n            budget,\n            u64::MAX,\n        )"
    },
    {
      "fnd": "FND-020",
      "outcome": "fixed",
      "after_excerpt": "/// Locate the following closure charges at the admitted graph node being\n/// closed: a reached closure walks package nodes, not sidecar rows, so the node\n/// is the actual coordinate of the edge examined.\nfn at_closure_node(\n    package: &CheckedPackageV2,\n    budget: &mut NativeJoinBudget,\n    node: &CheckedNodeId,\n    form: &'static str,\n) {\n    let source = package.node_position(node).map_or(\n        NativeJoinLocusSource::Package,\n        NativeJoinLocusSource::GraphNode,\n    );\n    at_locus(budget, source, form);\n}"
    },
    {
      "fnd": "FND-021",
      "outcome": "fixed",
      "after_excerpt": "    #[trace(\"FR-352-AC-35\", \"FR-358-AC-16\")]\n    #[test]\n    fn reached_read_joins_model_union_rows_and_projects_direct_and_optional_record_fields() {"
    }
  ],
  "new_findings": [],
  "nonregression": "The previously fixed FND-001..004, 007, 009, 011..018 were rechecked at 70b24456c against the changed paths: shared MemberType::node_id_in through PayloadNodeAlloc, charged before allocation; the payload projection delegates to project_wrapped_record_field_type; the closure locus is GraphNode; the handle preflight and common pass are unchanged; the row loci are unchanged; the union stage still retains into the local clone and publishes atomically; complete_selected_documents now skips only union-family variants. No regression found. Receipts: final_conf_muw 17 passed, final_lib_model_union 4, final_reached2 32, final_it_fields 90, final_it_cw 16, final_it_limits 7, final_it_map 17, final_conf_carried 19. Reviewer reruns at 70b24456c: lib exact_projection:: 20 passed rc=0, lib selected_facts 1 passed rc=0."
}
```
