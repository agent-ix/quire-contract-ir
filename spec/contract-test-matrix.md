---
id: TM-002
title: "quire-contract-ir v0.1 semantic contract test matrix"
type: TestMatrix
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/11
    type: covers
---
# quire-contract-ir v0.1 semantic contract test matrix

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| StR-001 | FR-011 through FR-015, FR-019, FR-023, FR-025 through FR-034 | TC-015 through TC-018, TC-035, TC-038 through TC-043 | ✅ bounded-Kani and target-neutral output-mapping foundation implemented through TC-043; target-specific output correspondence remains excluded |
| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018 | ✅ implemented |
| StR-003 | FR-012, FR-014, FR-015, FR-017 through FR-020 | TC-015, TC-016, TC-018, TC-020 | ✅ implemented |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-011 | FR-011-AC-1 through FR-011-AC-3 | TC-015 | ✅ implemented |
| FR-012 | FR-012-AC-1 through FR-012-AC-6 | TC-015, TC-016 | ✅ implemented |
| FR-013 | FR-013-AC-1 through FR-013-AC-4 | TC-016 | ✅ implemented |
| FR-014 | FR-014-AC-1 through FR-014-AC-6 | TC-016 | ✅ implemented |
| FR-015 | FR-015-AC-1 through FR-015-AC-7 | TC-016 | ✅ implemented |
| FR-016 | FR-016-AC-1 through FR-016-AC-3 | TC-017 | ✅ implemented |
| FR-017 | FR-017-AC-1, FR-017-AC-2 | TC-017 | ✅ implemented |
| FR-018 | FR-018-AC-1 through FR-018-AC-3 | TC-018 | ✅ implemented |
| FR-019 | FR-019-AC-1, FR-019-AC-2 | TC-018 | ✅ implemented |
| FR-020 | FR-020-AC-1, FR-020-AC-2 | TC-018 | ✅ implemented |
| FR-023 | FR-023-AC-1 through FR-023-AC-5 | TC-035 | ✅ implemented |
| FR-025 | FR-025-AC-1 through FR-025-AC-8 | TC-038 | ✅ implemented against immutable QSL `f1700a92`, Quire Observation `9ac80e93`, Quire Protocol `34d1752e`, and tl-syntax `842d8255` owner revisions |
| FR-026 | FR-026-AC-1 through FR-026-AC-8 | TC-039 | ✅ implemented against immutable QSL `f1700a92`, Quire Observation `9ac80e93`, Quire Protocol `34d1752e`, tl-syntax `842d8255`, and tl-mltl `22862189` owner revisions |
| FR-027 | FR-027-AC-1 through FR-027-AC-6 | TC-040 | ✅ implemented against the exact nine-repository Task-011 manifest, immutable schema digests, and merged FR-025/FR-026 owner path |
| FR-028 | FR-028-AC-1 through FR-028-AC-5 | TC-041 | ✅ implemented architecture enablement |
| FR-029 | FR-029-AC-1 through FR-029-AC-3 | TC-042 | ✅ implemented through PRs #87/#88 |
| FR-030 | FR-030-AC-1 through FR-030-AC-3 | TC-042 | ✅ implemented through PRs #88–#91 |
| FR-031 | FR-031-AC-1 through FR-031-AC-3 | TC-042 | ✅ implemented through native replay PR #92 and codegen corpus PR #47 |
| FR-032 | FR-032-AC-1 through FR-032-AC-4 | TC-043 | ✅ implemented target-neutral admission; no target mapper credited |
| FR-033 | FR-033-AC-1 through FR-033-AC-5 | TC-043 | ✅ implemented bounded mapper/record accounting; no preservation default or partial record claim |
| FR-034 | FR-034-AC-1 through FR-034-AC-5 | TC-043 | ✅ implemented atomic generated package and downstream observer evidence |
| FR-035 | FR-035-AC-1 through FR-035-AC-4 | TC-044 | ✅ implemented through Contract IR #104 (`1baa5af`): strict I04 reader and exact independent lowering records |
| FR-036 | FR-036-AC-1 through FR-036-AC-4 | TC-045 | 🚧 planned after #99 review; runtime #16 and codegen #48/#49 consume it |
| FR-037 | FR-037-AC-1 through FR-037-AC-5 | TC-046 | 🚧 planned after #99 review; codegen #50 and runtime #16 own replay integration |
| FR-038 | FR-038-AC-1 through FR-038-AC-7 | TC-047, TC-048, TC-049, TC-050 | 🚧 planned in Contract IR #106 against QSpec `4780a9e6`: version dispatch, strict V2 reader, V1-to-V2 migration outcome and V2 lowering |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | repeated golden corpus and cross-platform comparison | TC-017, TC-019 | issue #9 same-process goldens implemented; cross-platform TC-019 planned |
| NFR-002 | vocabulary, schema, API, and exact Rust inspection | TC-015, TC-016, TC-019, TC-036 | AC-2/3/4 implemented; cross-platform AC-1 planned |
| NFR-003 | negative corpus, mutation, panic-free, and orphan checks | TC-017 through TC-019 | issue #9 version/orphan fail-closed checks implemented; full TC-019 planned |
| NFR-004 | repository/assurance/plan inspection | TC-014, TC-020, TC-021 | foundation covered |
| NFR-005 | exact toolchain declarations and executed compatibility matrix | TC-036, TC-037 | ✅ implemented |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-001 | exact registered code sets, precedence, structured fields, and no message parsing | TC-015 through TC-018, TC-038 through TC-040 | issue #6 through #10 and TC-038 through TC-040 closed code catalogs implemented |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-014 | Baseline, licenses, protected workflow, and publication lock agree | Inspection | P0 | NFR-004 | ✅ implemented |
| TC-015 | Package, revision, anchor, clause, dependency, and diagnostic identities conform | Integration | P0 | FR-011, FR-012, NFR-002, STD-001 | ✅ implemented |
| TC-016 | Types, expressions, short-circuiting, and definedness conform | Integration | P0 | FR-012..FR-015, NFR-002, STD-001 | ✅ implemented |
| TC-017 | Canonical bytes, digests, migrations, and orphan classes conform | Property | P0 | FR-016, FR-017, NFR-001, NFR-003 | ✅ implemented |
| TC-018 | Schema, corpus, diagnostics, dependencies, and interfaces conform | Integration | P0 | FR-018..FR-020 | ✅ implemented |
| TC-019 | Determinism, portability, and fail-closed metrics meet thresholds | Analysis | P0 | NFR-001..NFR-003 | 🚧 planned across issues #8–#10 |
| TC-020 | Five assurance artifacts declare boundaries, evidence, failures, and owner | Inspection | P0 | StR-003, NFR-004 | ✅ implemented |
| TC-021 | Composite review and dependency DAG preserve spec-first child gates; Python matrix and native orchestration controls enforce AC-5/6 | Inspection | P0 | NFR-004 | ✅ implemented |
| TC-035 | Derived executable projections bind complete typed clause populations through the public IR boundary | Integration | P0 | FR-023 | ✅ implemented |
| TC-036 | Exact Rust baseline declarations, stable-release trigger, and time-bounded older-version exception policy agree | Integration | P0 | NFR-005-AC-1, NFR-005-AC-4, NFR-005-AC-5, NFR-005-AC-6, NFR-002-AC-2 | ✅ implemented |
| TC-037 | Compiler, targets, formatting, lint, safety, supply-chain and repository-specific tools execute and classify failures correctly | Integration | P0 | NFR-005-AC-2, NFR-005-AC-3, NFR-005-AC-6 | ✅ implemented |
| TC-038 | Native predicate projections bind exact total-Boolean values to deterministic TL signals/propositions or expose a typed non-value state | Property | P0 | FR-025-AC-1, FR-025-AC-2, FR-025-AC-3, FR-025-AC-4, FR-025-AC-5, FR-025-AC-6, FR-025-AC-7, FR-025-AC-8 | ✅ implemented with real owner views/readers, deterministic goldens, strict re-readers, fail-closed identity/join checks, non-value preservation, corrections, and resource failpoints |
| TC-039 | Native temporal projections correspond exactly to supported TL profiles or fail closed | Integration | P0 | FR-026-AC-1, FR-026-AC-2, FR-026-AC-3, FR-026-AC-4, FR-026-AC-5, FR-026-AC-6, FR-026-AC-7, FR-026-AC-8, STD-001 | ✅ implemented with complete owner-checked leaf populations, sibling native/TL request construction, strict projection/join readback, independent future/past owner evaluation, exact event/fixed-sample mapping, timestamp refusal, correction lineage, 23 independent contract axes, and bounded no-partial-output failures |
| TC-040 | Export and strict-read the bounded non-authoritative ecosystem model | Property | P0 | FR-027-AC-1, FR-027-AC-2, FR-027-AC-3, FR-027-AC-4, FR-027-AC-5, FR-027-AC-6, STD-001 | ✅ implemented with exact campaign selection, immutable schemas, typed graph/topology refusal, deterministic export/re-export, every resource cliff, non-authoritative proposals, and the real owner bridge path |
| TC-041 | Preserve the Contract IR API while proving the model/owner/bridge Cargo graph is acyclic | Integration | P0 | FR-028-AC-1, FR-028-AC-2, FR-028-AC-3, FR-028-AC-4, FR-028-AC-5 | ✅ implemented architecture enablement |
| TC-042 | Versioned bounded-Kani profile, finite input/outcome firewall, module dispatch, provenance, and native counterexample replay conform | Integration | P0 | FR-029, FR-030, FR-031 | ✅ implemented across shared, arithmetic, graph, collection and native-replay suites; integrated codegen corpus at `73c82ad` |
| TC-043 | Output-mapping request, mapper seam, per-obligation records, limits, atomic package, and observer separation conform | Integration | P0 | FR-032, FR-033, FR-034 | ✅ implemented in `tests/output_mapping.rs` and model overflow tests; target-specific OCL/SysML/FRETish semantics excluded |
| TC-044 | Complete-V1 target-neutral ContractPackage lowering conforms | Property | P0 | FR-035 | ✅ implemented through Contract IR #104 (`1baa5af`); consumes merged QSpec I04 #66 and exercises exact/one-over reader limits |
| TC-045 | Exact backend capability negotiation conforms | Integration | P0 | FR-036 | 🚧 planned; mirrors QSpec TC-218 through runtime #16 and codegen #49 |
| TC-046 | Canonical backend counterexample replay conforms | Integration | P0 | FR-037 | 🚧 planned; mirrors QSpec TC-219 through codegen #50 and runtime #16 |
| TC-047 | CheckedPackage version dispatch keeps V1 frozen and V2 separate | Integration | P0 | FR-038-AC-1 | 🚧 planned in #106; mirrors QSpec TC-217 FR-322-AC-11 and FR-201-AC-5 |
| TC-048 | CheckedPackage V2 strict reader re-derives package and nominal identities | Property | P0 | FR-038-AC-2, FR-038-AC-3, FR-038-AC-4, FR-038-AC-5 | 🚧 planned in #106; mirrors QSpec TC-217 FR-322-AC-4, FR-322-AC-8 and FR-322-AC-10 |
| TC-049 | CheckedPackage V1 to V2 migration returns the exact MigrationOutcome | Integration | P0 | FR-038-AC-6 | 🚧 planned in #106; mirrors QSpec TC-217 FR-322-AC-12 |
| TC-050 | CheckedPackage V2 items lower independently with exact correspondence | Property | P0 | FR-038-AC-7 | 🚧 planned in #106; mirrors QSpec TC-217 FR-195-AC-1 through FR-195-AC-5 |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-036 cases | Coverage, error, edge | exact 1.98.1 in each applicable surface; mutations to 1.75, 1.85, floating `stable`, absent declarations and inconsistent minimum/qualification values; newer-stable event inside/outside seven days; adopted, justified-hold, expired-hold and inherited-pin dispositions |
| TC-037 cases | Coverage, error, edge | all-target test/build, warning-denied Clippy, 1.98.1 rustfmt, unsafe audit, cargo-deny/audit, and each declared target/tool; distinguish pass, launch/protocol/compiler incompatibility, formatting change, new lint, dependency vulnerability and shared-assurance rejection |
| TC-038 cases | Coverage, permutation, boundary, error, transition, edge | both Boolean values; every non-Boolean/non-final result kind; every identity axis; input-order permutations; complete/duplicate/omitted/conflicting populations; predicate/fact 10,000/10,001 and byte bounds; inside/outside deciding-fact loss; every cause and precedence; every projection/valuation state; correction/supersession/conflict; independently authored canonical preimage/digest goldens; deterministic allocation failpoints; real public tl-syntax reader/join failures; source/parser/evaluator purity guards |
| TC-039 cases | Coverage, boundary, error, transition, edge | closed/open event-position differential corpus for every admitted operator; interval boundaries `a=0`, `a=b`, `b=u32::MAX`, witness at each endpoint, witness absent, horizon beyond closure, and empty/one/multiple positions; exact fixed-sample mapping; one-position false-extension and until-lower-bound discriminators; timestamped, finite-window and past-time refusals; complete rectangular FR-025 per-position valuations plus cell replay/swapping, wrong position-to-anchor/snapshot/invocation binding, incomplete/unavailable/unsupported/failed/refused/conflicting cells and unused correspondences; exact TL trace/request construction with explicit false cells omitted only after verification; capture, clock, sample, history, silence, observation closure, late-data and superseding/invalidating direct-predecessor transitions; assessment execution, both progress/closure axes, truth, settlement basis, decision support and completeness mutated independently, including foreign clock/subject/scope/source/interval/history bindings and missing facts inside/outside settled support; every embedded and top-level contract selection and result-availability assertion mutated independently, including identity conflicts; one/both producer unavailable without fabricated result fields; every projection/result kind, cause, ordering and precedence; strict tagged-shape round trip and invalid-input refusal; 10,000/10,001 node/correspondence/position/valuation/support and byte/depth/string/cause bounds; deterministic allocation failpoints; real selected formula/trace/request/result readers; all three corrected independent canonical formula preimage/digest vectors; repeated-input identity and mutation of every formula/valuation/trace/request/correspondence/result binding dimension; parser/evaluator/ambient/network/foreign-runtime purity guards |
| TC-040 cases | Coverage, permutation, boundary, error, topology, authority | exact nine-repository graph and every input permutation; independent node, edge, revision, schema and relation mutations; duplicate/dangling/multiple-owner/ill-typed/self/cyclic edges; exact and one-over byte/depth/string/node-kind/edge/work bounds; deterministic allocation failpoints; model re-export byte equality; constructor privacy; compile-time absence of authority/evaluator/parser/network/plugin inputs; proposal remains unaccepted until a distinct reviewed owner revision is selected |
| TC-041 cases | Coverage, compatibility, dependency, feature, edge | Cargo metadata cycle check for default/all/minimum features; model package owner/TL dependency absence; root-package compatibility imports; existing schema/canonical/digest/diagnostic/corpus equality; QSL dependency-key alias build; locked bridge-and-QSL owner API composition build; dev/historical pin isolation; compile-fail probes for public owner constructors, callbacks, trait validators, trust flags and copied owner wire types |
| TC-042 cases | Coverage, boundary, error, transition, replay, provenance | every profile/matrix construct and unknown/conflicting/missing entry; each valid, duplicate, dangling, foreign, wrong-type, incomplete, unavailable and one-over-bound population/snapshot/reference/collection case; no-assumption invalid-input probes; every typed outcome kind and Boolean-field absence; arithmetic/definedness, object/reference/graph and collection/query dispatch ownership; generated artifact mutation for source/model/profile/module/tool/options/assumption/bound/dependency identity; every concrete Kani counterexample round-trips through native `runtime::execute`; replay mismatch, unavailable runtime and malformed packet remain typed non-success; repeat corpus parity with exact Kani executable/options digest |
| TC-047 cases | Coverage, compatibility, error | V1 and both V2 fixtures through the dispatcher; unknown and absent version; V2 bytes and V2-only node member through V1; V1 bytes through V2; equal digest bytes under the V1/V2 package domains; recorded V1 read/lower golden |
| TC-048 cases | Coverage, boundary, error, identity | both V2 fixtures; every vendored adverse mutation; malformed, duplicate, unknown member, noncanonical, stale digest, unreported/unsupported feature, dangling reference, incomplete source map; exact and one-over byte/depth/node/edge/occurrence/diagnostic/work limits; excluded and included package-preimage edits; every node-identity vector and invalid mutation; absent/foreign preimage; retained-preimage case/type/dependency/target contradictions |
| TC-049 cases | Coverage, error, ordering | both positive migration vectors; every refusal vector including missing-before-ambiguous and missing-before-stale ordering; target family and lock mismatch |
| TC-050 cases | Coverage, boundary, error, independence | every node family and nominal node; absent key; unsupported reachable tag; unbounded and bounded numeric type under a bounds-required profile; one-over work; mixed request sibling equality |
| TC-043 cases | Coverage, permutation, boundary, error, transition, identity, atomicity | each exact target profile independently without target semantics; nonempty ordered unique obligation selection; every source-fact state and disposition invariant; separate observation/protocol adequacy; missing/duplicate/foreign/stale/cross-profile/cross-wired inputs; zero/exact/just-over/overflow request, obligation, node, depth, work, record and emitted-byte limits; malformed and UTF-8-unsafe regions; cancellation/allocation/mapper failure; deterministic replay and mutation of every record/package identity member; path/time/locale/display/observer independence; no partial package or preservation fallback |
