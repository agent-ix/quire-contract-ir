---
id: SR-612086
title: "Failure-domain review of IR-1005 PR #643"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md; spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md"
review_set: subset
---

## Summary

Independent review of draft PR #643 against main. Ticket: IR-1005. The request dependency graph is acyclic and refuses a wrong profile identity or request digest; owner read failures become typed refused states.

## Verdict

**PASS** — The request dependency graph is acyclic and refuses a wrong profile identity or request digest; owner read failures become typed refused states.

## Scope

- FR-350-AC-1 (examined): Given a strict `/2` package and QSL `Supported` classification, the projection takes its target from the classification, emits the required formula and valuation request for each supported row, and each generated artifact is admitted by its selected owner reader. Each bounded past operator defined by QSpec FR-092 is projected alone under both false-extension profiles with origin-complete history; a history cutoff yields `incomplete`/`missing-history`.
- FR-350-AC-5 (examined): Every join decision row is reached; only an equal-final pair carries a Boolean; every non-final pair preserves its causes and all result axes, and a correction is accepted only with its exact direct predecessor.
- FR-350-AC-6 (examined): Missing, duplicate, swapped, replayed or foreign valuation cells; owner-reader rejection; malformed or noncanonical projection/join data; and each one-over resource limit return one deterministic non-admitted/incomplete result, with no partial artifact and no evaluator invocation. Exact limits admit.
- FR-350-AC-7 (examined): The canonical correspondence record matches the complete field inventory and fixed 1429-byte vector specified in the Wire section; interval, model binding and bridge-definition mutations match the stated digests; reordered or whitespace-altered bytes are refused.
- FR-350-AC-8 (examined): Each `TlLimits` setting uses its stated default, charges before work, admits exactly at the setting and returns `incomplete` one past it with `limit-reached{limit, value, setting}` and no partial artifact.
- TC-461 (examined): Steps 1 and 6–8, including request identity, ordered join causes, seven limit boundaries and fixed byte vector.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
