---
id: SR-13543
title: "IR-881 PR #485 spec-review-integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/spec.md; spec/tests.md; spec/core/matrix/tests.md; spec/core/matrix/TC-456-public-name-portability.md"
review_set: subset
---

## Summary

Ticket: IR-881. Checked consistency of the new implemented-credit claims against the requirement and matrix indexes. Three summary rows still say credit is withheld for the retired allowance.

## Scope

Examined: NFR-002-AC-1; NFR-002-AC-3; TC-456. PR diff...

## Examined units

- NFR-002-AC-1 (examined, spec/core/non-functional/NFR-002-portability-independence.md): "Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary."
- NFR-002-AC-3 (examined, spec/core/non-functional/NFR-002-portability-independence.md): "Public JSON field names, identity kinds, anchors, clause kinds, dependency kinds, and diagnostic codes contain no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary."
- TC-456 (examined, spec/core/matrix/TC-456-public-name-portability.md): "The registered Kani checker-absence code uses `kani_checker_absent` and has no exception to this rule."
- TM-root (examined, spec/spec.md): "NFR-002-AC-1 credit is withheld until IR-881 removes the temporary registered-code allowance."
- TM-009 (examined, spec/tests.md): "NFR-002-AC-1 and NFR-002-AC-3 credit is withheld until IR-881 removes the temporary registered-code entry."
- TM-core (examined, spec/core/matrix/tests.md): "NFR-002-AC-1 credit is withheld, blocked by IR-881."

## Verdict

**CONDITIONAL** — Checked consistency of the new implemented-credit claims against the requirement and matrix indexes. Three summary rows still say credit is withheld for the retired allowance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The requirement and matrix indexes still state IR-881 blocks NFR-002-AC-1/AC-3 credit after TC-456 and the edited NFR-002 row claim both implemented; update spec/spec.md:64, spec/tests.md:12, and spec/core/matrix/tests.md:13. | spec/spec.md:64; spec/tests.md:12; spec/core/matrix/tests.md:13 |

## Coverage

Plan completion: not assessed. Review limited to PR #485's changed obligations and associated tests.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The replacement summaries claim TC-456 backs NFR-002-AC-3, but TC-456 carries only NFR-002-AC-1; TC-015 is the tagged AC-3 test. Name TC-015 for AC-3 in all three summaries. | spec/spec.md:65; spec/tests.md:12; spec/core/matrix/tests.md:13 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed |, all three stale credit-withheld claims are replaced with implemented-credit statements; the exact new spec/spec.md sentence reads: "The registered-code allowance is removed, and TC-456 backs NFR-002-AC-1 and NFR-002-AC-3." |
| FND-002 | still-open | The new statement assigns AC-3 to TC-456. tests/it/public_interface.rs traces TC-456 to AC-1 only; tests/it/identity.rs traces TC-015 to AC-3, matching spec/core/matrix/tests.md:44,48. |
| FND-002 | fixed |, all three summaries now attribute NFR-002-AC-1 to TC-456 and NFR-002-AC-3 to TC-015, matching the test trace tags and matrix rows. |
