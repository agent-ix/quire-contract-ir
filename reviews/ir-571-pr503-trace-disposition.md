---
id: SR-15365
title: "IR-571 PR 503 trace correction disposition"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #503..; three test trace tags and one QSpec context comment"
review_set: subset
---

## Summary

Narrow independent review of the trace correction on PR #503. The original code-review and gap-analysis findings in SR-15326 and SR-15327 remain fixed.

## Examined units

- `crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs`: two tests now cite local `TC-225` and `FR-346-AC-7`. Their assertions require name refusal, repeated-identity precedence, parameter path, and retained operation origin at selected-document intake.
- `tests/it/checked_package_v2_abstraction_relation.rs`: the public reader test now cites the same local rows. It checks an admitted control, a case-distinct control, eleven adverse cases including name/reference and collision/reference precedence, and the selection-row code, cause, path, and locus. The QSpec identifier remains explanatory prose only.
- `spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md` AC-7 explicitly requires malformed parameter names and repeated identities to refuse at domain selection before the frame check. `spec/checked_package/matrix/TC-225-checked-package-v2-abstraction-relation-body.md` names the corresponding cases.
- `.github/workflows` has no change in this correction or the PR diff. `git diff --check` passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS** — The tag correction cites two declared local rows that cover the tested behavior. The computed `quire matrix --format json` status for `FR-346-AC-7` is `tagged` and binds all three corrected tests. Its diagnostics contain no unmatched citations. The former six `FR-154-AC-14`/`TC-280` citations no longer appear as `#[trace]` tags in the PR head. No production behavior changed from the previously reviewed head.

Focused `quire coverage --scope. --strict` exits 1 on the inherited 48 unbacked rows and zero contradicted statuses; the 48-row set is exactly equal to the previous gate log, while unmatched citations fell from six to zero. The complete output is in the adjacent `coverage.err` log. This review does not claim a full `make ci` rerun.
