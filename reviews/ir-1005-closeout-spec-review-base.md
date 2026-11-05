---
id: SR-2026110414560022
title: "spec-review/base of IR-1005 matrix correction"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/temporal/matrix/tests.md; IR-1005"
review_set: base
---

## Summary

PASS for the matrix correction only; IR-1005 remains open. All seven live criteria are Planned/unbacked; AC-7 stays withdrawn. This partial delivery does not satisfy full ticket acceptance.

## Coverage

- `FR-350` (examined, `spec/temporal/matrix/tests.md`): | FR-350 | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 | TC-461 | 🚧 Planned in `agent-ix/quire-integration` (IR-1005). AC-1 through AC-6 and AC-8 remain unbacked until the QSL-typed bridge and executable correspondence tests carry their criterion trace tags there; AC-7 is withdrawn and has no TC-461 coverage. FR-028-AC-3/TC-041 continue to prohibit QSL dependencies in both Contract IR packages. |
- `TC-461` (examined, `spec/temporal/matrix/tests.md`): | TC-461 | Project native temporal subjects and join owner results through correspondence | Integration | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 | 🚧 Planned in `agent-ix/quire-integration` (IR-1005); covers QSL classification, admitted checked-package/v2 inputs, every bounded past operator, TL owner readers, native/TL joins, mutations and limits. |
- `FR-350-AC-1` (examined, `spec/temporal/matrix/tests.md`): | FR-350-AC-1 | Planned; unbacked | Implement `project_tl`, formula/history/trace/request construction and owner-reader admission for every supported row, including each bounded past operator under both false-extension profiles. Test origin-complete history and the bridge's `incomplete`/`missing-history` cutoff decision. The existing instance-binding controls cover only the pre-projection portion of TC-461 step 1. |
- `FR-350-AC-2` (examined, `spec/temporal/matrix/tests.md`): | FR-350-AC-2 | Planned; unbacked | Implement unsupported projection decisions preserving every classified dimension and emitting no partial artifacts; test timestamped, mixed, infinite-trace and finite-window equivalence refusals. |
- `FR-350-AC-3` (examined, `spec/temporal/matrix/tests.md`): | FR-350-AC-3 | Planned; unbacked | Implement the correspondence value and projection reader; test every preserved typed-member mutation and non-Boolean leaf refusal. Classification binding alone does not construct or verify these members. |
- `FR-350-AC-4` (examined, `spec/temporal/matrix/tests.md`): | FR-350-AC-4 | Planned; unbacked | Implement fixed-sample projection; test retained epoch/period/unit, changed-period and event-count refusals, and missing-sample incompleteness. |
- `FR-350-AC-5` (examined, `spec/temporal/matrix/tests.md`): | FR-350-AC-5 | Planned; unbacked | Implement `join_tl` and `read_tl_join`; test every decision row, ordered causes, all result axes, equal-final Boolean restriction and exact direct correction predecessors. |
- `FR-350-AC-6` (examined, `spec/temporal/matrix/tests.md`): | FR-350-AC-6 | Planned; unbacked | Implement valuation admission, projection/join readers and deterministic refusal/incomplete paths; test missing, duplicate, swapped, replayed and foreign cells, owner-reader refusals, malformed/noncanonical data, exact and exceeded limits, no partial artifacts and no evaluator invocation. |
- `FR-350-AC-8` (examined, `spec/temporal/matrix/tests.md`): | FR-350-AC-8 | Planned; unbacked | Implement all seven `TlLimits` counters, defaults and pre-work charging; test each exact/one-over boundary and simultaneous-crossing priority with the required named cause and no partial artifact. |
- `FR-350:input-binding-evidence1` (examined, `spec/temporal/matrix/tests.md`): The instance-binding control in `agent-ix/quire-integration`
`tests/temporal_mapping_binding.rs` exercises the checked-view input boundary:
foreign but equal admitted instances, altered disposition, retained definition
authority and surrounding closure. It is partial TC-461 evidence only. The
projection, TL reader, native-result join and limit controls above remain Planned;
this input check does not confer whole-criterion coverage.
- `FR-350:criterion-dispositions1` (examined, `spec/temporal/matrix/tests.md`): Every live FR-350 criterion remains **Planned and unbacked**. The implementation
and executable TC-461 tests belong to `agent-ix/quire-integration`, under
IR-1005. The following are the missing bridge controls, not coverage supplied by
the input-binding test or by owner-contract documentation.
- `FR-350:owner-contracts-and-remaining-work1` (examined, `spec/temporal/matrix/tests.md`): QSL FR-515/FR-527 supply the checked subject and authenticated classification;
FR-045 defines support and FR-043 supplies native evaluation inputs/results.
QSL's checked-view mapping supports bounded-past classification. This does not
discharge the bridge's outstanding history premise or construct TL artifacts.
QSpec FR-092/FR-095 supply bounded-past semantics and correspondence meaning;
the selected TL owners supply their formula, history/trace, request and result
readers as listed in FR-350. These available contracts are dependencies to
consume, not evidence that the bridge is implemented or
- `FR-350:owner-contracts-and-remaining-work2` (examined, `spec/temporal/matrix/tests.md`): IR-1271 completed the FR-350/TC-461 owner and consumed-interface correction; it
did not implement the bridge. P-TL-9's formula/evaluation work does not own the
cross-owner bridge. IR-1005 retains the remaining implementation and test work
in `agent-ix/quire-integration`; Contract IR retains this normative contract and
matrix, with no QSL dependency added to either package.
- `FR-350-AC-1` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Given an admitted checked-package/v2 and QSL `Supported` classification, the projection takes its target from the classification, emits the required formula and valuation request for each supported row, and each generated artifact is admitted by its selected owner reader. Each bounded past operator defined by QSpec FR-092 is projected alone under both false-extension profiles with origin-complete history; a history cutoff yields `incomplete`/`missing-history`.
- `FR-350-AC-2` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Timestamped-event, mixed past-and-future, infinite-trace unbounded and other unsupported classifications return `unsupported` with every named dimension and no formula, request or correspondence record; a finite-window-to-closed-trace equivalence claim is refused.
- `FR-350-AC-3` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Mutating any preserved source/span, package/declaration/profile, model/type/predicate binding, anchor, capture, clock, observation, interval, closure/history, TL selection or bridge-definition axis changes the corresponding exposed typed member and identifies that axis; a non-Boolean leaf never becomes a proposition.
- `FR-350-AC-4` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Fixed-sample projection retains epoch, period and unit, refuses a changed period or event-count reinterpretation with equal formula bytes, and reports a missing required sample as `incomplete`.
- `FR-350-AC-5` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Every join decision row is reached; only an equal-final pair carries a Boolean; every non-final pair preserves its causes and all result axes, and a correction is accepted only with its exact direct predecessor value.
- `FR-350-AC-6` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Missing, duplicate, swapped, replayed or foreign valuation cells; owner-reader rejection; malformed or noncanonical projection/join data; and each one-over resource limit return one deterministic non-admitted/incomplete result, with no partial artifact and no evaluator invocation. Exact limits admit.
- `FR-350-AC-7` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Withdrawn by the IR-1141 no-hash decision under QSpec #517/#518: the fixed 1429-byte vector, SHA-256 known answers and derived correspondence reference are removed. This id is retained and never reused.
- `FR-350-AC-8` (context_only, `spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md`): Each `TlLimits` setting uses its stated default, charges before work, admits exactly at the setting and returns `incomplete` one past it with `limit-reached{limit, value, setting}` and no partial artifact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

Read FR-350 and TC-461 unchanged, the requested IR-1005 comment 9c5ad53f, and the owner decision comment 953f0ac3. Independently inspected integration src/temporal.rs and tests/temporal_mapping_binding.rs on origin/main: validate_mapping_binding checks instance membership and full reclassification equality only. The test exercises real producer/strict-reader binding, but constructs no TL artifact. A source/test symbol search found no project_tl, read_tl_projection, join_tl, read_tl_join or TlLimits. Thus no complete AC-1 through AC-6/AC-8 can be backed by that test. The diff fixes both stale IR-358 references, retains original Test verification and exact partial-evidence boundary, and does not invent an upstream blocker. IR-1271 corrects contracts; P-TL-9 does not own this bridge. No production change, copied owner definitions, pin, or coverage inflation was introduced. No applicable AssuranceProfile exists under spec/. No Cargo, Kani, build, test, census or measurement run was performed; git diff --check passed.

## Verdict

PASS for the matrix correction only; IR-1005 remains open. All seven live criteria are Planned/unbacked; AC-7 stays withdrawn. This partial delivery does not satisfy full ticket acceptance.

Code-review and base spec-review apply. Rust, production gap analysis and additional spec lenses do not apply: only matrix ownership and evidence dispositions changed; no requirement, domain object, relationship edge or implementation changed. The base review checks row IDs, links, consistency, boundary/error coverage descriptions and truthful test ownership without treating planned cases as executable coverage.
