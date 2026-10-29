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
  - target: ix://agent-ix/quire-mltl/FR-004
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
   `mapping_support_v2` and its retained package/declaration/profile selection (FR-350-AC-1).
   Project a future-only event-position formula under complete and open
   surrounding execution. Require the corresponding closed-trace and
   online-prefix formulas, trace requests and owner-reader admissions (FR-350-AC-1).
   Supply the admitted native subject identity and the fixed
   `quire.native-temporal-correspondence/v1` profile identity to quire-mltl
   `RequestInput`; owner-read the generated request before hashing its canonical
   bytes into `target.request`, then derive the record reference. Recompute the
   request with a different correspondence profile identity and require an
   owner-reader mismatch; substitute the record reference for that profile
   identity, and separately corrupt only `target.request`, and require refusal
   before an admitted projection. The request identity must be independent of
   the record reference; the record reference must cover the request digest.
2. Under each false-extension profile, project `once[0,2] p`,
   `historically[0,2] p`, `p since[0,2] q` and `p triggered[0,2] q`
   individually with an execution-origin history. Also project strong previous
   as `once[1,1] p`. Require formula-v2, the past-operator catalog,
   position-history and origin-complete profile selection, admitted by each
   owner's public reader. Repeat the `[0,2]` cases with an anchor at the first
   retained position and a history cutoff with no proof of the two prior
   positions; require `incomplete` with `missing-history`. Combine a past and
   future operator and require `unsupported` with no formula, request or
   correspondence record (FR-350-AC-1, FR-350-AC-2).
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
   with their exact public readers (FR-350-AC-3, FR-350-AC-4).
6. Join native `evaluate_v2` and TL mapped results through every decision row,
   including equal-final, equal-pending, unavailable, unsupported, incomplete,
   failed, refused and conflict. Remove each preserved result axis, exercise
   correction predecessors and supply each owner-reader refusal (FR-350-AC-5, FR-350-AC-6).
   For every pair matching two non-final rows, require the first row's decision
   and both causes once in table order; include refused plus failed, unavailable
   plus unsupported and unsupported plus incomplete pairs. Reverse the order in
   which the two owner results are supplied and require the same decision and
   cause order. A value/pending pair matching no row refuses as
   `result-mismatch`; an owner-read refusal cannot be reinterpreted as an
   equal-final value.
7. Run every configured limit exactly at its setting and one unit over. Also
   test missing, duplicate and foreign valuation cells and noncanonical
   projection/join encodings (FR-350-AC-6).
   For each of the seven limit fields, inspect its published default, then use
   a smaller positive caller setting and construct one case whose named
   counter equals that setting and another whose next charge makes it
   `setting + 1`. Use one canonical record of exact byte length for
   `max_bytes`; formula nodes for `max_nodes`; positions for `max_positions`;
   cells for `max_valuations`; compared support entries for `max_support`;
   emitted ordered causes for `max_causes`; and combined node, position, cell
   and support visits for `max_work`. Keep all other settings above their
   observed counts, then force two limits to cross on one action and require
   the first limit in FR-350's table. Check that the rejected unit was not
   visited, the returned `value` is `setting + 1`, and no partial record exists.
8. Encode the canonical correspondence record from FR-350-AC-7 and require its
   specified 1429-byte encoding. Independently hash that exact line, then
   recompute after changing only interval `[[0, 2]]`, model package to 64 `a`s
   and bridge definition to 64 `b`s, one at a time. Each mutation remains
   1429 bytes, has a distinct digest, and its correspondence reference equals
   the digest derived from its own bytes. Reordered or whitespace-padded records
   refuse. These byte-level controls do not substitute for step 1's owner-reader
   evidence (FR-350-AC-7, FR-350-AC-8).

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
- The canonical record line and each single-member interval, model-package
  and bridge-definition mutation are 1429 bytes. Their independently derived
  digests differ pairwise, and each correspondence reference matches the digest
  of its own canonical bytes. No recorded digest supplies expected identity.
- Malformed or over-limit inputs produce one deterministic refusal or
  incomplete result with no partial artifact and no evaluator invocation by
  Contract IR. Exact limits admit; one past yields `limit-reached` naming the
  limit, its value and setting.
