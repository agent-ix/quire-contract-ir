---
id: NFR-004
title: "Preserve evidence and human release boundaries"
type: NFR
quality_attribute: compliance
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: traces_to
---
# NFR-004: Preserve evidence and human release boundaries

## Statement

Every implementation child shall retain requirement-tagged tests, review and
gap findings, exact tool/input/output identities, inconclusive or skipped
results, and explicit limitations. Registry publication remains disabled.

## Scope

Issues #5, #6, #8, #9, #10, epic #11, repository evidence, and source-release
workflow.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Requirements with planned verification | 100% | Any unplanned criterion blocks review | Strict traceability coverage |
| Open blocking review findings at merge | 0 | Any open blocker prevents merge | Composite review and gap analysis |
| Registry publication setting | `publish = false` | Any other value fails | Cargo manifest inspection |
| Automated source-release decisions | 0 | Any automated approval fails | Evidence and assurance-argument review |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-004-AC-1 | Cargo publication is disabled, both license texts are present, and CI has no automatic trigger. | Test (TC-014) |
| NFR-004-AC-2 | The five assurance artifacts name decision, system, component, measurement, failure, evidence, and human-owner boundaries. | Test (TC-020) |
| NFR-004-AC-3 | The implementation plan keeps dependency-blocked children pending and records every composite-review dimension. | Test (TC-021) |

### Retired criteria

`NFR-004-AC-4` required a contradicted retained review claim to be append-only
corrected to `inconclusive` without rewriting the original record. Its subject
was `evidence/corrections/COR-001-pr12-code-review.json` and the retained record
it corrected. Both are deleted under the pre-stable release of the
evidence-preservation constraint decided by the repository owner on 2026-09-02
([engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7)).
The correction and the claim it superseded went together, so the criterion is
retired rather than reassigned to a surviving artifact: nothing else in this
repository is a contradicted retained review claim. The identifier is not
reused.

## Verification

Foundation inspection and named human sufficiency decision
(TC-014, TC-020, TC-021).
