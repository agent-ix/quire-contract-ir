---
id: REV-015
title: "Executable projection boundary review and measured corrections"
type: SpecReview
analysis: gap-analysis
scope: "Issue 50 public executable binding; synthetic shared-form consumer tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-023
    type: reviews
---

# REV-015: Executable binding boundary

## Summary

The coordinator selected an IR-owned keyed binder rather than a second codegen
wire model. Before implementation, the codegen consumer reviewed FR-023's API
requirements, complete full-ClauseRef population and dependency equality rule.
The public projection is derived interchange only: issue #52 still owns the
authoritative Markdown/model/formal-clause frontend. No hand-authored sidecar is
promoted to the normal source, and synthetic test fixtures do not prove frontend
coverage. The implementing coordinator records this account; the independent
reviewers' findings and tests are identified below, not impersonated as a human
release decision.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1501 | high | Independent adversarial tests reproduced process stack overflow on valid semantic-depth256 BooleanNot input inside the public binder, after fixture encoding. Initial Serde-only stack protection was insufficient. The decoder now reserves bounded same-thread recursive-work stack after byte/depth guards. Subprocess controls require depth256 success and257 structured refusal for expressions, types and reference bodies. Native qualification does not imply universal target or host-memory-exhaustion guarantees. | FR-023-AC-4; TC-035 | correct-requirement-no-evidence |
| FND-1502 | high | Independent code review found the aggregate digest relied on serde_json Map ordering, which downstream preserve_order feature unification changes. Reuse of the existing explicit lexical canonical writer removes that dependency. An independently assembled exact envelope hash is exercised under feature unification. Existing canonical kinds and source exclusions remain unchanged. | FR-023-AC-3; TC-035 | correct-requirement-no-evidence |
| FND-1503 | high | Independent raw-byte consumer reproduced duplicate package id accepted through Value normalization while the existing direct package decoder refused it. A recursive unique-member visitor now rejects duplicates before normalization; raw-byte tests cover nested identity, expression and provenance objects as well as the envelope. No free-text diagnostic or verdict parsing is introduced. | FR-023-AC-4; TC-035 | wrong-requirement |
| FND-1504 | medium | Initial document validation found the new FR lacked Dependencies and the new test summary omitted the required status marker. Corrected without changing criterion expectations. Inherited matrix schema/coverage-field disagreement remains a shared module/pin integration gate, not a reason to reverse the corrected Status header. | FR-023; TC-035; Quoin issue350 | correct-requirement-no-evidence |
| FND-1505 | medium | The initial test comments named criteria but did not use the established bound doc-comment form. Structured Quire output showed unmatched tags. Exact TC-035 and per-test criterion doc comments now bind all five FR-023 criteria with no unmatched FR-023 tags. This is a static census, not a test-execution verdict. | FR-023; TC-035 | correct-requirement-no-evidence |
| FND-1506 | medium | Integration replay found the separate matrix census did not recognize the 13 binder tests' tc035 names. Renaming them to the existing tc_035 convention, including the depth subprocess's exact test selector, fixes the declared-symbol join without changing test inputs, expectations, doc-comment bindings, or the fail-closed census. | NFR-004-AC-5; TC-035 | correct-requirement-no-evidence |

## Verification

The initial implementation passed four public consumer tests, all36 Rust tests
(including all99 unchanged conformance rows), Clippy and Rust1.75 all-target
checking. These preceded the adversarial fixes and are not final-head claims.
The expanded13-test binder suite subsequently passed on stable and Rust1.75.0
with serde_json/preserve_order, including recursive and escaped-equivalent
duplicate-member controls and eight isolated depth subprocesses. A full44-test
Rust run also passed after the semantic fixes but before the thirteenth test.
These are precommit runs; exact committed-head gates follow separately.

Tests are synthetic adaptations of explicitly included shared corpus forms.
Dependency-positive and mismatch controls, exact five-executable/one-information
population, informational-only and empty populations, declaration/expression
digest changes, and independent normative schema mutations exercise the public
boundary. No private wire imports, codegen-owned package format, local assurance
framework, attestation or tool-verdict synthesis is used.

## Remaining gates

### Integration checkpoint: 2026-09-06

The independent final binder review and committed-head qualification of
`93674480c572c237fe87c5d509b17206664bdd62` were completed, and the real public
codegen consumer is published in codegen PR #27 at
`cd345e1dc0199db9abeac9955fd1bfcc121cddc9`. Those bounded gates are no longer
pending; frontend, campaign provenance and shared release qualification remain.

The historical FND-1504 header conflict is now resolved by the shared
reference-specific status-column declaration: process
`e6ea5151b59a55d7ce0d43f1581cbe276f750e04`, ISO
`a60ee12d735976081849f60a38d603fb5494b015`, CLI
`977a32af8737f8b0111cf2220528145c3dbfe318` and engine
`11969b707382203fe8df1c92e8a2fb2fa7e0bbbc`. IR recovery commit
`8d359e00e40f69500bbc97984f43e3cbdd214eb7` changes only the two functional
coverage headers to Coverage Status; binder integration merge `c88d660`
preserves that exact delta. The global Status vocabulary is unchanged.
Explicitly loading just the process and ISO module directories validates both
matrices: 2/2 grammar-clean, zero grammar findings. Intrinsic module duplicate
declaration advisories remain; this is not full-repository or release validation.

Before the naming correction, the unchanged matrix census refused TC-035 as
complete without a declared executable test. Afterward the census and its three
Python regression tests pass. The corrected tree passes all 45 Rust tests with
serde_json/preserve_order on stable and all 13 binder tests with that feature on
Rust 1.75.0, including the eight isolated depth subprocesses. Formatting and
diff checks pass. These are tested-tree results before this documentation commit;
no assertion, fixture, semantic library, schema or canonical expectation changed.

The following list records the original dispatch gates; the first two have the
bounded dispositions above, while the shared and frontend gates remain open.

- Complete independent final review and committed-head stable/MSRV/
  feature-unified tests; precommit adversarial controls have passed.
- Qualify the actual codegen consumer and full package identity in source maps.
- Resolve shared Quire/Quoin module schemas, source merges and exact pins before
  the full release-check; no human review/protection bypass is authorized.
- Complete #52 frontend/model lowering and native campaign/provenance joins;
  this binder alone is not a spec-to-contract executable campaign.
