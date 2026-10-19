---
id: "SR-13231"
title: "IR PR 479 EARS conformance: FR-035-AC-12 to AC-14"
type: SpecReview
analysis: ears-conformance
review_set: subset
scope: "agent-ix/quire-contract-ir@f5229f01c640aa772fe7ba1c109a3e94dfe75b26; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md lines 163-167 (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13231: IR PR 479 EARS conformance

## Summary

Ticket: IR-893. PR: agent-ix/quire-contract-ir#479 at f5229f0.

Examined: FR-035-AC-12, FR-035-AC-13 and FR-035-AC-14, and the trailing note "FR-035-AC-12
through FR-035-AC-14 are planned until TC-047 tests carry their trace tags." For comparison,
the sibling criteria FR-035-AC-10 and AC-11 were read as context.

All three new criteria are event-driven ("When <trigger>, <system> SHALL <response>"). Each has
one named system (`ContractPackageWireV1::read`), one trigger and one observable response
(code, path and cause), plus a "SHALL admit no `ContractPackageV1`" clause. The no-admission
clause is a second response, but it restates the same `Refused` outcome, so the criteria are
not compound in a way a test could split. None uses a vague term.

`quire validate` (quire 0.36.2, engine 0.50.2) passes the three files with three new warnings,
`[ac:non-canonical-shape] criterion is obligation-shaped`, at FR-035 lines 163, 164 and 165.
On origin/main, FR-035 raises no such warning.

## Verdict

ACCEPTABLE WITH ONE LOW FINDING. The trigger and response grammar is sound. The precondition
defects in AC-12 and AC-14 are recorded once, in SR-13230, and are not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-035-AC-12, AC-13 and AC-14 are written as obligations ("SHALL return ... SHALL admit no"). The sibling reader criteria AC-10 and AC-11 use the direct-assertion form ("`read` returns `Refused` ... it returns no admitted package"). The obligation form adds three `ac:non-canonical-shape` warnings to `quire validate` that were not there before. Restate them in the AC-10 form | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:163-165 |
