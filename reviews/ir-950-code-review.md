---
id: SR-1395
title: "IR-950 Rust corpus generator code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-950; src/bin/quire-contract-corpus-generator.rs; src/bin/quire-contract-corpus-generator/cases.rs; src/bin/quire-contract-corpus-generator/ordered_json.rs; tests/it/conformance.rs; corpus/contract-v0.1/README.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
---

## Summary

The Rust generator, ordered JSON model, all 107 case builders, regeneration test, README, and removed Python implementation were reviewed. The generated deep coverage trace embeds a fixed requirement digest, and generation mutates the destination before the runner has su.

## Verdict

**FAIL** — FND-001 violates the repository's explicit digest/pin rule; FND-002 can leave the checked-in corpus partially regenerated after an ordinary runner failure.

## Scope

- FR-018-AC-1 (examined): Running the corpus directory yields one matching row per input with non-empty trace targets, and the union of observed tokens equals the published inventory, so every registered public construct, STD-001 diagnostic, operation, and boundary token is covered; a corpus missing a token's only fixture, a fixture with an unknown operation prefix or a missing expectation, and oversize, over-count and over-budget corpora fail before any row is written.
- FR-018-AC-2 (context only): Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing.
- FR-018-AC-3 (context only): Raw package probes pin exact and one-past wire depth, and quoted delimiters do not count toward depth.
- The new regeneration test carries #[trace("FR-018-AC-1", "TC-018")], compares the full relative-file set and every file's bytes, and uses real generator and runner binaries. Its binding is correct. CI workflow and Makefile are unchanged. The Rust code adds no unsafe block, allow lint, external network, or foreign-repository copy.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new case builder embeds a fixed 64-digit requirement digest; changing the source requirement leaves a stale pin in the generator and conflicts with CLAUDE.md's no new hash/digest/pin rule. | src/bin/quire-contract-corpus-generator/cases.rs:1189 |
| FND-002 | medium | Generation deletes the destination's corpus directories before the runner executes; a missing or failing runner leaves the checked-in corpus replaced by incomplete inputs and placeholder expectations. | src/bin/quire-contract-corpus-generator.rs:123 |

## Rust Review

The ordered JSON serializer intentionally preserves object declaration order for byte parity. The panic/index operations in its builder are confined to locally authored case construction, not untrusted inputs. The case table uses the existing conformance runner for observed tokens rather than asserting declared coverage as observation. Formal gates belong to the author and were not rerun in this review; the focused regeneration test is behaviorally meaningful but cannot protect the destination from a failed invocation.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: the requirement digest is derived from the parsed fixture package through `canonical_requirement(..., CanonicalProfile::V1).digest`; the 64-digit literal is gone. |
| FND-002 | fixed | Round 1: generation and runner self-check complete in a sibling stage before the destination is renamed; the new `tc_018_failed_regeneration_preserves_the_existing_corpus` test verifies paths and bytes after a missing runner. |

After excerpts (same source locations):

- FND-001: `let digest = parsed.canonical_requirement(&parsed.requirements()[0], CanonicalProfile::V1).expect("coverage fixture requirement canonicalizes").digest().to_string();`
- FND-002: `generate(root, &staged_corpus, runner)?;` precedes `fs::rename(corpus, &backup)?;` and `fs::rename(&staged_corpus, corpus)`.

## Current verdict

**PASS ** — both original findings are fixed by inspected source changes. The original review verdict above remains the frozen-head record; the final aggregate gate is the author's pending pre-PR qualification.

Rebase confirmation (current main ): FND-001 remains fixed because the entire `requirement_digest` member and `coverage-digest` case were removed from the upstream corpus contract; the Rust `trace` builder now emits only `{kind: deep|shallow}`. FND-002 remains fixed because sibling staging, runner self-check, and destination swap still occur in that order. The conflict resolution matches main's 106 inputs and `2^127` numeric range fixture, and `Scratch::empty` was moved into the shared support module. No new finding in this narrow rebase review. **Current verdict: PASS **, subject to the author's final aggregate gate.
