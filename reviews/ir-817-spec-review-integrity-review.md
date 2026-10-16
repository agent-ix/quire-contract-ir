---
id: SR-11821
title: "spec-review/integrity review of IR-817 PR #418"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@f07017c25c744c66750f0e86ff0bb8b20bb9707d; PR #418; spec/conformance/functional/FR-020-json-conformance-interface.md; spec/conformance/matrix/tests.md; spec/core/functional/FR-012-anchors-clauses-dependencies.md; spec/core/matrix/tests.md; spec/model/functional/FR-023-executable-projection-binding.md; spec/tests.md; tests/it/conformance.rs"
review_set: subset
---

## Summary

Ticket: IR-817. Diffed the compound FR-020-AC-3 on origin/main (6b80801) against the ten new rows. Every obligation is kept: fixture-schema IntegerString spellings (AC-3), package-schema 2^53 bound (AC-4), probe/coverage refusal above 2^53 (AC-5), exact-bound pass (AC-6), decoder fixture result (AC-7), its expectation code (AC-8), schema-level refusal of a number in a valid-operation input (AC-9), canonical bytes (AC-10), strings-only spelling (AC-11), and corpus exit 0 with every fixture a match (AC-12). The "Landed with code change B (IR-274)" prefix was provenance, not an obligation; it moved to the conformance matrix status cell, which is correct. Cross-references FR-012-AC-6, FR-023 and the core matrix FR-011 row now cite FR-020-AC-5, which is the right criterion.

## Verdict

**FAIL (one medium)**: the conformance matrix FR-020 status cell states a retag that this PR itself performs as still pending. Remaining findings are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Conformance matrix FR-020 status says the tests are still tagged FR-020-AC-3 and AC-4..AC-12 show untagged until a follow-up retag, but this PR does the retag and quire matrix computes FR-020 12/12 tagged at f07017c | spec/conformance/matrix/tests.md:13 |
| FND-002 | low | AC-9 rewords the old "the same number in a valid-operation input" to "a JSON number": the pairing with the decoder fixture's number is lost, and the backing test mutates with 0 and 1.0, never the decoder fixtures' -10 and 1 | spec/conformance/functional/FR-020-json-conformance-interface.md:142 |
| FND-003 | low | AC-12 restates part of AC-1 (exit 0, one match per fixture input) and is backed by the same test; redundant criterion kept only to preserve the old clause | spec/conformance/functional/FR-020-json-conformance-interface.md:145 |
