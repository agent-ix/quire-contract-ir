---
id: SR-12031
title: "IR-836 NFR-003 tests gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/core/matrix/tests.md, spec/tests.md, spec/core/non-functional/NFR-001-determinism.md, spec/core/non-functional/NFR-002-portability-independence.md, spec/core/non-functional/NFR-003-diagnostic-integrity.md, spec/core/functional/STD-001-diagnostic-registry.md, tests/it/nfr003_invalid_input.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/NFR-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/NFR-002
    type: references
  - target: ix://agent-ix/quire-contract-ir/STD-001
    type: references
---

## Summary

Ticket: IR-836, PR quire-contract-ir#428. This analysis has no plan: it reads `quire coverage --scope. --strict` and `quire trace`. Plan completion: not assessed. Measurements were taken with quire 0.36.2 (engine 0.50.2) and quoin 0.28.3:

| Tree | Strict unbacked |
| --- | --- |
| (PR merge base) | 43 |
| (origin/main now, with #426) | 45 |
| | 39 |
| head merged into (no-commit merge in a throwaway worktree) | 39 |

The PR body says the drop from 43 to 39 is NFR-003-AC-1/2 and TC-019. That is not what was measured. On the four rows that leave the list are `TC-019`, `NFR-001-AC-2`, `NFR-001-AC-3` and `NFR-002-AC-1`. NFR-003-AC-1/2 were not on that list then, because the legacy doc lines that #426 removed were still crediting them. On the merged tree, six rows leave the list compared with main: NFR-003-AC-1, NFR-003-AC-2, TC-019, NFR-001-AC-2, NFR-001-AC-3 and NFR-002-AC-1.

`quire validate --scope.` over the three changed or affected spec files is clean (only the module-level advisory). `quire trace --id TC-019` resolves three `rust-trace-attribute` claims, one per test, plus one file-scope citation.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| NFR-003-AC-1 | examined | Every declared invalid-input class returns a stable diagnostic code and no public panic. |
| NFR-003-AC-2 | examined | No malformed, unsupported, stale, undefined, or orphaned input contributes successful canonical identity or covered status. |
| TC-019 | examined | Determinism, portability, and fail-closed metrics meet thresholds \| Integration \| P0 \| NFR-001..NFR-003 |
| NFR-001-AC-2 | examined | For each fixed admitted corpus input and supported profile that yields canonical output, Linux, macOS, and Windows produce byte-identical canonical bytes when cross-platform CI is enabled; compare those bytes directly. |
| NFR-001-AC-3 | examined | For each fixed rejected corpus input and profile, repeated runs and supported operating systems produce exactly the same complete ordered sequence of diagnostic `(code, path)` tuples. |
| NFR-002-AC-1 | examined | Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary. |
| NFR-003 | context_only | Scope: Parsing, validation, dependency derivation, canonicalization, coverage classification, and the conformance runner. |
| STD-001 | context_only | This registry owns the stable machine-readable diagnostic codes of FR-011 through FR-019, the bounded-Kani cause and outcome error codes FR-030 raises and the code-form code FR-044 raises. |
| FR-020 | context_only | Exit 2 emits... closed code (`invalid_invocation`, `invalid_corpus`, `unsupported_profile`, `unsafe_path`, `fixture_io`, or `resource_exhausted`), and stable path; prose detail is non-comparable. Panics, partial JSON, and mixed stdout/stderr records are forbidden. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The PR retypes TC-019 as Integration and tags it on tests that cover only NFR-003. As a result, the strict coverage gate stops reporting three acceptance criteria that no test backs: NFR-001-AC-2 (cross-platform canonical bytes), NFR-001-AC-3 (ordered `(code, path)` tuples across runs and OSes) and NFR-002-AC-1 (schema/model vocabulary). Each of them has the Verification cell "Test (TC-019)". quire treats them as backed once TC-019 has a symbol, yet `quire trace --id NFR-001-AC-2` on the merged tree returns 0 claims. The TC-019 row's own status text says those parts "have no executable test". So the gate passes because the TC is too coarse, not because the criteria are met. The PR body's coverage claim (43 to 39 is "NFR-003-AC-1/2 and TC-019") hides this. Fix in this PR: give NFR-003-AC-1/2 their own TC (Integration, NFR-003), point their Verification cells and the tests' tags at it, and leave TC-019 for NFR-001/NFR-002 with its planned status. Then strict coverage on main goes from 45 to 43, with exactly NFR-003-AC-1/2 leaving the list. | spec/core/matrix/tests.md:47, tests/it/nfr003_invalid_input.rs:307, tests/it/nfr003_invalid_input.rs:337, tests/it/nfr003_invalid_input.rs:421, spec/core/non-functional/NFR-001-determinism.md:36-37, spec/core/non-functional/NFR-002-portability-independence.md:34 |
| FND-002 | medium | NFR-003-AC-1 does not say where its "declared invalid-input classes" are declared. The PR's reading is the 44 `DiagnosticCode` codes of STD-001's four semantic tables. The NFR-003 metric "Diagnostic registry/corpus reconciliation" and the FR-011..FR-019 ownership sentence in STD-001 support that reading. The reading itself is recorded only in a test doc comment, the matrix status cell and the PR body, and the exclusions are recorded only in the PR body. One exclusion is not stated anywhere: the six runner operational codes of FR-020 (`invalid_invocation`... `resource_exhausted`). These are closed, stable codes of "the conformance runner", which NFR-003's own Scope names, and FR-020 forbids panics for them. The exclusion of `invalid_code_form` (`Std001Code::new` validates an untrusted candidate string, and "validation" is in scope) is argued only in the PR body. Excluding the Kani codes (FR-030 outcomes, outside NFR-003's scope) and `kani_outcome_invalid` (root-crate constructor) is supported by the text. Fix: state the class set and its exclusions in NFR-003 (or the core matrix row), or include the runner codes with TC-018 as their evidence. | spec/core/non-functional/NFR-003-diagnostic-integrity.md:36, spec/core/functional/STD-001-diagnostic-registry.md:24-28, spec/conformance/functional/FR-020-json-conformance-interface.md:68-75, tests/it/nfr003_invalid_input.rs:5-6 |

## Verdict

CHANGES REQUESTED (one high, one medium). Not mergeable until FND-001 is fixed.

Clean units:

- **NFR-003-AC-1 and NFR-003-AC-2 are genuinely backed** for the class set the PR chose (see SR-12030 for the mutant evidence and the one AC-2 subset gap). On the merged tree, both leave the unbacked list for the right reason.
- **Retyping TC-019 from Analysis to Integration is consistent with the vocabulary.** The `spec-artifacts-process` `test_type` vocabulary has no plain `Test`. `Integration` is a Test-class harness kind, and every criterion that cites TC-019 says "Test (TC-019)". `Analysis` is in `no_source_symbol`, so on main TC-019 was reported both as having no backing symbol and as "verified by `Analysis`". The defect is the TC's breadth (FND-001), not the type word.
- **Status text.** The NFR-003 row, the TC-019 row and the spec/tests.md Core row say only that AC-1/AC-2 are implemented, and keep NFR-001/NFR-002 and AC-3 as planned. That text is accurate. Once FND-001 is fixed, the new TC row should carry it.
- **Trace.** The three `#[trace]` attributes and `/// Trace:` lines name only TC-019 and NFR-003-AC-1/2. No other test claims NFR-003-AC-1/2 (`quire trace`).
- **Fixtures.** Every fixture is from the repo's own `corpus/contract-v0.1`. Nothing is copied, and no corpus file changes.
- **Not run.** `quoin:spec-review` and its sub-analyses were not run, as the brief directed. The spec diff is three status/type cells, and they are judged here.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1. NFR-003-AC-1/2 now cite "Test (TC-454)". The new row `TC-454 \| Every declared invalid-input class fails closed... \| Integration \| P0 \| NFR-003` is added, and the three tests tag `TC-454` with NFR-003-AC-1/2. TC-019 is back to `Analysis`, traces NFR-001 and NFR-002 only, and is planned. TC-454 is unused on origin/main (whose highest TC is TC-453) and in open PR #427 (which uses only TC-048). Measured with `quire coverage --scope. --strict`: origin/main has 45 unbacked rows and has 43. The only rows that leave the list are NFR-003-AC-1 and NFR-003-AC-2. NFR-001-AC-2, NFR-001-AC-3, NFR-002-AC-1 and TC-019 (no backing symbol, verified by Analysis) are listed again. `quire trace --id TC-454` shows 3 claims, and `--id TC-019` shows 0. |
| FND-002 | fixed | Round 1. NFR-003's Scope now states the class set: the `DiagnosticCode` values of STD-001's four tables, from "Identity and Reference Codes" through "Semantic Limit Codes". It also states three exclusions, each with a reason and its evidence: the `kani_*` codes including `kani_outcome_invalid` (FR-030 outcomes, not rejected input); `invalid_code_form` (FR-044, asserted by TC-442); and the six FR-020 runner codes (asserted by TC-018). The runner exclusion quotes STD-001 accurately ("neither `DiagnosticCode` values nor semantic diagnostic shapes", STD-001 Dependencies) and rests on NFR-003's statement about rejected semantic conditions. I accept these exclusions. Each is stated and backed by tagged evidence elsewhere, and the runner's no-panic obligation stays with TC-018. |
| FND-003 | fixed | Round 2. The model matrix TC-017 row now reads "direct assertions for FR-017-AC-2 through AC-4, FR-017-AC-6 and NFR-003-AC-3 remain planned (NFR-003-AC-1 and AC-2 are backed by TC-454)". The core TC-019 row is now titled "Determinism and portability metrics meet thresholds", traces NFR-001 and NFR-002, and has the status "planned:... and portability analysis have no executable test; NFR-003 is TC-454's". `git grep` finds no remaining "TC-019 has no test" or "fail-closed threshold" text under spec/. quire validate is clean. The change is status/title text only. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | Two status cells are stale now that TC-454 owns NFR-003-AC-1/2. The model matrix TC-017 row (on main since #426) still says "NFR-003-AC-1 through AC-3 remain planned (IR-836 tracks NFR-003; TC-019 has no test)". The core TC-019 row, which now traces only NFR-001/NFR-002, still lists "fail-closed threshold analysis" among its planned work and keeps "fail-closed" in its title. Neither cell affects coverage. Fix: in the TC-017 row, say NFR-003-AC-1/2 are TC-454's and only AC-3 is planned, and drop the fail-closed part from TC-019. | spec/model/matrix/tests.md:29, spec/core/matrix/tests.md:47 |
