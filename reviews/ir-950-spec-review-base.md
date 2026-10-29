---
id: SR-1397
title: "IR-950 FR-018 spec applicability review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-950; spec/conformance/functional/FR-018-conformance-corpus.md; corpus/contract-v0.1/README.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
---

## Summary

FR-018 already owns the checked-in generator, observed-token rejection, and scratch-directory byte-for-byte regeneration gate. The change only ports the generator implementation and its documented command; no requirement, acceptance criterion, schema, or corpus wire interface changed.

## Verdict

**PASS** — no normative spec amendment is needed for this port. The code-review findings remain separate implementation concerns.

## Scope

- FR-018 behavior paragraph (examined): Downstream tools can execute the process runner without linking the Rust library. The checked-in generator is owned by this requirement, rejects a declared token the runner does not observe, and has a scratch-directory byte-for-byte regeneration gate.
- FR-018-AC-1 (examined): Running the corpus directory yields one matching row per input with non-empty trace targets, and the union of observed tokens equals the published inventory, so every registered public construct, STD-001 diagnostic, operation, and boundary token is covered; a corpus missing a token's only fixture, a fixture with an unknown operation prefix or a missing expectation, and oversize, over-count and over-budget corpora fail before any row is written.
- FR-018-AC-2 (context only): Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing.
- FR-018-AC-3 (context only): Raw package probes pin exact and one-past wire depth, and quoted delimiters do not count toward depth.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Rebase update

 over main, FR-018 still owns the generator, observed-token rejection, and byte-for-byte regeneration gate. Main's deep coverage input no longer carries a requirement digest, the corpus has 106 inputs, and the Rust port follows those current interfaces. No new normative spec amendment is needed.
