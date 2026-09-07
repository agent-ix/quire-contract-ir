---
id: REV-008
title: "Corpus depth and native-runner remediation preimplementation review"
type: SpecReview
analysis: code-review
scope: "issues #30, #31, and #36; wire-depth boundary observability; standalone corpus orchestration"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: reviews
---

# REV-008: Corpus depth and native-runner remediation preimplementation review

## Summary

Issues #30, #31, and #36 describe two false-green classes in one domain lane. The 576- and
577-level package probes had byte-identical expectations, so removing the pre-decode nesting guard
left the corpus green. Separately, the Make target piped the native runner through deletable Python
`assert` statements and discarded the runner's own exit and diagnostics.

## Verdict

**PASS to implement a bounded domain correction.** The Make target should invoke the native runner
directly. It must not grow another wrapper, generic verifier, `pipefail` self-test, or Makefile trust
guard. The wire boundary should remain one diagnostic code but give pre-decode depth refusal a
stable path distinct from ordinary package-shape decoding.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-801 | high | The at-limit and over-limit fixtures had the same input class, diagnostic projection, expectation digest, and coverage predicate. Required control: exact limit reaches ordinary decoding at `document`; one-past is refused pre-decode at `document.nesting`; disabling or moving the guard makes the corpus red. | issue #30; FR-018; TC-018 | correct-requirement-no-evidence |
| FND-802 | high | The standalone corpus target used Python `assert`, which disappears under optimization. Required control: remove the Python census and invoke the native runner directly so `PYTHONOPTIMIZE` is irrelevant. | issue #31; Makefile | correct-requirement-no-evidence |
| FND-803 | medium | The pipeline replaced the runner's structured mismatch/error output with an unrelated `AssertionError`. Required control: preserve the native runner's stdout/stderr directly. | issue #36; Makefile | correct-requirement-no-evidence |
| FND-804 | medium | A semantically valid package cannot reach wire depth 576 because stricter semantic depth limits apply first to every recognized recursive body. Do not weaken semantic limits to manufacture a passing fixture; distinguish normal shape decoding from pre-decode depth refusal instead. | FR-018; FR-019; FR-020 | wrong-requirement |

Hosted CI remains manual-only and is outside this remediation.
