---
id: "SR-13240"
title: "IR PR 480 spec review: cumulative ContractPackage reader work and occurrence stop pointers"
type: SpecReview
analysis: base
review_set: subset
scope: "agent-ix/quire-contract-ir PR #480; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13240: IR PR 480 spec review

## Summary

Ticket: IR-889. PR: agent-ix/quire-contract-ir#480 at the reviewed PR head. Spec-only (three Markdown files).

Renumbering note: after IR PR #479 (IR-893) merged with its own FR-035-AC-12 to AC-14, this
PR's criteria were renumbered. The AC-12 and AC-13 named in this review, including its findings
and dispositions, are now FR-035-AC-15 and FR-035-AC-16. This review records the ids as they
stood when it was written.

The PR records decision (a): one cumulative `CheckedPackageReadLimits.work` bound across the V1
reader's value validation and reference validation. It also moves the occurrence-limit stop
pointer from `/source_map` to the charged occurrence or region. It amends the FR-035 statement,
FR-035-AC-11 and TC-047, and adds FR-035-AC-12 and FR-035-AC-13.

Evidence for (a) over per-pass (b), read from the code:

- `CheckedPackageReadLimits.work` is documented as "Maximum semantic-term validation visits"
  (shared.rs:27). That is one read-level ceiling, with no per-pass wording.
- `CheckedPackageIncomplete.consumed` is the "Counter value at the failed charge", and `path` is
  "The RFC 6901 pointer of the value whose charge failed" (shared.rs:461-466).
- The V2 reader threads one `WorkMeter` through lock, graph and diagnostics validation
  (v2/mod.rs:1009-1025).
- The V2 source-map stop names the entry or region (common.rs:745-782), not the container.
- FR-035-AC-9 already requires the adapter to "continue the existing FR-038 meter without reset".
- The V1 reader today sums body work locally (contract_package.rs:1630-1673). It then starts a
  fresh `WorkMeter::new(limits.work)` in `validate_references` (line 2045). The occurrence stop
  reports `/source_map` (line 1750).

The evidence supports (a).

I reproduced the current behaviour on `mixed_fixture` (9864 bytes, 3 nodes, 2 edges) with a
throwaway probe:

- Value pass: per-node charges of 1, 1 and 5, so 7 in total. Limits 0, 1 and 2..6 stop at
  `/semantic_graph/nodes/{0,1,2}/body` with consumed 1, 2 and 7.
- Reference pass: limits 7..19 stop there with a fresh meter, consumed limit+1. Limit 7 stops at
  `/semantic_graph/nodes/2/node_id`. The reference pass totals 20 units.
- Limit 20 admits after 27 units.
- Occurrences: 9 occurrences and 9 regions, 18 in total. Every stop reports `/source_map`, with
  consumed rising in steps of 2.

I then built a throwaway prototype of the cumulative rule: `validate_values` returns its work,
and the reference meter is pre-charged with it. The occurrence and region charges were changed
to one unit each, in order. Results:

- Limits 0..6: unchanged.
- Limit 7: consumed 8 at `/semantic_graph/nodes/0/node_id`, the first reference charge.
- Limit 20, which today admits: consumed 21 at `/semantic_graph/nodes/2/node_id`.
- Limit 26: consumed 27 at `/semantic_graph/nodes/2/body/members/1/value/type`.
- Limit 27: admits.
- Occurrence limit 17: consumed 18 at `/source_map/refusals/1/occurrences/0/regions/0`.
- Occurrence limit 0: consumed 1 at `/source_map/nodes/0/occurrences/0`.

AC-12 and AC-13 hold under that prototype. Limits 20..26 fit each pass separately but not their
sum, so the fixture can exercise AC-12's "fits each pass" clause. The spec states no concrete
example numbers, so there are no wrong example numbers. The PR body states the behaviour change
honestly: cumulative `consumed`, new stop points, packages that used to be admitted now return
`Incomplete`, and bytes and admitted values are unchanged. It adds no compatibility layer.
FR-035-AC-11's amendment does not weaken it.

Examined: the FR-035 statement paragraph on read limits, FR-035-AC-11, AC-12 and AC-13, the
TC-047 Description, Test Procedure, Expected Results and Status, and the tests.md FR-035 and
TC-047 rows. FR-035-AC-9 and AC-10 were read for context.

Skipped: spec-criterion-strength-analysis, because no Jev client is available. The ability of
AC-12 and AC-13 to fail was judged by hand. A per-pass meter admits at limit 20, and a
container pointer fails AC-13, so both can fail.

Informational, not a finding: the concurrent IR PR #479 (IR-893) also adds FR-035-AC-12 and
FR-035-AC-13, with different text, so the ids collide. The lead merges #479 first and then
renumbers this PR. Once renumbered, AC-11's "follow AC-12 and AC-13" cross-reference and the
TC-047 and tests.md ranges must move with it.

## Verdict

CHANGES REQUESTED: two medium findings. Decision (a) is well supported, and the new criteria are
correct under a prototype of the rule. The remaining gaps are under-specification:

- The spec does not fix the occurrence charge unit or order. That decides `consumed` and the
  pointer at any limit other than total-1.
- AC-12 asks for an "independently counted" reference-pass total that no spec text defines.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The amended FR-035-AC-11 requires "the failing source-map value" and the "consumed amount at the failed charge" for any occurrence limit. Neither AC-11, AC-13 nor the statement fixes the charge unit or order. They do not say whether each occurrence and each region is one unit, or whether an occurrence is charged before its regions. They also do not give the part order (nodes, requested_items, refusals). The V2 precedent charges an entry, then all its regions in bulk, and reports the post-bulk count with the first region over (common.rs:767-782). A V1 port of that rule reports a different `consumed` than per-region charging for multi-region occurrences. AC-13 at total-1 is determinate, because it always names the last region. TC-047's extra "cases under nodes and requested_items or refusals" have no derivable expected pointer or count. The fix is to state the unit and order. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:148-151,175,177; crates/quire-contract-model/src/checked_package/common.rs:745-782 |
| FND-002 | medium | FR-035-AC-12, and TC-047 after it, takes as its oracle "the independently counted total visits across both passes". The spec never defines a reference-validation visit. Those charges come from `walk_body`, `validate_application_keys`, `validate_derived_keys`, `validate_nominal_nodes` and `validate_recursion` (contract_package.rs:2134-2227). A test cannot count them independently of the implementation, and a total read from the reader's own admission threshold makes the "equals" clause tautological. The fix is to pin measured `mixed_fixture` values in TC-047: value 7, reference 20, total 27. Limit 26 gives consumed 27 at `/semantic_graph/nodes/2/body/members/1/value/type`. Limit 20 gives consumed 21 at `/semantic_graph/nodes/2/node_id`, and limit 7 gives consumed 8 at `/semantic_graph/nodes/0/node_id`. Alternatively, define the visit. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:176; spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md:51-55; crates/quire-contract-model/src/checked_package/v2/contract_package.rs:2134-2227 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix adds statement SHALLs that fix the unit and order: one unit per occurrence before one unit per region, array order, `nodes` then `requested_items` then `refusals`; the stop reports the pointer of the crossing occurrence or region and the count including that failed charge. The TC-047 limits 0, 17 and 18 and the `requested_items` first-occurrence case match the reviewer prototype. |
| FND-002 | fixed | TC-047 now pins the measured `mixed_fixture` oracle: value 7, reference 20, total 27. Limit 27 admits. Limits 7, 20 and 26 stop at consumed 8, 21 and 27 with pointers that match the reviewer prototype exactly. |
