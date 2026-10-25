---
id: SR-558006
title: "IR-1014 PR 564 Rust and code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #564 at a08376c2; refinement reader, sidecar, writer and focused tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-061
    type: reviews
---

## Summary

Reviewed the PR diff against IR FR-061 and QSpec FR-375/FR-322 at the merged QSpec #442/#443 contracts. The early catalog checks are explicitly partial; FR-061-AC-4, full row typing and FR-375-AC-19 remain open. The repository's AGENTS.md, CLAUDE.md and Rust review baseline were applied. The sidecar validity defect below is in the delivered provenance slice.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Validate the `refinement_step_rows[].region` interval and locked source even when an owner is proof-ineligible. `validate_step_rows` checks only owner, ordinal and order; `validate_occurrences` checks a sidecar region only through a generated source-map occurrence, which an ineligible owner must not have. Consequently an ineligible owner with a step can carry `start >= end` or an unlocked `source` and still reach admission. Refuse the offending sidecar region and add an ineligible-owner vector. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:1416, 1495, 1541; QSpec FR-322 `refinement_step_rows` |

## Coverage

Examined FR-061-AC-2, AC-3, AC-6, AC-7, AC-8 and AC-9, with FR-061-AC-1 and AC-4 as context for the declared partial scope. QSpec FR-322-AC-128/129 and FR-375-AC-23 were context. The public wire structs, dedicated identity preimage, exact dependency collection, catalog order and local key checks have no other established defect in this pass. Plan completion: not assessed.

## Verdict

CONDITIONAL. Fix FND-001 in this PR. Complete catalog attestation remains owned by IR-874, and this review does not close IR-1014.

## Dispositions

Fix round 1 at PR head `b71dcf3022e9c94017b081f0d336eaa0e4efaaea`:

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `validate_step_rows` now checks each row region's source against `lock.sources` and requires `start < end` before occurrence checks. The new ineligible-owner vector removes generated occurrences and asserts both malformed intervals and unlocked sources refuse at the sidecar region. After excerpt: `if !locked_sources.contains(&row.region.source) { return Err(invalid_step_row_region(index, true)); } if row.region.start >= row.region.end { return Err(invalid_step_row_region(index, false)); }` |

Final verdict: PASS for the reviewed early refinement slice. IR-874 and the other open FR-061 criteria remain outstanding.

## String-edge gate re-review

At PR head `421176f25daed888a2fb549bfef55bd1ec76e97b`, the source-map root comparison uses `JsonPointer` instead of a raw string, the body header check is extracted unchanged, and class/open/memory spellings flow through typed `string_edge` functions. The `source_node` presence test in catalog absence is equivalent to the former memory-prefix test because `check_wire` first enforces exactly that pairing for each step. The new open-term vector reaches the stale-key stage for `open` and refuses `other` at the argument pointer. No semantic regression or new code-review finding was established. PASS remains for this head; the code lead owns the final aggregate gate.

## Root-gate regression re-review

At PR head `92aa6e74a10708bb8d0fca4d6849e5f662e4c0a5`, the refinement writer now emits each closed row through direct canonical writer events and bounded nested `CheckedTerm::encode_into`, removing the intermediate `serde_json::to_value` from production identity paths. The typed replacement walk reaches header, population, object, step, history, fairness, protocol and catalog node keys, plus nested expression targets. It preserves package IDs, class spellings, profiles and other non-node-key fields. The new typed writer and replacement vectors exercise the four target kinds, nested terms and representative catalog keys. The selected state-machine fixture corrections restore the former positive form checks under the newly enforced owner rule. No new code or Rust finding was established; PASS remains for this head, subject to the code lead's final aggregate gate.
