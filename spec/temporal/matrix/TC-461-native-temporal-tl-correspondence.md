---
id: TC-461
title: "Project native temporal subjects and join owner results through correspondence"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-045
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-050
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-043
    type: references
  - target: ix://agent-ix/quire-specification/FR-095
    type: references
  - target: ix://agent-ix/quire-specification/FR-092
    type: references
---
# TC-461: Project native temporal subjects and join owner results through correspondence

## Description

Verify the Contract IR temporal subsystem against QSL's admitted classification,
strict `/2` package and `evaluate_v2` result, the TL owners' public readers and
QSpec FR-095's correspondence meaning. The bounded-past operator set is defined
once by QSpec FR-092; strong previous is `once[1,1]`.

## Test Procedure

1. For a strict `/2` package, obtain a `Supported` classification through QSL
   `mapping_support_v2` and its retained package/declaration/profile selection.
   Project a future-only event-position formula under complete and open
   surrounding execution. Require the corresponding closed-trace and
   online-prefix formulas, trace requests and owner-reader admissions.
2. Project each bounded past operator defined by QSpec FR-092 alone under both
   false-extension profiles, using an execution-origin history. Require the
   formula-v2, past-operator catalog, position-history and origin-complete
   profile selections and admission by each selected owner's public reader.
   Repeat each operator with a history-cutoff boundary and require `incomplete`
   with `missing-history`. Combine a past and future operator and require
   `unsupported` with no formula, request or correspondence record.
3. For every required position and proposition, provide one explicit Boolean
   valuation cell bound to the package's subject, anchor, capture and
   observation revision. Exercise false as an explicit cell, then remove,
   duplicate, swap, replay and foreign-bind a cell.
4. Project a fixed-sample subject with the exact authenticated epoch, period
   and unit; change only the period while retaining equal formula bytes, then
   omit a required sample.
5. Mutate each preserved correspondence key axis independently, including the
   source span, predicate/type/model binding, anchor, capture, clock,
   observation, interval, closure/history, TL selection and bridge definition.
   Recompute the bridge-owned reference and read the generated TL artifacts
   with their exact public readers.
6. Join native `evaluate_v2` and TL mapped results through every decision row,
   including equal-final, equal-pending, unavailable, unsupported, incomplete,
   failed, refused and conflict. Remove each preserved result axis, exercise
   correction predecessors and supply each owner-reader refusal.
7. Run every configured limit exactly at its setting and one unit over. Also
   test missing, duplicate and foreign valuation cells and noncanonical
   projection/join encodings.

## Expected Results

- Each bounded past operator individually projects only with origin-complete
  history; each cutoff case is incomplete with `missing-history`; mixed
  past/future is unsupported and emits no substitute artifact.
- Generated formula, history/trace, request, valuation and correspondence
  records are admitted by their owners and preserve distinct identities.
- A changed preserved axis changes the correspondence reference. A changed
  period refuses; an absent sample is incomplete.
- Only an equal final pair yields `agreement` with a Boolean. Every other
  decision has no Boolean and preserves its ordered causes, progress/closure,
  settlement, support, completeness and supersession axes.
- Malformed or over-limit inputs produce one deterministic refusal or
  incomplete result with no partial artifact and no evaluator invocation by
  Contract IR. Exact limits admit.
