---
id: SR-16624
title: "IR-641 PR 553 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@beb1975f6b297edc8c8eabf3044b551aa3d55e34; PR #553 Makefile and lock coherence integration tests; FR-055-AC-1 through AC-5"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-055", type: references }
---
# SR-16624: IR-641 PR 553 incremental gap analysis

## Summary

Planless incremental requirement, trace, test, and source analysis for the FR-055 consumer change. Ticket: IR-641.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-055-AC-1 examined: tests/it/lock_coherence.rs:90-100 and 130-149 tag AC-1; the aligned and committed-lock cases exercise the normal Make recipe and assert success plus the command invocation echo.
- FR-055-AC-2 examined: the first tagged test has separate duplicate-name and split-revision controls and requires the owner's coherence diagnostic. The valid resolved fragments prevent malformed-source refusal from masquerading as a coherence result.
- FR-055-AC-3 examined: the same tagged test asserts nonzero exits for malformed lock, checker refusal, and missing checker.
- FR-055-AC-4 examined by source inspection: the copied checker is deleted and the deny recipe calls only the owner command after cargo-deny; no new local parser appears.
- FR-055-AC-5 examined: the second tagged test invokes the owner checker against the committed lock and compares pre/post manifest and lock bytes. The PR diff itself has no manifest or lock change. The released-command final gate has not run, so this is coverage evidence, not final qualification.
- Reverse gap: the new Rust test helpers serve only the FR-055 controls; the Makefile change is owned by FR-055. No stub, vacuous assertion, or unowned new production behavior was found in this diff.
- Plan completion: not assessed. Full repository matrix and inherited gaps were outside this incremental PR review; no repository-wide PASS is asserted.

## Verdict

PASS for incremental FR-055 trace and test causality at the draft head. Released checker installation and final-head gate are outstanding.

## Continuation at 35d44745fbc157b8f8426ca9bcc0c19f23bec808

The workflow addition installs the authoritative Quire source at a full revision in both hosted jobs and directs the normal test and deny recipes to the installed binary. This preserves the owner algorithm, but the installed artifact is compiled from an unreleased source commit. FR-055-AC-1, the Description, Qualification Status, and Consumer Verification Plan explicitly require a **released** command/binary. The installed fleet `quire` still lacks `check-lock`; the source installation therefore cannot satisfy that qualification condition. No acceptance criterion has been edited. The PR remains draft pending released-binary qualification or a first-hand owner decision that changes the written requirement.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | Pinned source install does not satisfy FR-055-AC-1's released-binary requirement | .github/workflows/ci.yml:29 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | The workflow installs an unreleased source revision and the fleet released Quire CLI does not provide check-lock. |

## Continuation verdict

FAIL for FR-055-AC-1 qualification at this head. The source install and consumer wiring may be functionally correct, but the released-binary requirement remains unmet.
