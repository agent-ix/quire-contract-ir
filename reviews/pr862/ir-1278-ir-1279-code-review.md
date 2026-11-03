---
id: SR-20261103150651
title: "IR-1278 IR-1279 PR 862 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #862; IR-1278, IR-1279; temporal_binding.rs; temporal_clause_names.rs; intake.rs; mod.rs; temporal_clock_bindings.rs; temporal_binding_public.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-019", type: references }
---

## Summary

Reviewed frozen 70a0de01 against the cited QSpec and IR acceptance criteria. Rust review inspected closed serde and canonical encoding, occurrence joins, ordered validation, public borrowing and external use. Ticket: IR-1278 and IR-1279.

## Verdict

**CONDITIONAL** — the external default-feature test does not establish the required two-occurrence public consumer behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The default-feature external consumer test reads and re-encodes one named occurrence only, so it does not prove that two names on one shared clause node remain distinct across the public borrowed API and writer. | crates/quire-contract-model/tests/temporal_binding_public.rs:93 |

## Coverage

- FR-019-AC-25, examined: external producer and consumer API; two shared-node named occurrences remain untested by this public test.
- FR-038-AC-437 through FR-038-AC-447, examined: typed wire, pair joins, ordering, coverage, public read and encoder, published vectors and limits.
- QSpec FR-322-AC-342 through FR-322-AC-370, examined: closed name and binding carriers, positive and adverse vector behavior.
- AD-004, context only: checked-package reader seam; no applicable AssuranceProfile was found.
- Diff-wide Rust style, copying and pin rules, examined: no new vendored artifact, digest, pin, compatibility layer or CI gate change found.

## Evidence

The test constructs `supplied` as a one-element array and asserts `names.len() == 1`; the separate conformance test exercises two occurrences inside the defining crate, but not the default-feature external consumer specified by FR-019-AC-25. The implementation itself exposes `Option<&[TemporalClauseName]>`, requires occurrence in the closed row, and the canonical encoder retains the supplied order. No aggregate gate was run in this reviewer lane.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The revised default-feature test applies QSpec CN-P2, asserts two rows with equal declaration, ordinals (0, 1), names Clause0 and Clause1, and strictly rereads the encoded two-row input. |

## Round 1 evidence

The disposition compares commit ed0a7a10 against frozen head 70a0de01. The revised tests exercise the published QSpec cases by reference. No new finding arose from the focused fix diff.
