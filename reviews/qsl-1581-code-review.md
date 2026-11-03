---
id: SR-2026110283845
title: "QSL-1581 PR 857 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; QSL-1581; crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs, spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Reviewed PR #857 at 283884c00c02174a080bfdbf0bc2493ceab9df07 against its stacked spec branch for FR-052-AC-28 through AC-30. The new QSL carrier types preserve exact labels and measured limit fields without a copied capability catalog. The acceptance test for requirement forms issues an impossible Complete QSL outcome.

## Verdict

**FAIL.** The new form identity can enter a Complete QSL outcome, contrary to the partial-form contract, and the AC-28 test exercises that invalid state.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `from_enumerator` accepts `RequirementForm` rows with a normal `Ok(None)` terminal event and labels the QSL outcome Complete. The new AC-28 test does exactly this because its synthetic enumerator has no stop. FR-052 and QSL FR-514 permit form rows only in stage-limited Incomplete inventories; a caller can pass these partial rows to the census as a complete QSL inventory. Reject this structural combination with a typed outcome error, then test both the refusal and the valid incomplete retention. | crates/quire-contract-model/src/source_obligation.rs:660; tests/it/source_obligation.rs:315 |

## Scope

| Role | Unit | Excerpt |
| --- | --- | --- |
| examined | FR-052 requirement-form behavior | "An owner-issued Complete spine inventory shall contain only checked requirement records and pending obligations; a stage-limited Incomplete inventory may retain these form rows." |
| examined | FR-052-AC-28 | "A default-feature model consumer constructs a SourceRequirementFormId ... all are distinct QSL-owned RequirementForm inventory identities." |
| examined | FR-052-AC-29 | "CapabilityKind values with two different admitted FR-057 labels ... remain distinct pending-obligation identities." |
| examined | FR-052-AC-30 | "A default-feature model consumer retains a spine S1 stage limit ... [and] a composed-lane cause retains None stage/setting/observed." |
| context_only | QSL FR-514 | "Requirement form row (only in an Incomplete outcome)." |

## Coverage

Reviewed the source-obligation public types and all new exhaustive match arms, the three new traced tests, and the two spec status edits. No CI or Cargo gate was run in this review pass; the code lead owns the final aggregate gate. Quire validation of the two changed spec files passed 2/2. No applicable AssuranceProfile was found under `spec/`.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At ddc1bc51, `from_enumerator` returns `RequirementFormInComplete` when normal exhaustion follows a QSL form row, while the typed `Incomplete` path retains those rows. The AC-28 test covers both outcomes; the criterion now states the terminal-state rule explicitly. |
