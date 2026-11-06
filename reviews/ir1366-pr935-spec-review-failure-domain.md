---
id: SR-2026110602354632
title: "IR-1366 PR 935 spec-review-failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; IR-1366; runtime_type_projection, nominal_reads, charged model_members, reached Exact witness helpers, FR-360, TC-470"
review_set: subset
---

## Summary

Ticket: IR-1366. The spec defines incomplete/refused results, first-denial preservation, cumulative budget state, no partial values and clone determinism. Finite work bounds cycles/depth; borrowed package data and producer witnesses preserve identity boundaries. There are no callbacks or side-effectful evaluation hooks in the changed scope. The missing runtime locus is an implementation violation already recorded by code-review, not an absent spec constraint.

## Scope

- role: examined; id: FR-360-AC-1; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: Each public runtime type projection, monitor-closure, runtime-reachable nominal-read, and selected unit-definition entry point requires the caller's mutable `NativeJoinBudget`; budget-aware helper variants charge allocations to that same budget. They do not route storage through `model_members::Budget` / `WorkMeter`, create a local allowance, or offer an unbudgeted runtime entry point or compatibility wrapper.
- role: examined; id: FR-360-AC-2; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: Every inspected runtime declaration, closure node or edge, nominal row, identity component, and selected unit edge charges work before inspection. Every IR-owned temporary or retained allocation, clone, copied name/type, formatted unit string, and runtime-reachable `element_type` / `slot_type` result charges FR-358 logical storage to the caller's budget before allocation or copying. Exact-limit work and storage succeed; one unit below either measured requirement refuses before the first unpaid operation.
- role: examined; id: FR-360-AC-3; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: A storage-limited refusal occurs before the denied allocation and returns a bounded typed budget refusal with the first unpaid resource and amount, no partial runtime view, and no uncharged owned diagnostic payload. A work-limited refusal reports the first unpaid work locus without advancing the denied charge. Any earlier admitted charges remain visible and the refused operation has no success delta.
- role: examined; id: FR-360-AC-4; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: At sufficient limits, representative source and model runtime type reads, selected nominal reads, and selected unit-definition reads return the same complete values, original identities, order, and semantic refusal causes as the existing behavior. Equal input, rows, and limits produce equal values, cumulative charges, deltas, or first refusals for the original and a clone of the admitted package.
- role: examined; id: FR-360-AC-5; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: Runtime projection and monitor-closure paths use budget-aware variants for `DomainModel::element_type` / `slot_type` and other helpers, charging every reachable `BTreeSet` / `Vec`, cloned name/member type, formatted unit string, and other allocation to the caller's `NativeJoinBudget` before allocation or copy. This ticket does not add storage accounting to `WorkMeter` / `model_members::Budget` or widen `model_fields`, `model_operations`, semantic-operation, state, `step_relation`, or `comparison_operands` APIs; these serve separate paths and are not called by the audited runtime/closure path.

Changed production files examined: runtime_type_projection.rs; nominal_reads.rs; model_members.rs; exact_projection.rs and exact_projection/complete_witness.rs. Changed test files examined: model_members/tests.rs; checked_package_v2_model_fields.rs and operation_census.rs; nominal_reads.rs. TC-470 procedure and expected results examined. FR-358/FR-359/FR-354 and repository README/CLAUDE/AGENTS are context only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. The spec defines incomplete/refused results, first-denial preservation, cumulative budget state, no partial values and clone determinism. Finite work bounds cycles/depth; borrowed package data and producer witnesses preserve identity boundaries. There are no callbacks or side-effectful evaluation hooks in the changed scope. The missing runtime locus is an implementation violation already recorded by code-review, not an absent spec constraint.

## Coverage

Plan completion: not assessed

No Cargo rebuild or aggregate gate was run by this reviewer. Inspected existing receipts: one helper ownership test, four Exact helper tests, 85 model/runtime tests and 12 nominal tests pass; workspace Clippy, formatter and two-document specification validation pass. Final combined-tree gate remains lead-owned and pending. The static matrix command completed within its 45-second bound; no evidence store was used. Optional gap-analysis semantic fan-out was not requested and was skipped; mandatory code/test alignment was performed in the Rust lane. No whole-repository assurance claim is made.


## Round 1 Evidence Status

Static fix review does not change this method's original scoped conclusion. No global matrix recomputation or Cargo rebuild was performed. New regression receipts remain pending; prior 102 tests apply only to the earlier candidate. The lead final combined-tree gate remains pending. See the code-review round 1 new findings before merge.


## Round 2 Evidence Status

The static scope and earlier method conclusion remain qualified as before. No global matrix recomputation or Cargo checks were run by the reviewer. New regression results and the final combined-tree gate remain pending. Code-review FND-006 requires a same-PR correction before merge.


## Round 3 Evidence Status

All six code findings are now statically fixed with no new finding. New regression checks are running and their actual results remain pending; no runtime PASS is claimed here. Original scoped matrix qualification and pre-existing global gaps remain unchanged. The lead final combined-tree gate remains pending.


## Final Focused Evidence and Current Verdict

PASS for the reviewed specification scope. All six code findings have a latest fixed disposition; no new finding remains. This current evidence update supersedes the earlier round notes saying that regression execution was pending.

Actual current-production regression receipts were read in full: /tmp/ir1366-round3-focused.sh and its head log identify the reviewed production candidate; the set-e script completed workspace checks with conformance-cli enabled. Tests passed: 2 runtime helper tests, 3 native-budget tests, 4 Exact helper tests, 86 model/runtime integration tests, and 13 nominal integration tests (108 total; zero failed/ignored in these selections). These include shared slot multiplicities, fixed first-denial metadata, earlier-row tuple/unit insertion attribution, unresolved declaration coordinates, context restoration, and the multi-term parent-dimension denial regression. Workspace Clippy with warnings denied and the integration-target compile check passed. That Clippy invocation omitted --all-targets; broader lint coverage remains pending with the lead/coder. This is focused execution evidence, not the final combined-tree gate.

All five FR-360 criteria were tagged in the earlier computed matrix. Its repository-wide pre-existing gaps remain qualified and were not recomputed or resolved here; no global repository assurance PASS is claimed. Plan completion: not assessed.

Merge readiness remains conditional on committing the latest versions of all six scratch review artifacts and the lead-owned final combined-tree gate, including the required all-targets coverage. No reviewer build, source edit, push or merge was performed.


## Narrow Rebase Integration Review

PASS for the narrow rebase/integration scope. All six findings remain fixed and no new finding was found. Range-diff shows the three fix commits and retained review artifacts carry unchanged; the only additional PR change adapts runtime-mode restoration assertions to the new upstream charged native-denial behavior. The shared-rule, parent-dimension, direct-coordinate and precharge implementations are unchanged. Upstream native_budget_denial remains the single ordinary budget-denial constructor, charging the complete diagnostic floor before ownership and preserving the original first unpaid charge when its diagnostic storage is unaffordable.

The revised mode-restoration oracle is substantive: it requires a runtime refusal inside admitted_tuple_row, then reuses that budget with abundant storage for an ordinary join and asserts an owned requested locus, additional admitted diagnostic storage, matching denial metadata and no leaked runtime coordinate. A leaked scoped Exact mode returns compact with no requested node and fails this oracle. The observed_restored_native_mode assertion requires that branch to execute. The separate no-storage ordinary-denial case correctly expects compact payloads under the new baseline; it is no longer relied on alone as proof of mode restoration.

Inspected actual rebase-focused script, tested-head receipt, range-diff and execution logs. Passing selections total 169: 2 runtime helpers, 3 native-budget unit tests, 5 Exact helper tests, 86 model/runtime integration tests, 13 nominal tests, 9 complete-witness tests, 33 owner tests and 18 union tests. The native_join_budget integration filter runs zero standalone tests and is not included in that total. Workspace Clippy, feature-matched integration-target compilation and formatter receipts pass. The focused script uses --locked --workspace with conformance-cli. No reviewer build or global matrix rerun was performed. No excluded scratch/old dependency commit is in the candidate ancestry.

Current scoped review verdict: PASS, subject to the lead-owned final combined-tree gate after the latest six artifacts are committed. Earlier pending lint/evidence notes are historical; the current rebase receipts replace them for this candidate. Repository-wide matrix gaps remain qualified and unchanged; this is no global assurance claim.


## Final-Gate Discovery Correction Review

PASS for the narrow final-gate discovery corrections; all six original findings remain fixed and no new unresolved finding was found. The first native full-gate attempt failed with 1280 passing, 10 failing and one ignored integration test; an earlier whitespace preflight failure was separate. Neither failed attempt is a final gate PASS. Three failures exposed the missing declaration of the two actual public metadata types; seven were missing checker/scanner executables. After configuring the real tools, one actual helper dispatch violation remained and was corrected without an exemption.

FR-019 now explicitly inventories NativeJoinLocus and NativeJoinLocusSource under FR-360, and FR-360 documents their existing fixed-coordinate API. The exact public-inventory comparison and fail-closed checks remain intact; only the two legitimate names are added. Duration selection now uses the existing typed own_seconds_declared fact to construct exactly the owning-model or global resolved key. Foreign units sharing the suffix cannot match. Four borrowed parts form a stack array; the common native_join_storage::text_parts function computes checked/saturating logical text size with no heap allocation. Caller work and storage are admitted before concat or map lookup; one charged temporary key is then used. The common text footprint delegates to the same function, avoiding a duplicate formula. Nonruntime getter signatures and shared semantic implementation remain intact.

Actual newly completed receipts were inspected: the duration-green script uses --locked --workspace and conformance-cli with the real QSL_STRING_EDGE_BIN and QUIRE_LOCK_CHECKER. Its tested-head receipt matches the frozen candidate. All 143 selected tests pass: 3 runtime helpers, 3 native-budget tests, 5 Exact helpers, 6 authoritative string-edge tests, 25 public-interface tests, 2 lock-coherence tests, 86 model/runtime tests and 13 nominal tests. The added exact owner/global test checks own presence, absent-own with global available, global selection and absent-global with foreign/own decoys. The zero-storage key test failed on the prior scan implementation and now passes. Existing ownership-boundary tests exhaust every storage prefix. Clippy/fmt receipts pass. Scoped FR-019/FR-360 validation reports both documents grammar-clean. No global matrix rerun, reviewer build or source edit occurred.

Additional examined scope: spec/model/functional/FR-019-rust-library-interface.md closed export inventory; the new FR-360 metadata paragraph; public_interface exact inventory; duration helper, common text_parts footprint and owner/zero-storage tests. role: examined; excerpt: "The runtime refusal metadata types `NativeJoinLocus` and `NativeJoinLocusSource`".

Current scoped verdict: PASS. The latest six scratch artifacts must be copied and committed before the corrected final combined-tree gate. That full gate remains pending; existing repository-wide matrix gaps remain qualified, with no global assurance PASS claimed.


## Integration Round 4 Status

The scoped specification-method conclusion remains PASS; code-review has three unresolved integration findings. New current-head test results remain pending. No whole-repository assurance claim is made.

## Current Integration Scope

Current own runtime test artifact: TC-470 at spec/checked_package/matrix/TC-470-runtime-type-projection-shared-budget.md. The upstream TC-469 reached-model artifact is preserved. Historical evidence and findings retain their original context. Additional examined paths: monitor_plan/selected_closure_walk.rs, monitor_plan/reached_model_read.rs, model_enum_witness.rs, typed_terms::visit_selected_edges, exact_projection::with_model_enum_shapes, shared native storage/budget, FR-360 reached-path Behavior, and FR-357-AC-13/14. The public nonruntime numeric enum and independent model getters remain unchanged. Nested reads use the same native budget, and outer success/refusal restores scoped mode/locus; no fresh native allowance was found.

The new upstream reached-model prerequisite is now reachable from the shared runtime budget path. Audited graph walk, reached reader, private enum stage/read cores, shape attachment and nested projection. The six earlier findings remain fixed, but this integration introduces three unresolved resource/noninterference defects. Source verdict FAIL; current-head regression receipts remain pending. No build, new worktree or global matrix rerun was performed. Earlier passing receipts concern earlier candidates and are not current-head execution evidence. Prior failed full-gate attempts remain historical failures; a successful final combined-tree gate is still required after fixes and artifact custody.


## Round 5 Integration Disposition

The first nine code findings are statically addressed; code-review reports new FND-010 on retained conformance work semantics. Expanded gap assurance remains CONDITIONAL; specification findings remain scoped to their original reviewed units with this additional interaction pending correction.

The genuine emitted-unreached-declaration red receipt was inspected at /tmp/ir1366-round4-reached-red3.log: the prior implementation exhausted the exact reached-only allowance. Earlier red1/red2 fixture failures are not accepted as product evidence. The fixed test retains strict admission, confirms the extra declaration is actually emitted, compares complete types, and requires identical cumulative work/storage. The visitor zero/short-storage test stops before initial/nested frame growth while the legacy callback control still sees all terms. The field streaming test uses a nonempty counted iterator and verifies zero fields inspected, zero storage and the precise parent locus at zero work. None is accepted as a current-head runtime PASS until actual receipts arrive.

Additional examined unit: FR-357-AC-12, spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md, repeated-edge accounting; excerpt: Implemented prerequisite (IR-owned strict QSpec monitor recipe in the explicit conformance-qspec lane; self-contained term bounds run in the default test gate; QSL same-emission handoff remains AC-11). Given one admitted selected monitor root, an internal borrowed read seeds its four selected type handles and four ordered entries, then visits only reached local graph nodes, dependency edges, semantic-type edges and ordinary typed-term reference, type and operation-member structural IDs and flat nominal IDs with iterative worklists and node/edge deduplication; specialized frame, abstraction, ex

Own runtime test artifact remains TC-470; upstream TC-469 remains preserved. Original findings, historical failed gates and global-matrix qualifications remain unchanged. No source edit, reviewer build or global matrix rerun occurred. Latest scratch artifacts require custody before any lead final combined-tree gate.


## Round 5 Partial Execution Update

Current confined-check update supplied during review: all-target conformance-cli Clippy, 11 reached-reader tests, 5 runtime tests, 3 budget tests and 5 Exact tests passed; model/runtime selection passed 85 of 86 and stopped at the expected-form assertion in runtime_two_tuple_indices_activate_the_first_row_after_witness_join. The newly paid ordinary declaration-list stage correctly reports SelectedDeclaration(0)/selected_declaration_index before the old semantic stage. Updating only that later expected form is legitimate; the earlier required post-witness row-zero selected_tuple_index assertion remains independent and must stay. Later filters did not run. This is not a full-gate PASS.


## Round 6 Evidence Status

All ten code findings are statically fixed; the scoped specification conclusions are unchanged, while expanded gap assurance awaits current execution.

The genuine repeated-reference red receipt (/tmp/ir1366-round5-repeat-red2.log) was inspected: prior work was 149317 against expected 148908. The new self-contained test compares equal reached-node values, exactly one extra semantic work unit in both result and caller counters, and exactly two additional stack-frame storage units. It preserves the original QSpec-lane once.work + 1 assertion. Four test-only term mutations now use Arc::make_mut to match the shared admitted backing store. The later ordinary-declaration denial expected form is corrected to selected_declaration_index; the earlier required row-zero post-witness insertion oracle remains intact. Borrowed edge tuples allocate nothing, and first-edge retention/enqueue still requires caller storage admission; visitor stack headers/frames remain precharged. No source edit or reviewer build was performed. Current-head qualified compilation/lint/focused tests are running; no unobserved runtime PASS is claimed. Historical failed full gates and global assurance limitations remain preserved. Own test artifact remains TC-470. Latest six artifacts require custody before the lead final combined-tree gate.


## Final Integration Execution Evidence and Current Verdict

PASS for the reviewed changed scope. All ten findings have latest fixed dispositions; no unresolved finding remains. The final nominal test-only change correctly distinguishes measured.storage - 1 (Package/runtime_model_units, no resolved node) from measured.storage - 2 (the resolved source dimension, selected_dimension). Its required post-witness source-row-zero insertion oracle and complete conversion/identity/clone assertions remain unchanged.

Actual staged receipts were inspected: the production candidate passed qualified QSpec workspace/all-target compilation, conformance-cli all-target Clippy, and focused selections of 12 reached-reader, 6 runtime, 3 native-budget, 5 Exact and 86 model/runtime tests (112 executions). After only the nominal boundary assertion correction, subsequent selections passed 13 nominal, 72 model-member, 25 public-interface, 6 authoritative scanner and 2 lock-coherence tests (118 executions). Total: 230 successful selected test executions, including two overlapping library tests; this is not a count of 230 distinct tests or one combined-head full run. Separate tested-head receipts identify the production stage and later test-only stage. The visitor-stack, streaming-field, emitted-unrelated-declaration and repeated-reference regressions all pass. The earlier nominal failure was accurately recorded before its test correction; earlier full-gate and whitespace-preflight failures remain historical failures.

This current update supersedes earlier integration notes that current runtime execution was pending. Scoped code/gap review PASS; global computed-matrix gaps remain qualified and no whole-repository assurance claim is made. Own runtime test artifact is TC-470; upstream TC-469 is preserved. The latest six scratch artifacts must be committed before the lead-owned final combined-tree gate. Review is ready for that final gate; merge remains contingent on its success. No reviewer source edits, builds or audit expansion occurred.
