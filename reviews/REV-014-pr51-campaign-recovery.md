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

## Scope

This is the implementing coordinator's measured account, not an independent
review or a human release decision. Base: `9b9102c3806e9cda0ed70312f4f6c23a211f6fbf`.
The acceptance authority remains the current owning criteria and the independent
review on PR 51; this account does not replace them.

## Findings

| ID | Severity | Disposition | Refs | Escape Cause |
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
