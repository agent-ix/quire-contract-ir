---
id: SR-15311
title: "IR-641 IR consumer specification gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-641; spec/, src/, crates/, tests/, Makefile, AGENTS.md, CLAUDE.md"
review_set: subset
---

## Summary

Full-repository computed matrix has 68 criterion evidence gaps (64 inherited; four newly planned consumer Test criteria). A pre-existing authored trace on a helper cannot bind. These are not concealed by the specification-only delivery.

Ticket: IR-641. Reviewed PR #499, exact frozen candidate and fresh main range verified outside this artifact. The change is specification-only.

## Verdict

**FAIL** — Repository assurance remains incomplete; this is not a complete consumer adoption or passing final gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-035-AC-9 has no test carrying its criterion binding. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:183 |
| FND-002 | high | FR-038-AC-132 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4812 |
| FND-003 | high | FR-038-AC-135 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4815 |
| FND-004 | high | FR-038-AC-232 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4911 |
| FND-005 | high | FR-038-AC-233 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4912 |
| FND-006 | high | FR-038-AC-234 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4913 |
| FND-007 | high | FR-038-AC-235 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4914 |
| FND-008 | high | FR-038-AC-281 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4956 |
| FND-009 | high | FR-038-AC-282 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4957 |
| FND-010 | high | FR-038-AC-283 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4958 |
| FND-011 | high | FR-038-AC-284 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4959 |
| FND-012 | high | FR-038-AC-285 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4960 |
| FND-013 | high | FR-038-AC-286 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4961 |
| FND-014 | high | FR-038-AC-287 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4962 |
| FND-015 | high | FR-038-AC-288 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4963 |
| FND-016 | high | FR-038-AC-289 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4964 |
| FND-017 | high | FR-038-AC-290 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4965 |
| FND-018 | high | FR-038-AC-291 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4966 |
| FND-019 | high | FR-038-AC-292 has no test carrying its criterion binding. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4967 |
| FND-020 | high | NFR-006-M-1 has no test carrying its criterion binding. | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md:1 |
| FND-021 | high | NFR-006-M-2 has no test carrying its criterion binding. | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md:1 |
| FND-022 | high | NFR-006-M-3 has no test carrying its criterion binding. | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md:1 |
| FND-023 | high | FR-045-AC-1 has no test carrying its criterion binding. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:72 |
| FND-024 | high | FR-045-AC-2 has no test carrying its criterion binding. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:73 |
| FND-025 | high | FR-045-AC-3 has no test carrying its criterion binding. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:74 |
| FND-026 | high | FR-045-AC-4 has no test carrying its criterion binding. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:75 |
| FND-027 | high | FR-045-AC-5 has no test carrying its criterion binding. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:76 |
| FND-028 | high | FR-045-AC-6 has no test carrying its criterion binding. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md:77 |
| FND-029 | high | FR-055-AC-1 has no test carrying its criterion binding. | spec/core/functional/FR-055-owned-lock-coherence-gate.md:33 |
| FND-030 | high | FR-055-AC-2 has no test carrying its criterion binding. | spec/core/functional/FR-055-owned-lock-coherence-gate.md:34 |
| FND-031 | high | FR-055-AC-3 has no test carrying its criterion binding. | spec/core/functional/FR-055-owned-lock-coherence-gate.md:35 |
| FND-032 | high | FR-055-AC-5 has no test carrying its criterion binding. | spec/core/functional/FR-055-owned-lock-coherence-gate.md:37 |
| FND-033 | high | NFR-001-AC-2 has no test carrying its criterion binding. | spec/core/non-functional/NFR-001-determinism.md:36 |
| FND-034 | high | NFR-001-AC-3 has no test carrying its criterion binding. | spec/core/non-functional/NFR-001-determinism.md:37 |
| FND-035 | high | NFR-001-M-1 has no test carrying its criterion binding. | spec/core/non-functional/NFR-001-determinism.md:1 |
| FND-036 | high | NFR-001-M-2 has no test carrying its criterion binding. | spec/core/non-functional/NFR-001-determinism.md:1 |
| FND-037 | high | NFR-001-M-3 has no test carrying its criterion binding. | spec/core/non-functional/NFR-001-determinism.md:1 |
| FND-038 | high | NFR-002-M-1 has no test carrying its criterion binding. | spec/core/non-functional/NFR-002-portability-independence.md:1 |
| FND-039 | high | NFR-002-M-2 has no test carrying its criterion binding. | spec/core/non-functional/NFR-002-portability-independence.md:1 |
| FND-040 | high | NFR-003-M-1 has no test carrying its criterion binding. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-041 | high | NFR-003-M-2 has no test carrying its criterion binding. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-042 | high | NFR-003-M-3 has no test carrying its criterion binding. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-043 | high | NFR-003-M-4 has no test carrying its criterion binding. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:1 |
| FND-044 | high | FR-030-AC-4 has no test carrying its criterion binding. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:157 |
| FND-045 | high | FR-030-AC-5 has no test carrying its criterion binding. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:158 |
| FND-046 | high | FR-030-AC-7 has no test carrying its criterion binding. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:160 |
| FND-047 | high | FR-036-AC-1 has no test carrying its criterion binding. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:58 |
| FND-048 | high | FR-036-AC-2 has no test carrying its criterion binding. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:59 |
| FND-049 | high | FR-036-AC-3 has no test carrying its criterion binding. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:60 |
| FND-050 | high | FR-036-AC-4 has no test carrying its criterion binding. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:61 |
| FND-051 | high | FR-036-AC-5 has no test carrying its criterion binding. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md:62 |
| FND-052 | high | FR-037-AC-6 has no test carrying its criterion binding. | spec/kani/functional/FR-037-canonical-backend-replay-and-qualification.md:66 |
| FND-053 | high | FR-039-AC-1 has no test carrying its criterion binding. | spec/kani/functional/FR-039-root-crate-public-interface.md:113 |
| FND-054 | high | FR-039-AC-2 has no test carrying its criterion binding. | spec/kani/functional/FR-039-root-crate-public-interface.md:114 |
| FND-055 | high | FR-039-AC-3 has no test carrying its criterion binding. | spec/kani/functional/FR-039-root-crate-public-interface.md:115 |
| FND-056 | high | FR-039-AC-4 has no test carrying its criterion binding. | spec/kani/functional/FR-039-root-crate-public-interface.md:116 |
| FND-057 | high | FR-016-AC-1 has no test carrying its criterion binding. | spec/model/functional/FR-016-canonicalization-digests.md:148 |
| FND-058 | high | FR-016-AC-3 has no test carrying its criterion binding. | spec/model/functional/FR-016-canonicalization-digests.md:150 |
| FND-059 | high | FR-017-AC-2 has no test carrying its criterion binding. | spec/model/functional/FR-017-version-orphan-coverage.md:97 |
| FND-060 | high | FR-019-AC-4 has no test carrying its criterion binding. | spec/model/functional/FR-019-rust-library-interface.md:219 |
| FND-061 | high | FR-028-AC-4 is bound only to an ignored test. | spec/model/functional/FR-028-separate-cycle-free-contract-model.md:63 |
| FND-062 | high | FR-052-AC-7 has no test carrying its criterion binding. | spec/model/functional/FR-052-source-obligation-correspondence.md:471 |
| FND-063 | high | FR-052-AC-9 has no test carrying its criterion binding. | spec/model/functional/FR-052-source-obligation-correspondence.md:472 |
| FND-064 | high | FR-052-AC-10 has no test carrying its criterion binding. | spec/model/functional/FR-052-source-obligation-correspondence.md:473 |
| FND-065 | high | FR-052-AC-11 has no test carrying its criterion binding. | spec/model/functional/FR-052-source-obligation-correspondence.md:474 |
| FND-066 | high | FR-052-AC-16 has no test carrying its criterion binding. | spec/model/functional/FR-052-source-obligation-correspondence.md:479 |
| FND-067 | high | FR-052-AC-26 has no test carrying its criterion binding. | spec/model/functional/FR-052-source-obligation-correspondence.md:489 |
| FND-068 | high | FR-052-AC-27 has no test carrying its criterion binding. | spec/model/functional/FR-052-source-obligation-correspondence.md:490 |
| FND-069 | medium | An authored FR-092 trace on the non-test fixture_key helper cannot bind an evidence criterion. | tests/it/checked_package_v2_frame_entries.rs:390 |

## Coverage

No applicable AssuranceProfile with required review selection was found in the spec. No active exception was granted. CLI FR-027 was read from its published source; no copying of its checker logic is proposed. User-authorized dependency preservation and script-removal inspection are explicit task constraints, not a new hash/pin catalog.

Plan completion: not assessed

Reconciliation: quoin matrix 0.28.4 and quire matrix (CLI 0.36.2, engine 0.50.2). All 604 criteria have no run evidence; this is a static audit, not a test-execution claim. Whole repository: 524 tagged, 67 untagged, one tagged-by-ignored-test, 12 method-without-symbol. Exact fresh-base archive: 599 criteria, 524 tagged, 63 untagged, one tagged-by-ignored-test, 11 method-without-symbol. Existing criterion records are byte-equivalent at the JSON-object level; new criteria are exactly FR-055 AC1–5.

The four new untagged criteria (AC1,2,3,5) belong to IR-641 consumer implementation after this specification stage. AC4 is Inspection and legitimately mints no test symbol. Existing 64 evidence gaps and helper trace debt remain IR-878 program debt; no claim of automatic closure. Coverage untracked_symbols is zero. The 72 unmatched tag hits include prose/cross-repo IDs and are not assumed to be 72 bad criterion bindings. Fifteen diagnostics include absent archetype channels, existing uncatalogued method cells and the genuine helper-tag diagnostic recorded above; no new FR-055 declaration is excluded. The engine oracle-similarity suspicion compares two existing test helpers and is not, by itself, evidence of a copied production oracle.

Reverse inventory: 73 Rust source files and 150 total source/test files scanned; public surface grouped into the model construction/validation API (FR-019), checked-package V1/V2 readers/encoding and projections (FR-035/038/048–054), canonical identity/anchors/dependencies (FR-011/012), numeric/expression and output mapping (FR-014/032–034), conformance CLI (FR-018), and bounded Kani profile/ABI/dispatch/outcome lowerings (FR-029–031/036–039). Repository public exports and command entry were examined, with no new unowned product behavior introduced. Textual empty-function hits were checked: count_scan_step is a cfg-disabled production instrumentation no-op; compile-fail and scanner-test source strings are controls, not hollow product bodies. No new source or test stub was found. This is not an execution or full semantic correctness proof of the unchanged product.

Semantic review: not run as a full intent-to-test-to-code execution review; scoped spec correctness was reviewed. Cargo, rustfmt, native/backend and aggregate gates were not run in this spec-only reviewer pass.

## Examined Scope

```yaml
scope:
- id: FR-035-AC-1
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Each V1-BACK-001 through V1-BACK-004 and
- id: FR-035-AC-2
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: A source, type, anchor, identity, bound,
- id: FR-035-AC-3
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Mixed supported and refused requests ret
- id: FR-035-AC-4
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'Every local reference resolves by typed '
- id: FR-035-AC-5
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: One call emits exactly one FR-330 `quire
- id: FR-035-AC-6
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'For an admitted lowered root, the typed '
- id: FR-035-AC-7
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: A selected dependency reference in the v
- id: FR-035-AC-8
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Querying a non-`lowered` requested key r
- id: FR-035-AC-9
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: The typed profile exposes `required_sour
- id: FR-035-AC-10
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Given an otherwise valid canonical `Cont
- id: FR-035-AC-11
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'Given otherwise valid canonical package '
- id: FR-035-AC-12
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Given an otherwise valid canonical `Cont
- id: FR-035-AC-13
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Given otherwise valid `ContractPackageWi
- id: FR-035-AC-14
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Given an otherwise valid canonical `Cont
- id: FR-035-AC-15
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: For an otherwise valid canonical package
- id: FR-035-AC-16
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: For an otherwise valid canonical package
- id: FR-035-AC-17
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Given an otherwise valid canonical `Cont
- id: FR-035-AC-18
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Given an otherwise valid canonical Contr
- id: FR-038-AC-1
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Each positive V2 package fixture this re
- id: FR-038-AC-2
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The reader refuses malformed, duplicate-
- id: FR-038-AC-3
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Exact byte, node, edge, occurrence, diag
- id: FR-038-AC-4
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The recomputed package id equals each po
- id: FR-038-AC-5
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Every nominal preimage of the in-repo no
- id: FR-038-AC-6
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Every admitted node family lowers indepe
- id: FR-038-AC-9
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The shipped default read-limit policy is
- id: FR-038-AC-7
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Lowering every node of every positive fi
- id: FR-038-AC-8
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A closure holding both an out-of-profile
- id: FR-038-AC-10
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.model_selections` entry that rep
- id: FR-038-AC-11
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.model_selections` array carrying
- id: FR-038-AC-12
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `creates` or `deletes` entry's eligibi
- id: FR-038-AC-13
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An entry naming a digest that is not amo
- id: FR-038-AC-14
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A frame node carrying both a meaning-joi
- id: FR-038-AC-15
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package carrying two defective `state`
- id: FR-038-AC-17
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Each declared wire member the V2 contrac
- id: FR-038-AC-18
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For a keyed input that reaches the graph
- id: FR-038-AC-19
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.model_selections` array carrying
- id: FR-038-AC-20
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A selection binds by `identity`, so a `l
- id: FR-038-AC-21
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A node whose declared `qualified_name` d
- id: FR-038-AC-22
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A QSL-shaped package holding a function '
- id: FR-038-AC-23
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An application node whose `dependencies`
- id: FR-038-AC-24
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Every refusal about a value carries the '
- id: FR-038-AC-25
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An `unknown_contract_version` refusal po
- id: FR-038-AC-26
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Each one-over limit other than the byte '
- id: FR-038-AC-27
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A domain package selection is admitted o
- id: FR-038-AC-28
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A domain package document's declarations
- id: FR-038-AC-29
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose lock selects a domain pa
- id: FR-038-AC-30
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Reading a selected domain package and re
- id: FR-038-AC-31
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose lock and identity preima
- id: FR-038-AC-32
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `dependency_selections` entry whose `p
- id: FR-038-AC-33
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `dependency_selections` entry repeatin
- id: FR-038-AC-35
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose `function` call has a `d
- id: FR-038-AC-36
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `dependency_reference` with a bare-dig
- id: FR-038-AC-37
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A `dependency_selections` entry with no '
- id: FR-038-AC-293
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Given an otherwise valid package with di
- id: FR-038-AC-38
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A dependency function whose parameter is
- id: FR-038-AC-39
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Under a bounds-required profile, `x + 1`
- id: FR-038-AC-40
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Under a bounds-required profile a `bound
- id: FR-038-AC-41
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Under a bounds-required profile a `scala
- id: FR-038-AC-42
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An enum's operand family is `ordered_enu
- id: FR-038-AC-43
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`structural.eq` over an all-integer reco'
- id: FR-038-AC-44
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`operation.leaves` is exactly the derive'
- id: FR-038-AC-45
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package a producer emits under the con
- id: FR-038-AC-46
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose definition references ar
- id: FR-038-AC-47
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose source references are ex
- id: FR-038-AC-48
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A definition reference at a lock, `ident
- id: FR-038-AC-49
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An `operation.laws[].definition` that ca
- id: FR-038-AC-50
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row, `source_map` regio
- id: FR-038-AC-51
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A definition reference at a lock, `ident
- id: FR-038-AC-52
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A source reference at a `lock.sources` r
- id: FR-038-AC-53
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row whose `digest_domai
- id: FR-038-AC-54
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row, whose `digest_doma
- id: FR-038-AC-55
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `source_map` region `source` or `diagn
- id: FR-038-AC-56
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An `operation.laws[].definition` of a va
- id: FR-038-AC-57
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An `operation.laws[].definition` of a pr
- id: FR-038-AC-58
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The operation catalog read over supplied
- id: FR-038-AC-59
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A `lock.definition_selections` row that '
- id: FR-038-AC-60
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose one definition row's `id
- id: FR-038-AC-61
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `source` or `definition` nominal owner
- id: FR-038-AC-62
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A package whose `lock.model_selections` '
- id: FR-038-AC-63
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose `lock.dependency_selecti
- id: FR-038-AC-64
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Selections bind by identity and content '
- id: FR-038-AC-65
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Every application operator class, operat
- id: FR-038-AC-67
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An unknown identity refuses `unknown-ope
- id: FR-038-AC-68
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: At the operation check of a `quire.op.te
- id: FR-038-AC-69
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A member of kind `temporal_interval` or '
- id: FR-038-AC-70
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`structural.eq` over `record Node { labe'
- id: FR-038-AC-71
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Over `Node`, the recursion leaf placed b
- id: FR-038-AC-72
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The leaf count and derivation over a cyc
- id: FR-038-AC-73
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Under a bounds-required profile a positi
- id: FR-038-AC-74
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For every in-repo positive fixture, the '
- id: FR-038-AC-75
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For every in-repo positive fixture, the '
- id: FR-038-AC-76
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For every in-repo positive fixture, the '
- id: FR-038-AC-77
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `CheckedSemanticGraphV2` holding one n
- id: FR-038-AC-78
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The encodes of FR-038-AC-77, run under a
- id: FR-038-AC-79
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package document holding the integer 9
- id: FR-038-AC-80
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`quire-canonical` is a `branch = "main"`'
- id: FR-038-AC-81
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Observed at the operation check of a bod
- id: FR-038-AC-82
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Observed at the operation check, `quire.
- id: FR-038-AC-83
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Observed at the operation check, `quire.
- id: FR-038-AC-84
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Observed at the operation check, `quire.
- id: FR-038-AC-85
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Observed at the operation check, `quire.
- id: FR-038-AC-86
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Observed at the operation check, `quire.
- id: FR-038-AC-87
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The operand family of a node is exactly,
- id: FR-038-AC-88
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The preimage of an application node that
- id: FR-038-AC-89
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Every identity digest of an unchanged id
- id: FR-038-AC-90
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`NominalIdentityPreimage::digest` takes '
- id: FR-038-AC-91
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The files `canonical.rs`, `binding.rs`, '
- id: FR-038-AC-92
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The application node preimage, `LoweredN
- id: FR-038-AC-93
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A selected model document holding the nu
- id: FR-038-AC-94
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A selected model document whose bytes ar
- id: FR-038-AC-95
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Lowering encodes under the byte limit th
- id: FR-038-AC-96
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Each of the fifteen `temporal_formula` i
- id: FR-038-AC-97
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'On each of the eight interval operators '
- id: FR-038-AC-98
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The `fairness` member of `quire.op.tempo
- id: FR-038-AC-99
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package holding a `composite_type`/`un
- id: FR-038-AC-100
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Placement in both directions: an applica'
- id: FR-038-AC-101
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Admitted temporal/formula and expression
- id: FR-038-AC-102
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The temporal step runs after the frame a
- id: FR-038-AC-103
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A clause whose `over` argument is a `ref
- id: FR-038-AC-104
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Under a clause whose `temporal_profile` '
- id: FR-038-AC-108
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Each of the five members of QSpec FR-250
- id: FR-038-AC-105
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The temporal and case operands are check
- id: FR-038-AC-106
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`quire.op.structural.eq` over two values'
- id: FR-038-AC-107
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Read from the authoritative `quire-speci
- id: FR-038-AC-109
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A selected model document holding at `/p
- id: FR-038-AC-110
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The cause of each refusal FR-038-AC-93 n
- id: FR-038-AC-111
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package document holding the number `0
- id: FR-038-AC-112
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`make conformance-qspec` reads `proposal'
- id: FR-038-AC-113
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`make conformance-qspec` reads `proposal'
- id: FR-038-AC-114
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A node body in which an application of a
- id: FR-038-AC-115
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `case` application nested inside anoth
- id: FR-038-AC-116
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package whose `node_id`s, `identity_pr
- id: FR-038-AC-117
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`CheckedPackageReadLimits` has no depth '
- id: FR-038-AC-118
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Each of the five `body_grammar_mutations
- id: FR-038-AC-119
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'On each of the eight interval operators '
- id: FR-038-AC-120
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Each of these timed-form intervals refus
- id: FR-038-AC-121
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Under `quire.temporal.timed/v1`, `lower`
- id: FR-038-AC-122
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The timed form `{lower, upper, lower_end
- id: FR-038-AC-123
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Tamper regression (IR-627; implemented f
- id: FR-038-AC-124
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Closed body of each derived shape (IR-62
- id: FR-038-AC-125
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Self-typing and `semantic_type` (IR-627;
- id: FR-038-AC-126
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Indirect redirection (IR-627; implemente
- id: FR-038-AC-127
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Stage and order (IR-627 baseline impleme
- id: FR-038-AC-128
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Exact bounds at the extremes (IR-627; im
- id: FR-038-AC-129
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Trust root (IR-627; implemented). A pack
- id: FR-038-AC-130
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: One derivation (IR-627; implemented by t
- id: FR-038-AC-131
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED; new IR-630 controls UNRUN. Ever
- id: FR-038-AC-132
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: No copy (IR-627 checkout-reference harne
- id: FR-038-AC-133
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Positive fixtures carry derived keys (IR
- id: FR-038-AC-134
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: In-repo fixtures carry derived keys (IR-
- id: FR-038-AC-135
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Adverse mutations still reach their stag
- id: FR-038-AC-136
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Fields as values (IR-628; implemented, u
- id: FR-038-AC-137
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Effective set is the one admission resol
- id: FR-038-AC-138
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Every member kind (IR-628; implemented, '
- id: FR-038-AC-139
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Absence and refusal semantics (IR-628; i
- id: FR-038-AC-140
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Independent of any node body (IR-628; im
- id: FR-038-AC-141
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: One derivation with admission (IR-628; i
- id: FR-038-AC-142
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Public shape and stability (IR-628; impl
- id: FR-038-AC-143
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Pure, total, bounded (IR-628; implemente
- id: FR-038-AC-144
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Resolution is charged once, at admission
- id: FR-038-AC-145
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN. Correctly keyed
- id: FR-038-AC-146
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; retires ordinar
- id: FR-038-AC-147
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; retires off-cyc
- id: FR-038-AC-148
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; replaces former
- id: FR-038-AC-149
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN. Group membershi
- id: FR-038-AC-150
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED (IR-630); UNRUN; the prior owner
- id: FR-038-AC-151
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented by IR-644 and verified by TC
- id: FR-038-AC-152
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented by IR-644 and verified by TC
- id: FR-038-AC-153
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A declared record, tuple and function ea
- id: FR-038-AC-154
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Omitting a required node or projection o
- id: FR-038-AC-155
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Before structural key re-derivation, a `
- id: FR-038-AC-159
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For admitted `quire.op.integer.add`, `qu
- id: FR-038-AC-160
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A reference to a `value`/`parameter` ret
- id: FR-038-AC-161
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For an admitted inline integer literal a
- id: FR-038-AC-162
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An unknown graph node, a known non-appli
- id: FR-038-AC-163
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A crate-internal mutation of an admitted
- id: FR-038-AC-164
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An external Rust API fixture calls the a
- id: FR-038-AC-165
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-661). A selected, in
- id: FR-038-AC-166
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED / UNRUN (IR-661). Changing only '
- id: FR-038-AC-167
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED / UNRUN (IR-661). An unresolved '
- id: FR-038-AC-168
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-661). Repeating a re
- id: FR-038-AC-169
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-661). `relationship_
- id: FR-038-AC-170
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-661). Forward and in
- id: FR-038-AC-171
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-661). For `quire.op.
- id: FR-038-AC-172
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED / UNRUN (IR-661). A homogeneous '
- id: FR-038-AC-173
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED / UNRUN (IR-661). Removing direc
- id: FR-038-AC-174
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-175
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-176
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-654 code; external confo
- id: FR-038-AC-177
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented by `composite_application_op
- id: FR-038-AC-178
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented by `composite_application_op
- id: FR-038-AC-179
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented by `composite_application_op
- id: FR-038-AC-180
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implementation exists in `composite_appl
- id: FR-038-AC-181
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented by `composite_application_op
- id: FR-038-AC-182
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented by `composite_application_op
- id: FR-038-AC-183
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented and verified by TC-048 and T
- id: FR-038-AC-184
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented and verified by TC-048 (IR-6
- id: FR-038-AC-185
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented and verified by TC-048 (IR-6
- id: FR-038-AC-186
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-187
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-188
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-189
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-190
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-191
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-192
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-193
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; source Inspection completed
- id: FR-038-AC-194
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; source Inspection completed
- id: FR-038-AC-195
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The repository lint gate runs both Clipp
- id: FR-038-AC-196
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED; final-head runtime/Clippy c
- id: FR-038-AC-197
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-690) by `composite_appli
- id: FR-038-AC-198
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-690); TC-048 compares in
- id: FR-038-AC-199
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-690); TC-048 checks Miss
- id: FR-038-AC-200
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-690); TC-048 independent
- id: FR-038-AC-201
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (IR-690); source inspection '
- id: FR-038-AC-202
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); tagged local contr
- id: FR-038-AC-203
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); tagged local contr
- id: FR-038-AC-204
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); the tagged QSL pro
- id: FR-038-AC-205
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); tagged local contr
- id: FR-038-AC-206
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); tagged local contr
- id: FR-038-AC-207
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); tagged local contr
- id: FR-038-AC-208
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); tagged local contr
- id: FR-038-AC-209
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-703); tagged local contr
- id: FR-038-AC-210
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-484). For a valid self-t
- id: FR-038-AC-211
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-484). `quire.op.control.
- id: FR-038-AC-212
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-484). A body-root or arg
- id: FR-038-AC-213
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-484). `quire.op.quantity
- id: FR-038-AC-222
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-742). Through `CheckedPa
- id: FR-038-AC-223
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-742). Through `CheckedPa
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-225). Through `CheckedPa
- id: FR-038-AC-248
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-225). For a catalogued `
- id: FR-038-AC-249
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (IR-225). A `type_argument` '
- id: FR-038-AC-250
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-225). Through the public
- id: FR-038-AC-251
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-225; amended from an exp
- id: FR-038-AC-214
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'CODE-GATED (IR-575; isolated 2026-10-10 '
- id: FR-038-AC-215
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A selected model document that reads as '
- id: FR-038-AC-216
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Bytes of a selected model document that '
- id: FR-038-AC-217
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `dependency_selections` entry whose su
- id: FR-038-AC-218
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The `expected` and `actual` members of `
- id: FR-038-AC-224
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-490). A private operat
- id: FR-038-AC-225
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-739; tagged reader and c
- id: FR-038-AC-226
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-739; tagged reader and c
- id: FR-038-AC-227
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-739; tagged reader and c
- id: FR-038-AC-228
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-491). Through `Checked
- id: FR-038-AC-229
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-491). For `structural.
- id: FR-038-AC-230
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-491). For every catalo
- id: FR-038-AC-231
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-491). An otherwise val
- id: FR-038-AC-232
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-691). A fresh QSL FR-0
- id: FR-038-AC-233
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-691). In crate-interna
- id: FR-038-AC-234
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-691). A narrow whose a
- id: FR-038-AC-235
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-691). Independently co
- id: FR-038-AC-236
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-777). A selected Semanti
- id: FR-038-AC-237
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-777). A value typed by t
- id: FR-038-AC-238
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'IMPLEMENTED (IR-777). An empty selected '
- id: FR-038-AC-239
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-782). A package with a
- id: FR-038-AC-240
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-782). An extra depende
- id: FR-038-AC-241
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-782). A carrier with a
- id: FR-038-AC-294
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented; tagged native public-reader
- id: FR-038-AC-295
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented; tagged native public-reader
- id: FR-038-AC-242
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-800; tagged reader tests
- id: FR-038-AC-243
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-800; tagged reader tests
- id: FR-038-AC-244
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-800; tagged reader tests
- id: FR-038-AC-245
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-800; tagged reader tests
- id: FR-038-AC-252
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-824). This criterion sup
- id: FR-038-AC-253
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-824). For `quire.op.mode
- id: FR-038-AC-254
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-824). For `quire.op.mode
- id: FR-038-AC-255
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-824). For `quire.op.mode
- id: FR-038-AC-256
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-824). For `quire.op.mode
- id: FR-038-AC-257
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-824). A `quire.op.model.
- id: FR-038-AC-258
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-832). For an admitted `v
- id: FR-038-AC-259
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-832). `parameter_declara
- id: FR-038-AC-260
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-832). Two admitted param
- id: FR-038-AC-261
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-832). For an admitted `q
- id: FR-038-AC-262
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-851). When `relationship
- id: FR-038-AC-263
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-851). For admitted reach
- id: FR-038-AC-264
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-851). `relationship_end`
- id: FR-038-AC-265
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-851). When a crate-inter
- id: FR-038-AC-266
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-851). When `relationship
- id: FR-038-AC-267
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-851). An external Rust A
- id: FR-038-AC-268
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-851). `model_object_fiel
- id: FR-038-AC-269
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). When `reaches_popu
- id: FR-038-AC-270
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). When `reaches_popu
- id: FR-038-AC-271
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). `reaches_populatio
- id: FR-038-AC-272
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). A reaches applicat
- id: FR-038-AC-273
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). A reaches applicat
- id: FR-038-AC-274
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). A reaches applicat
- id: FR-038-AC-275
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). A `quire.op.model.
- id: FR-038-AC-276
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). A reaches or navig
- id: FR-038-AC-277
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). When `reaches_popu
- id: FR-038-AC-278
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). An external Rust A
- id: FR-038-AC-279
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). A reaches applicat
- id: FR-038-AC-280
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-852). A reaches applicat
- id: FR-038-AC-281
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A reader-admitted protocol member has a '
- id: FR-038-AC-282
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Each of the seven protocol member forms '
- id: FR-038-AC-283
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A protocol node derives its `quire.appli
- id: FR-038-AC-284
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The reader runs a protocol step after st
- id: FR-038-AC-285
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Every role, channel, queue, obligation a
- id: FR-038-AC-286
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A role admits the FR-465 static and repl
- id: FR-038-AC-287
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A clause names exactly one `finish` obli
- id: FR-038-AC-288
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Each of the thirteen control operators a
- id: FR-038-AC-289
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A `compensate` body admits the thirteen '
- id: FR-038-AC-290
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For each clause the reader constructs th
- id: FR-038-AC-291
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Binder levels start at the introducing c
- id: FR-038-AC-292
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: At a `choice` or `repeat` decision the r
- id: FR-038-AC-246
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-803). For a reader-admit
- id: FR-038-AC-303
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A self-typed `relation`/`population` nod
- id: FR-038-AC-304
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: After FR-154 admission, the reader joins
- id: FR-038-AC-305
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`quire.op.model.all_instances` and `quir'
- id: FR-038-AC-306
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A declared population reference used as '
- id: FR-038-AC-307
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The reader consumes QSpec's selected-che
- id: FR-040-AC-1
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `modifies` array holding a `relationsh
- id: FR-040-AC-2
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A relationship entry naming an object ty
- id: FR-040-AC-3
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A field entry on a model declaration obj
- id: FR-040-AC-4
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'Two field entries of one declaring node '
- id: FR-040-AC-5
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A frame whose `semantic_type` names a no
- id: FR-040-AC-6
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: Each of the fifteen `model` forms admits
- id: FR-040-AC-7
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: An operation anchor with the three bindi
- id: FR-040-AC-8
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: An invariant, a precondition and a postc
- id: FR-040-AC-9
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'For `Order.scaled(n: Integer): Integer`,'
- id: FR-040-AC-10
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `quire.op.state.clause` application st
- id: FR-040-AC-11
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `value`/`parameter` body of `name` the
- id: FR-040-AC-12
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'A package carrying a frame defect and a '
- id: FR-040-AC-13
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `quire.op.model.reaches_field` applica
- id: FR-040-AC-14
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'IMPLEMENTED (IR-770). With an authentic '
- id: FR-040-AC-15
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'IMPLEMENTED (IR-770). A selected record '
- id: FR-040-AC-16
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: IMPLEMENTED (IR-770). Selected record ow
- id: FR-040-AC-17
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A frame body carrying `modifies` field e
- id: FR-040-AC-18
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `field` entry with no `scope`, a `fiel
- id: FR-040-AC-19
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A frame body with no `deletes_scoped` me
- id: FR-040-AC-20
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `deletes_scoped` entry of a `kind` oth
- id: FR-040-AC-21
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `deletes_scoped` that does not hold th
- id: FR-040-AC-22
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'The `modifies` entries of one declaring '
- id: FR-040-AC-23
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: Two `field` entries sharing a declaratio
- id: FR-040-AC-24
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'A frame holding a `conflicting-binding` '
- id: FR-040-AC-25
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'A frame holding a `conflicting-binding` '
- id: FR-040-AC-26
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `scope` or a `deletes_scoped` entry ch
- id: FR-040-AC-27
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A `parameter` entry of a frame's `delete
- id: FR-040-AC-28
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: Two anchors of one `context` referencing
- id: FR-040-AC-29
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: A package carrying a frame-step defect a
- id: FR-040-AC-30
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: An anchor whose `context` is not a model
- id: FR-047-AC-1
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: A vacant domain key accepts its exact by
- id: FR-047-AC-2
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: Different raw bytes under an occupied do
- id: FR-047-AC-3
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: Equal admitted-content dependencies in d
- id: FR-047-AC-4
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: Unequal admitted dependencies under an o
- id: FR-047-AC-5
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: One and two exact feature declarations y
- id: FR-047-AC-6
  path: spec/checked_package/functional/FR-047-preserve-first-evidence-entry.md
  role: examined
  excerpt: Public model-crate callers handle both i
- id: FR-049-AC-1
  path: spec/checked_package/functional/FR-049-admitted-scalar-result-context.md
  role: examined
  excerpt: For the reader-admitted QSL source `func
- id: FR-049-AC-2
  path: spec/checked_package/functional/FR-049-admitted-scalar-result-context.md
  role: examined
  excerpt: For a QSL item with `None`, the expressi
- id: FR-049-AC-3
  path: spec/checked_package/functional/FR-049-admitted-scalar-result-context.md
  role: examined
  excerpt: For `Some`, an absent, non-lowered or un
- id: FR-049-AC-4
  path: spec/checked_package/functional/FR-049-admitted-scalar-result-context.md
  role: examined
  excerpt: A non-lowered expression root returns it
- id: FR-049-AC-5
  path: spec/checked_package/functional/FR-049-admitted-scalar-result-context.md
  role: examined
  excerpt: The result identifies the exact applicat
- id: FR-049-AC-6
  path: spec/checked_package/functional/FR-049-admitted-scalar-result-context.md
  role: examined
  excerpt: Contextual lowering emits exactly the ex
- id: FR-050-AC-1
  path: spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md
  role: examined
  excerpt: For QSL FR-062 RR-13, the checked `ratio
- id: FR-050-AC-2
  path: spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md
  role: examined
  excerpt: Two checked items sharing one applicatio
- id: FR-050-AC-3
  path: spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md
  role: examined
  excerpt: The public guard query returns the corre
- id: FR-050-AC-4
  path: spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md
  role: examined
  excerpt: 'Mutating only the supplied guard key to '
- id: FR-050-AC-5
  path: spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md
  role: examined
  excerpt: 'Every guard root has an ordinary FR-035 '
- id: FR-051-AC-1
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: A default-feature external consumer cons
- id: FR-051-AC-2
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: For an otherwise valid package, encoding
- id: FR-051-AC-3
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: 'An external producer mints `package_id` '
- id: FR-051-AC-4
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: For empty, nested, and deeply nested nod
- id: FR-054-AC-1
  path: spec/checked_package/functional/FR-054-admitted-scalar-operand-projection.md
  role: examined
  excerpt: For a reader-admitted positive scalar ap
- id: FR-054-AC-2
  path: spec/checked_package/functional/FR-054-admitted-scalar-operand-projection.md
  role: examined
  excerpt: For an admitted application whose two ar
- id: FR-054-AC-3
  path: spec/checked_package/functional/FR-054-admitted-scalar-operand-projection.md
  role: examined
  excerpt: For admitted applications with unbounded
- id: FR-054-AC-4
  path: spec/checked_package/functional/FR-054-admitted-scalar-operand-projection.md
  role: examined
  excerpt: A missing, foreign, non-lowered or non-a
- id: FR-054-AC-5
  path: spec/checked_package/functional/FR-054-admitted-scalar-operand-projection.md
  role: examined
  excerpt: A separate consumer test compiles agains
- id: FR-344-AC-1
  path: spec/checked_package/functional/FR-344-admit-or-refuse-the-adr-002-2-0-0-members.md
  role: examined
  excerpt: A `quire.checked-package/v2` document wh
- id: FR-344-AC-2
  path: spec/checked_package/functional/FR-344-admit-or-refuse-the-adr-002-2-0-0-members.md
  role: examined
  excerpt: A `relation`/`population` node whose `bo
- id: FR-344-AC-3
  path: spec/checked_package/functional/FR-344-admit-or-refuse-the-adr-002-2-0-0-members.md
  role: examined
  excerpt: A `quire.checked-package/v2` document ca
- id: FR-346-AC-1
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: A package holding one `correspondence`/`
- id: FR-346-AC-2
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: 'IR reading: a body missing `frames` (pat'
- id: FR-346-AC-3
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: A frame entry for (`ConfigVersion`, `att
- id: FR-346-AC-4
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: A node whose `node_id` is not the digest
- id: FR-346-AC-5
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: A `type` target naming a `relation`/`pop
- id: FR-346-AC-6
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: A `fields` name `ConfigVersion` does not
- id: FR-346-AC-7
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: A frame entry for (`ConfigVersion`, `noS
- id: FR-346-AC-8
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: Two entries that bind the object key `Co
- id: FR-346-AC-9
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: A package carrying a state-step defect a
- id: FR-346-AC-10
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: Changing one `rust_field` in an admitted
- id: FR-347-AC-1
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: Over a selected document declaring `Mone
- id: FR-347-AC-2
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: In the same document a call to `charge(m
- id: FR-347-AC-3
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: 'A call to `settle(t: TaxedMoney)` with a'
- id: FR-347-AC-4
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A parameter typed `Integer`, `Int[0, 9]`
- id: FR-347-AC-5
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: An argument typed at a `composite_type`/
- id: FR-347-AC-6
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A call with two record-typed parameters,
- id: FR-347-AC-7
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: The receiver of a call to `charge` typed
- id: FR-347-AC-8
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: Conformance of a `GiftTaxedMoney` argume
- id: FR-347-AC-9
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A `record.project` read of `occurred_at`
- id: FR-347-AC-10
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A `case` over a union whose arm bodies a
- id: FR-347-AC-11
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: 'A `union_value` whose member payload is '
- id: FR-347-AC-12
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A `function` node with declared `semanti
- id: FR-347-AC-13
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A `function.call` whose callee is a `ref
- id: FR-347-AC-14
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A callee function node whose `parameters
- id: NFR-006-M-1
  path: spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
  role: examined
  excerpt: 'Peak incremental resident memory in the '
- id: NFR-006-M-2
  path: spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
  role: examined
  excerpt: Concurrent extra owned copies of each pr
- id: NFR-006-M-3
  path: spec/checked_package/non-functional/NFR-006-v2-producer-memory.md
  role: examined
  excerpt: Canonical package bytes and `package_id`
- id: FR-018-AC-1
  path: spec/conformance/functional/FR-018-conformance-corpus.md
  role: examined
  excerpt: 'Running the corpus directory yields one '
- id: FR-018-AC-2
  path: spec/conformance/functional/FR-018-conformance-corpus.md
  role: examined
  excerpt: Mutation fixtures independently alter sc
- id: FR-018-AC-3
  path: spec/conformance/functional/FR-018-conformance-corpus.md
  role: examined
  excerpt: Raw package probes pin exact and one-pas
- id: FR-020-AC-1
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: A process test runs the published corpus
- id: FR-020-AC-2
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: Process fixtures pin exit 1 with all six
- id: FR-020-AC-3
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The published fixture schema accepts `"-
- id: FR-020-AC-4
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The published package schema accepts a r
- id: FR-020-AC-5
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The fixture input schema refuses as `inv
- id: FR-020-AC-6
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The fixture input schema passes the valu
- id: FR-020-AC-7
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The runner returns `invalid_wire_format`
- id: FR-020-AC-8
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The expectation of a decoder fixture tha
- id: FR-020-AC-9
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The same number as the decoder fixture s
- id: FR-020-AC-10
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: Every canonical file of the published co
- id: FR-020-AC-11
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: Every canonical file of the published co
- id: FR-020-AC-12
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: A published corpus run exits 0 and emits
- id: FR-020-AC-13
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The published fixture schema rejects `0`
- id: FR-020-AC-14
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: The published package schema rejects a r
- id: FR-020-AC-15
  path: spec/conformance/functional/FR-020-json-conformance-interface.md
  role: examined
  excerpt: 'A successful `document_json` expression '
- id: FR-011-AC-1
  path: spec/core/functional/FR-011-package-identity.md
  role: examined
  excerpt: A package round trip through the non-can
- id: FR-011-AC-2
  path: spec/core/functional/FR-011-package-identity.md
  role: examined
  excerpt: Incrementing one requirement revision ch
- id: FR-011-AC-3
  path: spec/core/functional/FR-011-package-identity.md
  role: examined
  excerpt: Empty or malformed package, source-docum
- id: FR-012-AC-1
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: A floating executable clause is rejected
- id: FR-012-AC-2
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: The recursive reference body returns eac
- id: FR-012-AC-3
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: Malformed, cross-package, stale-revision
- id: FR-012-AC-4
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: Every executable clause kind accepts onl
- id: FR-012-AC-5
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: The typed expression tree implements the
- id: FR-012-AC-6
  path: spec/core/functional/FR-012-anchors-clauses-dependencies.md
  role: examined
  excerpt: Source spans reject zero line/column pos
- id: FR-044-AC-1
  path: spec/core/functional/FR-044-typed-std001-code.md
  role: examined
  excerpt: '`Std001Code::new("kani_vacuous_proof")` '
- id: FR-044-AC-2
  path: spec/core/functional/FR-044-typed-std001-code.md
  role: examined
  excerpt: '`serde_json::to_string` of `Std001Code::'
- id: FR-044-AC-3
  path: spec/core/functional/FR-044-typed-std001-code.md
  role: examined
  excerpt: '`Std001Code::REGISTERED` equals, in orde'
- id: FR-044-AC-4
  path: spec/core/functional/FR-044-typed-std001-code.md
  role: examined
  excerpt: '`std001_code!("kani_corpus_identity_coll'
- id: FR-044-AC-5
  path: spec/core/functional/FR-044-typed-std001-code.md
  role: examined
  excerpt: '`Std001Code::from_static`, evaluated in '
- id: FR-045-AC-1
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: From a fixture whose inventories contain
- id: FR-045-AC-2
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: 'A proved production Kani result credits '
- id: FR-045-AC-3
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: An effective bound narrower than declare
- id: FR-045-AC-4
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: Requirement, module, function and obliga
- id: FR-045-AC-5
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: An FR-029 `unsupported` negotiation disp
- id: FR-045-AC-6
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: Repeating a baseline on unchanged author
- id: FR-055-AC-1
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: The normal `make deny` recipe invokes th
- id: FR-055-AC-2
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: Through the same normal `make deny` reci
- id: FR-055-AC-3
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: 'A missing checker executable, malformed '
- id: FR-055-AC-4
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: The committed `scripts/check_one_copy.aw
- id: FR-055-AC-5
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: Adoption leaves the committed Cargo mani
- id: NFR-001-AC-1
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: Two complete corpus runs over fixed inpu
- id: NFR-001-AC-2
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: For each fixed admitted corpus input and
- id: NFR-001-AC-3
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: For each fixed rejected corpus input and
- id: NFR-001-M-1
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: Cross-run byte equality
- id: NFR-001-M-2
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: Cross-platform canonical byte equality
- id: NFR-001-M-3
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: Ordered diagnostic code/path tuple equal
- id: NFR-002-AC-1
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Public schema and model names contain no
- id: NFR-002-AC-3
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Public JSON field names, identity kinds,
- id: NFR-002-AC-4
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Public type and expression schema/API vo
- id: NFR-002-M-1
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Forbidden public vocabulary
- id: NFR-002-M-2
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: examined
  excerpt: Target-dependent canonical fields
- id: NFR-003-AC-1
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Every declared invalid-input class retur
- id: NFR-003-AC-2
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: No malformed, unsupported, stale, undefi
- id: NFR-003-AC-3
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Given artifact traces whose source and t
- id: NFR-003-M-1
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Public panic paths for untrusted input
- id: NFR-003-M-2
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Failure classes with stable code
- id: NFR-003-M-3
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Coverage diagnostic locus accuracy
- id: NFR-003-M-4
  path: spec/core/non-functional/NFR-003-diagnostic-integrity.md
  role: examined
  excerpt: Orphans counted as covered
- id: FR-029-AC-1
  path: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  role: examined
  excerpt: A selected profile binds its versioned l
- id: FR-029-AC-2
  path: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  role: examined
  excerpt: 'Every encountered construct has exactly '
- id: FR-029-AC-3
  path: spec/kani/functional/FR-029-versioned-bounded-kani-profile.md
  role: examined
  excerpt: A scalar capability refuses an object, r
- id: FR-030-AC-1
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: Every model, population, snapshot, ident
- id: FR-030-AC-2
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: Invalid and incomplete populations, unav
- id: FR-030-AC-3
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: No invalid, refused, incomplete, unavail
- id: FR-030-AC-4
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: A `proved` outcome built from a run with
- id: FR-030-AC-5
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: An `unavailable` outcome built for an ab
- id: FR-030-AC-6
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: The `code` of the outcome built for a ze
- id: FR-030-AC-7
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: The IR outcome constructor accepts the o
- id: FR-030-AC-8
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: An external consumer calls each of the n
- id: FR-030-AC-9
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: External compile-fail probes cannot pass
- id: FR-031-AC-1
  path: spec/kani/functional/FR-031-bounded-kani-dispatch-and-terminal-map.md
  role: examined
  excerpt: The shared dispatch index routes defined
- id: FR-036-AC-1
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Generated symbolic ranges equal model-de
- id: FR-036-AC-2
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: 'Missing bounds, absent exact encodings, '
- id: FR-036-AC-3
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Every generated item keeps its precondit
- id: FR-036-AC-4
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: A provider completes accounting for ever
- id: FR-036-AC-5
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: A requested claim for which the provider
- id: FR-037-AC-6
  path: spec/kani/functional/FR-037-canonical-backend-replay-and-qualification.md
  role: examined
  excerpt: Contract IR's public API names no replay
- id: FR-039-AC-1
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: The root crate's public items are exactl
- id: FR-039-AC-2
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: No public item names a `quire_spec_langu
- id: FR-039-AC-3
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: Code naming any item the "Items QSL owns
- id: FR-039-AC-4
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: Every public `kani` function over untrus
- id: FR-013-AC-1
  path: spec/model/functional/FR-013-type-system.md
  role: examined
  excerpt: 'Every closed value-type and declaration '
- id: FR-013-AC-2
  path: spec/model/functional/FR-013-type-system.md
  role: examined
  excerpt: Public serialized types contain no Rust,
- id: FR-013-AC-3
  path: spec/model/functional/FR-013-type-system.md
  role: examined
  excerpt: Duplicate names/fields/variants, empty e
- id: FR-013-AC-4
  path: spec/model/functional/FR-013-type-system.md
  role: examined
  excerpt: A valid declaration environment round tr
- id: FR-013-AC-5
  path: spec/model/functional/FR-013-type-system.md
  role: examined
  excerpt: Integer bounds and literals at `i128::MI
- id: FR-013-AC-6
  path: spec/model/functional/FR-013-type-system.md
  role: examined
  excerpt: Public `IntegerType` bounds/accessors, `
- id: FR-014-AC-1
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: Ill-typed operands, result mismatches, i
- id: FR-014-AC-2
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: Canonical representation distinguishes s
- id: FR-014-AC-3
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: Current/pre/post observation policy, pur
- id: FR-014-AC-4
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: Rechecking identical declarations, conte
- id: FR-014-AC-5
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: Expression trees at the node/depth limit
- id: FR-014-AC-6
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: Typed expression fixtures derive exact i
- id: FR-014-AC-7
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: A guarded product of 120 leaves checks s
- id: FR-014-AC-8
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: With matching declared types, integer li
- id: FR-014-AC-9
  path: spec/model/functional/FR-014-expression-semantics.md
  role: examined
  excerpt: Inspection of every private checked-nume
- id: FR-015-AC-1
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: Every partial operation is accepted only
- id: FR-015-AC-2
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: Replacing a short-circuit guard with a t
- id: FR-015-AC-3
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: True/false facts from conjunction, disju
- id: FR-015-AC-4
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: Reject-overflow arithmetic uses constant
- id: FR-015-AC-5
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: Every `potentially_undefined` diagnostic
- id: FR-015-AC-6
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: Bottom-up range sets from literals, decl
- id: FR-015-AC-7
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: Positive and negative bounded-rational f
- id: FR-015-AC-8
  path: spec/model/functional/FR-015-definedness.md
  role: examined
  excerpt: At an `i128::MIN` endpoint, integer `rej
- id: FR-016-AC-1
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: 'Exact golden fixtures, regenerated from '
- id: FR-016-AC-2
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: A semantic change to a clause changes th
- id: FR-016-AC-3
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: A deterministic reservation-failure harn
- id: FR-016-AC-4
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: 'Each of the five object kinds exposes a '
- id: FR-016-AC-5
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: The eight integer members are JSON strin
- id: FR-016-AC-6
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: Every canonical byte sequence and digest
- id: FR-016-AC-7
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: The semantic value has no member that ca
- id: FR-016-AC-8
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: The digest for kind `K` equals SHA-256 o
- id: FR-016-AC-9
  path: spec/model/functional/FR-016-canonicalization-digests.md
  role: examined
  excerpt: Canonical bytes spell integer type bound
- id: FR-017-AC-1
  path: spec/model/functional/FR-017-version-orphan-coverage.md
  role: examined
  excerpt: Version preflight accepts schema 1.1; sc
- id: FR-017-AC-2
  path: spec/model/functional/FR-017-version-orphan-coverage.md
  role: examined
  excerpt: Shallow, deep, uncovered, and each close
- id: FR-017-AC-3
  path: spec/model/functional/FR-017-version-orphan-coverage.md
  role: examined
  excerpt: With package requirements and traces aut
- id: FR-017-AC-4
  path: spec/model/functional/FR-017-version-orphan-coverage.md
  role: examined
  excerpt: For the same current requirement, a vali
- id: FR-017-AC-5
  path: spec/model/functional/FR-017-version-orphan-coverage.md
  role: examined
  excerpt: Every repeated artifact ID produces exac
- id: FR-017-AC-6
  path: spec/model/functional/FR-017-version-orphan-coverage.md
  role: examined
  excerpt: For every artifact ID occurring more tha
- id: FR-019-AC-1
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: A TC-018 integration test imports `PUBLI
- id: FR-019-AC-3
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: '`expected_inventory` equals the five pre'
- id: FR-019-AC-4
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: A default-feature build of `quire-contra
- id: FR-019-AC-5
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: The `quire_contract_model` crate root ha
- id: FR-019-AC-6
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: The table's `CheckedArtifactRef` has exa
- id: FR-019-AC-2
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: The complete negative corpus executes pa
- id: FR-019-AC-7
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: Public compile-time/API fixtures constru
- id: FR-019-AC-8
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'Compile-time fixtures and public-source '
- id: FR-019-AC-9
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: An otherwise valid package or expression
- id: FR-019-AC-10
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'Public package and expression decoding, '
- id: FR-019-AC-11
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'Each public canonicalization entrypoint '
- id: FR-019-AC-12
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: A default-feature external API test impo
- id: FR-019-AC-13
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: Public-source signature inspection finds
- id: FR-023-AC-1
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: A labelled synthetic projection using sh
- id: FR-023-AC-2
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: Missing, duplicate, foreign, stale-revis
- id: FR-023-AC-3
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: Metadata/expression dependency disagreem
- id: FR-023-AC-4
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: Unknown format/members, trailing JSON, i
- id: FR-023-AC-5
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: An external integration test consumes pu
- id: FR-023-AC-6
  path: spec/model/functional/FR-023-executable-projection-binding.md
  role: examined
  excerpt: A projection whose package source revisi
- id: FR-028-AC-1
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: examined
  excerpt: Cargo metadata for every production feat
- id: FR-028-AC-2
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: examined
  excerpt: Schemas, canonical bytes, identities, di
- id: FR-028-AC-3
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: examined
  excerpt: The root package's own manifest names no
- id: FR-028-AC-4
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: examined
  excerpt: Default, all-feature and minimum-version
- id: FR-028-AC-5
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: examined
  excerpt: The split introduces no copied owner wir
- id: FR-046-AC-1
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: An external model-crate consumer derives
- id: FR-046-AC-2
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: An ordinary application produces the ind
- id: FR-046-AC-3
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: A group containing applications and nona
- id: FR-046-AC-4
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'Missing grouped context, absent target, '
- id: FR-046-AC-5
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: For an independently encoded preimage of
- id: FR-046-AC-6
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: After correct final-key/reference substi
- id: FR-048-AC-1
  path: spec/model/functional/FR-048-derive-structural-and-group-candidates.md
  role: examined
  excerpt: 'An external default-feature model-crate '
- id: FR-048-AC-2
  path: spec/model/functional/FR-048-derive-structural-and-group-candidates.md
  role: examined
  excerpt: Reader-admissible owner-free `reference`
- id: FR-048-AC-3
  path: spec/model/functional/FR-048-derive-structural-and-group-candidates.md
  role: examined
  excerpt: A reader-admissible group with structura
- id: FR-048-AC-4
  path: spec/model/functional/FR-048-derive-structural-and-group-candidates.md
  role: examined
  excerpt: Repeated selectors, an unknown tag/form,
- id: FR-048-AC-5
  path: spec/model/functional/FR-048-derive-structural-and-group-candidates.md
  role: examined
  excerpt: Exact canonical byte ceiling succeeds; o
- id: FR-052-AC-1
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: A default-feature consumer imports and c
- id: FR-052-AC-2
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: A typed fixture containing two Quire-aut
- id: FR-052-AC-3
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: A foreign source, each dangling endpoint
- id: FR-052-AC-4
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Inspection of the public constructor and
- id: FR-052-AC-5
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: An external default-feature consumer com
- id: FR-052-AC-6
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Omitting each of the four owner outcomes
- id: FR-052-AC-7
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Complete Quire scope covers every author
- id: FR-052-AC-9
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'For each of the four owners, the native '
- id: FR-052-AC-10
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: For each owner, stopping before normal e
- id: FR-052-AC-11
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: The neutral model retains an owner-issue
- id: FR-052-AC-12
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'A synthetic `SourceInventoryEnumerator` '
- id: FR-052-AC-13
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'For a synthetic enumerator, the neutral '
- id: FR-052-AC-14
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: A default-feature neutral consumer disti
- id: FR-052-AC-15
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: An explicit source-author binding connec
- id: FR-052-AC-16
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: With PLAT-1165's real Quire producer, no
- id: FR-052-AC-17
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given four synthetic `Complete` outcomes
- id: FR-052-AC-18
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'Given synthetic outcomes, two faults in '
- id: FR-052-AC-19
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'Given synthetic `Complete` outcomes, an '
- id: FR-052-AC-20
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given the FR-052-AC-17 inputs, permuting
- id: FR-052-AC-21
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'Given synthetic outcomes, an outcome in '
- id: FR-052-AC-22
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given synthetic `Complete` outcomes, two
- id: FR-052-AC-23
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'Given synthetic `Complete` outcomes, an '
- id: FR-052-AC-24
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Conflicting inventory facts (two `Semant
- id: FR-052-AC-25
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given an owner-issued IR V2 reader resul
- id: FR-052-AC-26
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given a selected QSL checked request, th
- id: FR-052-AC-27
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given a `quire-code-rs` selected product
- id: FR-053-AC-1
  path: spec/model/functional/FR-053-selected-population-binding.md
  role: examined
  excerpt: Through a default-feature public model c
- id: FR-053-AC-2
  path: spec/model/functional/FR-053-selected-population-binding.md
  role: examined
  excerpt: Two population nodes with the same selec
- id: FR-053-AC-3
  path: spec/model/functional/FR-053-selected-population-binding.md
  role: examined
  excerpt: An absent selector yields `Unavailable`;
- id: FR-053-AC-4
  path: spec/model/functional/FR-053-selected-population-binding.md
  role: examined
  excerpt: A selected domain-package PopulationReco
- id: FR-053-AC-5
  path: spec/model/functional/FR-053-selected-population-binding.md
  role: examined
  excerpt: A compiled default-feature consumer read
- id: FR-032-AC-1
  path: spec/output_mapping/functional/FR-032-admit-output-mapping-request.md
  role: examined
  excerpt: 'A valid request retains the exact bound '
- id: FR-032-AC-2
  path: spec/output_mapping/functional/FR-032-admit-output-mapping-request.md
  role: examined
  excerpt: Empty, duplicate, informational, missing
- id: FR-032-AC-3
  path: spec/output_mapping/functional/FR-032-admit-output-mapping-request.md
  role: examined
  excerpt: Target family, standard references, mapp
- id: FR-032-AC-4
  path: spec/output_mapping/functional/FR-032-admit-output-mapping-request.md
  role: examined
  excerpt: Path, timestamp, locale, display text, i
- id: FR-032-AC-5
  path: spec/output_mapping/functional/FR-032-admit-output-mapping-request.md
  role: examined
  excerpt: Every emitted refusal code has exactly o
- id: FR-032-AC-6
  path: spec/output_mapping/functional/FR-032-admit-output-mapping-request.md
  role: examined
  excerpt: A request whose canonical identity mater
- id: FR-033-AC-1
  path: spec/output_mapping/functional/FR-033-account-for-output-obligations.md
  role: examined
  excerpt: Every requested obligation is dispatched
- id: FR-033-AC-2
  path: spec/output_mapping/functional/FR-033-account-for-output-obligations.md
  role: examined
  excerpt: Preserved, conditional, unrepresented, a
- id: FR-033-AC-3
  path: spec/output_mapping/functional/FR-033-account-for-output-obligations.md
  role: examined
  excerpt: Mutating any source, dependency, profile
- id: FR-033-AC-4
  path: spec/output_mapping/functional/FR-033-account-for-output-obligations.md
  role: examined
  excerpt: Pending, incomplete, or refused source f
- id: FR-033-AC-5
  path: spec/output_mapping/functional/FR-033-account-for-output-obligations.md
  role: examined
  excerpt: Observation and protocol adequacy remain
- id: FR-033-AC-6
  path: spec/output_mapping/functional/FR-033-account-for-output-obligations.md
  role: examined
  excerpt: A mapper candidate whose record identity
- id: FR-034-AC-1
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: Equal admitted requests, mapper candidat
- id: FR-034-AC-2
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: Missing, duplicate, foreign, stale, reor
- id: FR-034-AC-3
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: Every zero, exact, just-over, and overfl
- id: FR-034-AC-4
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: Mutating source/profile/generator owner/
- id: FR-034-AC-5
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: Structural observer acceptance, refusal,
- id: FR-034-AC-6
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: Identity material is canonicalized under
- id: FR-034-AC-7
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: The record identity and the package iden
- id: FR-034-AC-8
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: In the written-out material each `Mappin
- id: FR-034-AC-9
  path: spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md
  role: examined
  excerpt: 'A request whose `maximum_emitted_bytes` '
- id: fixture_key
  path: tests/it/checked_package_v2_frame_entries.rs
  role: examined
  excerpt: /// FR-092/FR-094 keys computed here fro
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-002 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-003 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-004 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-005 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-006 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-007 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-008 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-009 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-010 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-011 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-012 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-013 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-014 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-015 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-016 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-017 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-018 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-019 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-020 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-021 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-022 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-023 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-024 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-025 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-026 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-027 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-028 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-029 | deferred | IR-641 owns consumer implementation after this spec-only prerequisite merge; the specification explicitly says PLANNED. This row remains untested and is not fixed or waived. |
| FND-030 | deferred | IR-641 owns consumer implementation after this spec-only prerequisite merge; the specification explicitly says PLANNED. This row remains untested and is not fixed or waived. |
| FND-031 | deferred | IR-641 owns consumer implementation after this spec-only prerequisite merge; the specification explicitly says PLANNED. This row remains untested and is not fixed or waived. |
| FND-032 | deferred | IR-641 owns consumer implementation after this spec-only prerequisite merge; the specification explicitly says PLANNED. This row remains untested and is not fixed or waived. |
| FND-033 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-034 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-035 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-036 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-037 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-038 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-039 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-040 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-041 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-042 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-043 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-044 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-045 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-046 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-047 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-048 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-049 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-050 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-051 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-052 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-053 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-054 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-055 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-056 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-057 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-058 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-059 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-060 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-061 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-062 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-063 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-064 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-065 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-066 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-067 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-068 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |
| FND-069 | deferred | Inherited exact-base program debt handed to existing IR-878 owner luna:IR:plan in live comment c235aa4e-50e1-4489-9477-f6915a9a6787; outside the specification-only consumer contract. Remains unresolved; no passing coverage claim. |

Stage disposition: all 69 findings are explicitly deferred to their actual owners. The full-gap verdict remains FAIL. The planned spec contract can merge without declaring consumer adoption implemented, without relaxing any gate, and without closing IR-641.

## Narrow Disposition (round 2)

Same-reviewer narrow disposition after review custody only: all 20 committed typed/raw review files match their source bytes; no product, specification, Makefile or dependency change exists relative to the reviewed candidate. Existing eight PASS method verdicts stand. The full-gap verdict remains FAIL and all 69 prior findings retain their explicit deferred outcomes. The normal final make-spec structural stage passed (1222/1222 documents, zero grammar findings), then strict coverage failed: exact base 40 inherited unbacked rows versus final 45, zero contradicted statuses in both, and no removed rows. The five added rows are exactly FR-055-AC-1/2/3/4/5; all belong to same-ticket IR-641 implementation/qualification after specification merge. AC4 is Inspection, not a fifth missing Test binding. The 40 inherited normal-gate rows were handed to the actual IR-878 owner in comment f2ed9a92-6141-4d8c-97d5-9554b825e83b. No failing gate, criterion, inspection result or implementation status is relabeled green.

No new finding was introduced by custody. No prior finding was reopened or changed; their latest explicit outcomes remain deferred. The empty round-2 dispositions list means there was no open finding requiring a new outcome, not that the full gap became clean.
