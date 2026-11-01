---
id: SR-202610310131
title: "IR-1228 PR 779 external compensation fairness spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #779; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed FR-038-AC-383 and AC-384 against the unchanged FR-038 normative fairness text and the current temporal and protocol reader joins. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**FAIL** — the new admission criterion contradicts the existing resolution rule, and its adverse criterion does not protect an orphan fairness node.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The existing rule says, "Protocol targets resolve through the protocol clause's retained `requirements` relation and the same membership used by protocol controls." AC-383 instead admits a compensation target with empty `Pay.requirements`. Amend that rule to state the compensation-only owner join and preserve requirement association for the other protocol targets; reconcile AC-103's matching-clause wording as needed. Otherwise two authoritative FR-038 passages specify opposite outcomes for the same package. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1962; FR-038-AC-383 |
| FND-002 | medium | AC-384 covers an unowned compensation target reached from an external temporal clause, but it does not require refusal of a standalone fairness node that points to an owned compensation template and is referenced by no admitted temporal clause. The later protocol sweep currently refuses that orphan. Add an explicit adverse assertion so the new owner-based path cannot admit it accidentally. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4536; FR-038-AC-384 |

## Coverage

- FR-038-AC-383, role: examined. An external temporal clause targets a unique compensation template even with empty protocol requirements.
- FR-038-AC-384, role: examined. Unowned, foreign, wrong-form and requirement-linked foreign target refusals.
- FR-038 fairness member resolution text and AC-103, role: context_only. Existing requirement relation and typed refusal order.

## Evidence

At the frozen head, `protocol_body.rs::validate_temporal_fairness` scans only protocol clause `requirements` to find the temporal node. `validate_protocol_fairness` also marks only fairness nodes reached through those requirements and refuses any other non-operation target. These two checks explain the existing refusal, while `validate_protocol_fairness_target` already checks retained clause ownership and target form. The proposed exception must be reachable from an admitted external temporal clause and cannot authorize an orphan fairness node solely because its target has a retained owner.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head 6187f1c1 narrows the retained requirements rule to protocol-root, attempt and parallel-branch targets, and states the compensation-template owner join only for an admitted external temporal clause. |
| FND-002 | fixed | Head 6187f1c1 adds a specific missing-name refusal for a fairness node absent from every admitted temporal clause's fairness argument, even with a uniquely owned compensation target. |

## Disposition verdict

**PASS** — both findings are fixed in the same PR; no new inconsistency appears in the changed text.
