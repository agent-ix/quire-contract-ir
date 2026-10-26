---
id: SR-589001
title: "Base spec review of IR-1032 dimension equivalence"
type: SpecReview
analysis: base
scope: "spec/checked_package/functional/FR-063-dimension-equivalence-node.md; spec/spec.md"
review_set: base
---
# SR-589001: Base spec review of IR-1032 dimension equivalence

## Summary

Reviewed IR PR #589 at corrected spec head 21f37cb8e3dc51ac60c22aa15288ad100ab3de79 against QSpec main 9f10eddf's FR-322-AC-118, TC-233, TC-280, and the published checked-package V2 schema. The positive form, identity, pair order, graph consistency, owner, occurrence, and typed writer/reader ownership are represented. FR-063-AC-1 through AC-4 and AC-6 pass this specification review. FR-063-AC-5 retains the adverse refusal cases and remains Planned and unclaimed pending the QSpec owner ruling in STD-497. FND-002's broad Description claim has been corrected. The final verdict is conditional **PASS for the spec slice**; no AC-5 refusal code or full feature acceptance is claimed.

The changed documents pass focused Quire validation (2/2 grammar-clean). No applicable active AssuranceProfile was found under `spec/assurance/`. The JCS/SHA-256 preimage digest is the contract's canonical node identity, not a tracking digest or revision pin. This was a spec-only review; no Cargo, Make, or runtime tests were run. Acceptance criteria are planned, and no implementation or test coverage is claimed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-063-AC-5 requires `invalid_semantic_graph` at the equivalence node for an absent, zero, or negative `root_scale`. QSpec's closed `DimensionEquivalencePreimage` schema requires `root_scale` and gives its numerator and denominator `PositiveIntegerString` (`^[1-9][0-9]*$`); those three inputs therefore fail wire validation before semantic graph admission and require `malformed_wire`. A `declaration` added while the required sole occurrence remains `anchor` also fails `DeclarationOccurrenceRule` at wire validation. QSpec FR-322-AC-118 and TC-280 say `invalid_semantic_graph` for these same cases, so this is a contradiction in the authoritative QSpec source, not merely an IR typo. Resolve the QSpec schema/refusal precedence with its owner, then align this AC and its test oracle to the ruling without dropping any adverse case. The schema-valid unreduced positive fraction remains a semantic refusal. | FR-063-AC-5; QSpec FR-322-AC-118; QSpec TC-280 MU-23; QSpec `proposals/checked-package-v2/schema.json` `PositiveIntegerString`, `PositiveRational`, `DimensionEquivalencePreimage`, `DeclarationOccurrenceRule` | wrong-requirement |
| FND-002 | high | The Description unconditionally states that every invalid equivalence refuses as `invalid_semantic_graph`. This claims a settled code for the four wire-invalid cases in FND-001 despite their deferral to STD-497, and it conflicts with the Behavior paragraph's explicit QSpec wire-before-graph refusal order. Replace only this broad Description sentence with a first-applicable-QSpec-check refusal rule; retain FR-063-AC-5 verbatim, Planned, and unclaimed pending STD-497. | FR-063 Description and Behavior; FR-063-AC-5; STD-497 | wrong-requirement |

## Disposition

FND-001 is **DEFERRED to STD-497**. That QSpec owner ticket covers all four conflicting cases: absent, zero and negative `root_scale`, plus `declaration` on an `anchor` occurrence. Its ruling must settle the wire versus semantic refusal code and be reflected in QSpec and IR before FR-063-AC-5 can pass. The criterion remains Planned, no refusal outcome is verified, and no adverse case is removed.

The remaining ACs pass as specification obligations: AC-1 gives a public typed writer/strict-reader round trip; AC-2 binds the closed preimage, ordered distinct pair, owner, JCS key and retained-key mutations; AC-3 carries QSpec's three positive time vectors and exact ratios; AC-4 carries the schema-valid pair and graph adverse cases; AC-6 preserves a single typed V2 path and prohibits a copied schema or fixture. The selected QSpec vectors and IR ownership links support those obligations.

FND-002 is **FIXED at 21f37cb8**. The Description now requires the first applicable QSpec FR-322 wire, graph, identity, or semantic check, with no partial admitted package. The diff from 6ad461b6 changes only this sentence; all six FR-063 AC rows are byte-identical. Independently verified the exact GitHub PR head, clean worktree, and focused Quire validation (2/2 grammar-clean). FND-001 remains **DEFERRED to STD-497**. The spec slice verdict is conditional **PASS**.
