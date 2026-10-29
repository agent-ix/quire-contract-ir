---
id: SR-11743
title: "IR-759 Slice A EARS conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir (PR #411); spec/model/functional/FR-052-source-obligation-correspondence.md"
review_set: subset
---

## Summary

Ticket: IR-759 (Slice A). This review checks the requirement sentences that PR #411 added or rewrote in FR-052 for EARS form: a trigger or state, a "shall" response, and no editorial or rationale text written as a requirement.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-052 constructor sentence | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | The completed `SourceObligationInput` constructor shall take the selected source, an explicit Quire-selection-to-source binding, four independently optional owner `SourceInventoryOutcome` values (one per owner position: Quire, IR V2, QSL, `quire-code-rs`), and the author-authored assertions as typed arguments. |
| FR-052 position sentence | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | Each outcome's owner and scope shall match its required input position, and its source shall match the top-level selected source. |
| FR-052 census refusal sentence | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | If the Quire binding or any owner outcome is absent, incomplete, or has a mismatched scope or source, `build_source_obligation_census` shall return a typed error naming that owner and shall return no census. |
| FR-052 foreign-row sentence | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | A foreign-source inventory row is refused at the outcome boundary with `SourceInventoryOutcomeError::RowSourceMismatch` and never reaches the census; ... The census therefore has no foreign-inventory disposition, and one condition has one rule. |
| FR-052 observed-count sentence | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | The outcome's observed row count shall equal the number of retained rows; `SourceInventoryOutcomeError::ObservedCountMismatch` owns that rule and refuses outcome construction, so the census never receives a count mismatch. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The added clause "and one condition has one rule" (:301) is reviewer-facing rationale written into a requirement paragraph. It states no system response and adds nothing for an implementer. Similarly, "owns that rule" (:159) describes spec structure rather than behaviour. EARS form would be: "When an enumerator's observed count differs from its retained rows, `from_enumerator` shall return `ObservedCountMismatch` and no outcome." | spec/model/functional/FR-052-source-obligation-correspondence.md:159-161, 300-301 |

## Verdict

ACCEPTABLE WITH ONE LOW FINDING. The rewritten constructor, position and census-refusal sentences are well-formed "shall" requirements with explicit conditions. The unwanted-behaviour sentence ("If ... absent, incomplete, or has a mismatched scope or source ... shall return a typed error") follows the EARS If/then pattern.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The rewritten count sentence capitalizes after a semicolon: "The outcome's observed row count shall equal the number of retained rows; A mismatch is refused with `SourceInventoryOutcomeError::ObservedCountMismatch` at outcome construction". The text is a single-character typo introduced by fix. Behaviour is unaffected. | spec/model/functional/FR-052-source-obligation-correspondence.md:161-162 |

## Dispositions

Round 1. Reviewed the diff `git diff origin/main...HEAD`.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | "and one condition has one rule" is gone from FR-052:318. "owns that rule" is replaced by a behavioural statement (:161-163): "A mismatch is refused with `SourceInventoryOutcomeError::ObservedCountMismatch` at outcome construction, so the census never receives one." |
| FND-002 | still-open | This round introduced it, and the typo is present (FR-052:162, "retained rows; A mismatch"). It needs a one-character fix to lower-case "a", or a lead decision of accepted-no-change. |
| FND-002 | fixed | Round 2 : FR-052:162 now reads "retained rows; a mismatch is refused with". That one character is the only spec change in the commit. |
