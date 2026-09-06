---
id: REV-014
title: "PR 51 campaign recovery and remaining integration gates"
type: SpecReview
analysis: gap-analysis
scope: "PR 51 findings against base 9b9102c; local recovery candidate"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/NFR-004
    type: reviews
---

# REV-014: PR 51 campaign recovery

## Summary

This is the implementing coordinator's measured account, not an independent
review or a human release decision. Base: `9b9102c3806e9cda0ed70312f4f6c23a211f6fbf`.
The acceptance authority remains the current owning criteria and the independent
review on PR 51; this account does not replace them.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1401 | high | Implemented native Make scheduling/failure controls across every declared lane, with omitted-dependency, global ignore, recipe-prefix and swallowed-exit mutations. Doubles test orchestration only, never domain verdicts. | IR51-01; NFR-004-AC-6 | correct-requirement-no-evidence |
| FND-1402 | high | Open integration prerequisite: the installed Quire strict gate exits zero despite structured status-column/denominator findings. Corrected authored functional matrix headers; CLI owner must implement and test strict policy. No console-parsing workaround. | IR51-02; quire-cli FR-017 | correct-requirement-no-evidence |
| FND-1403 | high | Published manifest schema now directly validates the real instance and rejects independent missing/empty/unknown/operation/digest/trace mutations. The runner already compiled and used the named manifest subschema at this base; that was not a missing execution path. | IR51-03; TC-018 | correct-requirement-no-evidence |
| FND-1404 | high | Authored complete coverage-token-to-criterion registry replaces operation-wide TC aliases. Runner checks exact registry inventory and per-fixture target union before executing; misbound, omitted and extra valid criterion IDs fail. Corpus inputs, expectations and canonical bytes are unchanged. | IR51-04; FR-018 | wrong-requirement |
| FND-1405 | medium | Record-ordering guards raise ChainError under normal and optimized Python; subprocess tests prove no Quoin invocation precedes sealing. | IR51-05; FR-022-AC-6 | correct-requirement-no-evidence |
| FND-1406 | medium | Removed false AC-5 tag from the Rust plan test. Python tests retain ownership. Installed old engine misclassifies unittest methods as non-binding; current-engine CLI integration remains pending. | IR51-06; NFR-004-AC-5 | wrong-requirement |
| FND-1407 | medium | Suite and record command use the declared Integration evidence kind, not the unsupported Conformance token; nonexistent planning glob removed and both required chain inputs listed. Interface consistency is tested. | IR51-07; TC-021 | correct-requirement-no-evidence |
| FND-1408 | medium | Norm explicitly excludes shape-invalid wire-depth probes from semantic-success boundaries. Manual hosted workflow is named/documented as an unqualified subset with provisioning limitations. No dispatch. | IR51-08; IR51-09 | wrong-requirement |
| FND-1409 | low | Help no longer claims a removed census. Independent literal-depth mixed object/array controls include escaped strings with 1200 delimiters and distinguish document from document.nesting. Manifest fixtures require minItems 1. | IR51-10; IR51-11; IR51-12 | correct-requirement-no-evidence |
| FND-1410 | low | Matrix census remains declared-symbol evidence only. Native tools execute the tests; neither this census nor the trace registry claims complete criterion proof or release sufficiency. | IR51-13 | wrong-requirement |
| FND-1411 | high | Independent review found operation-insensitive ownership: package reference bodies claimed typed-expression criteria and artifact diagnostics claimed semantic-reference criteria. Followup qualifies registry entries by operation and tests both false attributions plus healthy expression/package/coverage controls. | FR-018; independent campaign review | wrong-requirement |
| FND-1412 | high | Initial recovery commit 5e01fbd omitted the tested runner source from staging. Independent exact-commit inspection caught this; b3f8f1b includes it. Acceptance must use the complete followup chain, not the first commit or a dirty-tree test claim. | PR 51 recovery | correct-requirement-no-evidence |
| FND-1413 | medium | Full exact-module validation found six unsupported authored metadata declarations: policy was not an archetype and five reviews used an undeclared analysis category. The governance document now uses the existing Standard type and a stable code; reviews use spec-correctness. Two interface FRs drop undeclared optional object classifiers. All eight document bodies, IDs, criteria and review findings remain unchanged. No module vocabulary was widened. | NFR-004; canonical specification validation | correct-requirement-no-evidence |
| FND-1414 | medium | Independent review of c521aaa found spec-correctness is defined by process FR-002 for implementation-against-spec review, not these pre-implementation specification reviews. Followup uses base for SR-009's architecture review and gap-analysis for SR-010 through SR-013's specification findings/dispositions. This corrects FND-1413's initial classification without changing any original review body. | process FR-002; independent metadata review | wrong-requirement |
| FND-1415 | medium | Banked before repair: seven FR-011 through FR-017 matrix rows spell their status as bare implemented, outside the exact process module's declared emoji-prefix vocabulary. Their cited TC-015/016/017 tests are implemented and execute successfully. Correct only those status cells to the existing declared implemented spelling; do not change planned TC-019, NFR qualification statements, methods, criteria or expectations. | NFR-004-AC-5; functional-coverage | correct-requirement-no-evidence |

## Verification

The following ran in the isolated recovery worktree with
`CARGO_TARGET_DIR=/tmp/contract-core-ir-target`:

- Locked offline complete Rust suite with ignored tests included: 32 tests pass.
- Conformance corpus: 99 matching rows; two process runs byte-identical.
- Corpus scratch regeneration: byte-identical; only schema/manifest trace metadata
  and their sidecars changed in the checked-in corpus, not oracle expectations.
- All-target locked offline Clippy with warnings denied: pass.
- Normal/optimized Python ordering test: pass; three native orchestration tests:
  pass; three matrix tests and real-tree status census: pass.
- Quire document validation before this review: 109/109 grammar-clean.
  First attempt rejected the unsupported Conformance kind; corrected the command
  to the existing Integration vocabulary instead of widening the module.

Installed Quire is 0.31.0 with the older engine, and reports duplicated default
modules. Its zero strict exit is not acceptance: Python criteria AC-5/6 are
unbacked under that engine and structured diagnostics remain. Current exact module
selection, CLI engine update, Quoin trace-preservation integration, pinned shared
assurance, MSRV, independent candidate review and full release-check remain gates.
Nothing here claims that a corpus fixture proves all conjuncts of its criterion.

Independent review of b3f8f1b found no other blocker within the requested schema,
trace, Python ordering and Make-control scope. The reviewer reran the focused
schema/mismatch Rust tests and Python ordering/orchestration tests. After the
operation-qualified followup, the full five-test conformance suite passes again;
independent followup review and campaign-wide gates remain pending.

### Exact-module integration checkpoint: 2026-09-06

CLI `ff638b9802178aa62c757aab914cf0288c1cbe67` with engine
`d3bc2baff191c9521f1064480a56c8dc0bd1c7fa` is independently reviewed and its
canonical CI passes. Its explicit module roots for this checkpoint are process
`e6ea5151b59a55d7ce0d43f1581cbe276f750e04`, ISO
`a60ee12d735976081849f60a38d603fb5494b015`, and released engineering-assurance
`8ea16ce240934aa2c31c1cc3f781b7eb0f8c73ba`. Root arguments name each module's
actual manifest directory, not a parent checkout or installed default set.

The full binder-tree validation before FND-1413's correction reported six
structural failures and two unknown-object advisories. Inspection found six
real unsupported authored declarations out of six structural findings, not
six bad rules. Corrected metadata in this recovery tree validates all110
documents with zero structural or grammar failures. The two unknown-object
advisories are gone; intrinsic duplicate-module-declaration advisories remain.
Twelve native foundation/governance/reconciliation tests and the completed
matrix census pass; no test expectation or normative body changed.

The binder tree before this metadata correction separately passed strict
coverage with102/109 rows backed, zero unbacked-row findings and zero status
lies. That is not an all-criteria verification claim: seven bare implemented
statuses are undeclared, fifteen diagnostics and twelve unmatched tags remain,
including uncatalogued NFR measurement methods. These residual findings are
retained for separate authored-method/trace review, not suppressed or counted
as proof. Shared assurance acceptance/release gates remain open.

### Authored coverage-status bank

At recovery `c3b481309b9da51aa1cd82a0a252d24ff14c7683`, the same exact
CLI/engine and three module roots above report seven undeclared statuses for
FR-011 through FR-017. Native coverage with `--strict --severity
coverage:undeclared-status=error` exits 1; no finding is filtered. The existing
process vocabulary owns `complete: ["✅"]`, and its functional table grammar
requires an emoji prefix. The adjacent implemented rows already use that form.
All fourteen native identity, expression and canonicalization tests backing
TC-015/016/017 pass with locked offline Cargo. The proposed cell-only correction
recognizes existing evidence; it does not prove complete criterion semantics or
cross-platform qualification. Remaining NFR-method and source-tag diagnostics
stay visible and outside this repair.
