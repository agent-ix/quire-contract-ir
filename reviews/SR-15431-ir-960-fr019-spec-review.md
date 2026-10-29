---
id: SR-15431
title: 'Base spec review of IR-960 PR #508 FR-019 inventory correction'
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md:186; reviews/SR-15429-ir-960-code-review.md, reviews/SR-15430-ir-960-gap-analysis.md
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-019
  type: reviews
---
# SR-15431: Base spec review of IR-960 PR #508 FR-019 inventory correction

## Summary

Narrow independent review of the single added FR-019 Public items row on PR #508. The row lists seven extension evidence types owned by FR-348. The two review files added on this head were checked for byte-exact custody against the reviewer-held, disposition-complete originals.

## Verdict

**PASS.** The seven names exactly equal the seven `CheckedExtension*` crate-root exports in `crates/quire-contract-model/src/lib.rs`; each named type exists in the evidence or shared module. The row is implemented rather than planned and gives FR-348 as its owner, consistent with FR-019-AC-5 and TC-058. Both custody files have matching SHA-256 hashes. FR-019 and both review files validate grammar-clean with Quire.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

- **FR-019-AC-5** (examined): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.
- **FR-348** (context_only): owns the newly exported V2 extension evidence types.
- **TC-058** (examined): compares the model crate's default-feature public-item inventory against the FR-019 Public items table both ways.

## Base checks

The added row has one module group, seven distinct public type names and one owning requirement. No status exception or compatibility export is introduced.
