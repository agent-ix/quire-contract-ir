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

Reviewed IR PR #589 head e4f628b8ac1aabb4d56c3b8d12f80a26da3e72bb against QSpec main 9f10eddf's FR-322-AC-118, TC-233, TC-280, and the published checked-package V2 schema. The positive form, identity, pair order, graph consistency, owner, occurrence, and typed writer/reader ownership are represented. FR-063-AC-1 through AC-4 and AC-6 pass this specification review. FR-063-AC-5 retains the adverse refusal cases and remains Planned and unclaimed pending the QSpec owner ruling in STD-497. The spec slice is conditionally PASS for proceeding with the independent cases, without claiming the AC-5 refusal code or full feature acceptance.

The changed documents pass focused Quire validation (2/2 grammar-clean). No applicable active AssuranceProfile was found under `spec/assurance/`. The JCS/SHA-256 preimage digest is the contract's canonical node identity, not a tracking digest or revision pin. This was a spec-only review; no Cargo, Make, or runtime tests were run. Acceptance criteria are planned, and no implementation or test coverage is claimed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-063-AC-5 requires `invalid_semantic_graph` at the equivalence node for an absent, zero, or negative `root_scale`. QSpec's closed `DimensionEquivalencePreimage` schema requires `root_scale` and gives its numerator and denominator `PositiveIntegerString` (`^[1-9][0-9]*$`); those three inputs therefore fail wire validation before semantic graph admission and require `malformed_wire`. A `declaration` added while the required sole occurrence remains `anchor` also fails `DeclarationOccurrenceRule` at wire validation. QSpec FR-322-AC-118 and TC-280 say `invalid_semantic_graph` for these same cases, so this is a contradiction in the authoritative QSpec source, not merely an IR typo. Resolve the QSpec schema/refusal precedence with its owner, then align this AC and its test oracle to the ruling without dropping any adverse case. The schema-valid unreduced positive fraction remains a semantic refusal. | FR-063-AC-5; QSpec FR-322-AC-118; QSpec TC-280 MU-23; QSpec `proposals/checked-package-v2/schema.json` `PositiveIntegerString`, `PositiveRational`, `DimensionEquivalencePreimage`, `DeclarationOccurrenceRule` | wrong-requirement |

## Disposition

FND-001 is **DEFERRED to STD-497**. That QSpec owner ticket covers all four conflicting cases: absent, zero and negative `root_scale`, plus `declaration` on an `anchor` occurrence. Its ruling must settle the wire versus semantic refusal code and be reflected in QSpec and IR before FR-063-AC-5 can pass. The criterion remains Planned, no refusal outcome is verified, and no adverse case is removed.

The remaining ACs pass as specification obligations: AC-1 gives a public typed writer/strict-reader round trip; AC-2 binds the closed preimage, ordered distinct pair, owner, JCS key and retained-key mutations; AC-3 carries QSpec's three positive time vectors and exact ratios; AC-4 carries the schema-valid pair and graph adverse cases; AC-6 preserves a single typed V2 path and prohibits a copied schema or fixture. The selected QSpec vectors and IR ownership links support those obligations. This is a conditional **PASS for the spec slice**, with FND-001 still open under STD-497.
