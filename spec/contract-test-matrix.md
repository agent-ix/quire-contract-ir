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

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-011 through FR-015, FR-019, FR-023, FR-025, FR-026 | TC-015 through TC-018, TC-035, TC-038, TC-039 | existing rows implemented; FR-025/TC-038 and FR-026/TC-039 planned |
| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018 | ✅ implemented |
| StR-003 | FR-012, FR-014, FR-015, FR-017 through FR-020 | TC-015, TC-016, TC-018, TC-020 | ✅ implemented |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
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
| FR-025 | FR-025-AC-1 through FR-025-AC-8 | TC-038 | 🚧 planned; blocked on an accepted native checked-leaf/source-result/availability contract set, published TL signal-catalog schema/reader, and implementation routing |
| FR-026 | FR-026-AC-1 through FR-026-AC-8 | TC-039 | 🚧 planned; blocked on accepted FR-025, native temporal/result/availability/clock contracts and public TL formula/semantic/evaluator/result contracts |

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
| STD-001 | exact registered code sets, precedence, structured fields, and no message parsing | TC-015 through TC-018, TC-038, TC-039 | issue #6 through #10 codes implemented; issue #63/64 codes and TC-038/039 planned |

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
| TC-038 | Native predicate projections bind exact total-Boolean values to deterministic TL signals/propositions or expose a typed non-value state | Property | P0 | FR-025-AC-1, FR-025-AC-2, FR-025-AC-3, FR-025-AC-4, FR-025-AC-5, FR-025-AC-6, FR-025-AC-7, FR-025-AC-8 | 🚧 planned; blocked on accepted native checked-leaf/source-result/availability contracts and a published TL signal-catalog schema/reader |
| TC-039 | Native temporal projections correspond exactly to supported TL profiles or fail closed | Integration | P0 | FR-026-AC-1, FR-026-AC-2, FR-026-AC-3, FR-026-AC-4, FR-026-AC-5, FR-026-AC-6, FR-026-AC-7, FR-026-AC-8, STD-001 | 🚧 planned; blocked on accepted FR-025, native temporal/result/availability/clock contracts and public TL formula/semantic/evaluator/result contracts |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-036 cases | Coverage, error, edge | exact 1.98.1 in each applicable surface; mutations to 1.75, 1.85, floating `stable`, absent declarations and inconsistent minimum/qualification values; newer-stable event inside/outside seven days; adopted, justified-hold, expired-hold and inherited-pin dispositions |
| TC-037 cases | Coverage, error, edge | all-target test/build, warning-denied Clippy, 1.98.1 rustfmt, unsafe audit, cargo-deny/audit, and each declared target/tool; distinguish pass, launch/protocol/compiler incompatibility, formatting change, new lint, dependency vulnerability and shared-assurance rejection |
| TC-038 cases | Coverage, permutation, boundary, error, transition, edge | both Boolean values; every non-Boolean/non-final result kind; every identity axis; input-order permutations; complete/duplicate/omitted/conflicting populations; predicate/fact 10,000/10,001 and byte bounds; inside/outside deciding-fact loss; every cause and precedence; every projection/valuation state; correction/supersession/conflict; independently authored canonical preimage/digest goldens; deterministic allocation failpoints; real public tl-syntax reader/join failures; source/parser/evaluator purity guards |
| TC-039 cases | Coverage, boundary, error, transition, edge | closed/open event-position differential corpus for every admitted operator; interval boundaries `a=0`, `a=b`, `b=u32::MAX`, witness at each endpoint, witness absent, horizon beyond closure, and empty/one/multiple positions; exact fixed-sample mapping; one-position false-extension and until-lower-bound discriminators; timestamped, finite-window and past-time refusals; FR-025 incomplete/unavailable/mismatched plus unused correspondences; capture, clock, sample, history, silence, observation closure, late-data and direct-supersession transitions; every contract selection and result-availability assertion mutated independently; one/both producer unavailable without fabricated result fields; every projection/result kind, cause, ordering and precedence; strict tagged-shape round trip and invalid-input refusal; 10,000/10,001 node/correspondence and byte/depth/string/cause bounds; deterministic allocation failpoints; real selected formula/result readers; all three independent canonical formula preimage/digest vectors; repeated-input identity and mutation of every formula/correspondence/result binding dimension; parser/evaluator/ambient/network/foreign-runtime purity guards |
