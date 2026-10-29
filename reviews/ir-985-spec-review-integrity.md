---
id: SR-16267
title: "Integrity review of IR-985 PR 527"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; FR-038-AC-311..317, TC-048 hybrid procedure, FR-019 HybridForm inventory; QSpec FR-481/TC-966"
review_set: subset
---
# SR-16267: Integrity review of IR-985 PR 527

## Summary

Ticket: IR-985. Exact IR; QSpec #415. FR-038-AC-313 groups HY-11 and the schema-invalid HY-18 relation case with model checks 1–4. QSpec assigns those refusals to earlier name and strict-wire stages.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-313 assigns HY-11 and HY-18 strict-wire case to model checks 1–4, contrary to QSpec reader order | FR-038-AC-313; QSpec FR-481:249-255; TC-966:57,64 |

## Scope

- FR-038-AC-311 through FR-038-AC-317, examined: all seven new reader admission, identity, order and fixture criteria.
- TC-048 hybrid procedure, examined: published Braking, Braking2, Tank and HY vector assertions.
- FR-019 HybridForm inventory, examined: planned public enum under checked_package vocabulary.
- QSpec FR-481 and TC-966 at #415 head, context only: authoritative node forms, reader order and vectors.

## Verdict

Changes requested on FR-038-AC-313: separate HY-11 name validation and HY-18 malformed-wire relation from the model-check 1–4 claim. The valid HY-18 reset-conflict case remains within model checks.

## Dispositions

Round 1, reviewed against IR PR #527 exact.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-313 now assigns HY-11 to earlier name validation and HY-18 `eq` relation to strict wire validation; only the rebuilt-key reset-conflict case stays under model checks 1–4. |

## Upstream dependency

At QSpec #415, FR-481 Reader order checks 3 and 4 require `ill_typed`/`non-boolean-root`. The published `proposals/checked-package-v2/schema.json` `Diagnostic.cause_tag` enum and `DiagnosticCausePairing` omit `non-boolean-root`. This is an upstream QSpec schema mismatch, not an IR normative wording change. IR-985's QSpec fixture verification depends on QSpec resolving it.
