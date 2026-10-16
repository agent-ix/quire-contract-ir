---
id: SR-11810
title: "IR-759 Slice C base spec review of FR-052 census construction rules and AC-17..19"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@794d6e2 (PR 417, diff vs origin/main 6b80801); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-759. Base checklist over PR 417 at 794d6e2 (FR-052, TC-452, spec/model/matrix/tests.md), checked against merged crates/quire-contract-model/src/source_obligation.rs on origin/main and, as a read-only design reference only, branch code/ir-759-neutral-census-v2. Focus: precedence, the IR-reading labels, AC-8 supersession and stale envelope clauses.

## Verdict

**NOT MERGEABLE as-is**: four medium findings, no high. They are wording fixes inside this spec PR.

What holds:

- No AC id renumbered; AC-16 is byte-identical; AC-1..3, 5..7, 9..11, 15 stay Planned and AC-17..19 are Planned/UNRUN with explicit no-owner-native-credit clauses. Nothing claims owner-native completeness.
- AC-15's binding refusals now have three distinct named causes (`MissingCriterionSourceBinding`, `ForeignCriterionSourceBinding`, `ConflictingCriterionSourceBinding`) and defined triggers.
- The assertion pipeline, dispositions, endpoint resolution and ordering are each decisive enough to write a failing test from; every ordering key named is `Ord` on origin/main (SourceIdentity, SourceAuthoredCriterionId, SourceSemanticOccurrenceId, SourceProductionFunctionId, SourceAssertionProvenance, SourceInventoryIdentity). After exact-duplicate collapse the rejection sort key is total.
- The AC-8 transition removes the envelope port outright; no shim, fallback or second constructor is specified. Not a compatibility layer.
- The Behavior text no longer has the census accept a `SourceInventoryEnvelope`.
- AC-7's plural 'declared obligation sources' versus the single `obligation_scope()` reading is raised in Open questions and the FR-052 body labels the conflicting-binding rule and the QSL-row rule as IR readings.
- FR-019: nothing in this PR makes FR-019 false now. FR-052 still describes the current compiled surface (envelope port + outcome boundary) and FR-019:191 still lists it exactly; the new types are 'when implemented'.
- Gates: `quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md'` exits 0. `quire coverage --scope . --strict`: 51 unbacked / 0 contradicted at 794d6e2; 48 / 0 at origin/main (temporary worktree, removed). The diff of unbacked rows is exactly FR-052-AC-17, AC-18, AC-19 (strict exits 1 on both; pre-existing).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-18 orders positions 'Quire binding, Quire, IR V2, QSL, quire-code-rs', but the precedence rules check a conflicting binding after the Quire outcome passes. A conflicting binding plus an incomplete Quire outcome gives the binding error by AC-18 and IncompleteOwnerInventory(Quire) by the rules: two rules for one condition. Fix: spell AC-18's order as 'missing or foreign binding, Quire outcome, conflicting binding, IR V2, QSL, quire-code-rs' (and the same in TC-452 Expected Results). | spec/model/functional/FR-052-source-obligation-correspondence.md:469, 277-280 |
| FND-002 | medium | The IR-reading labels are not carried everywhere. AC-17 asserts 'every QSL row is ZeroMappings(NoAuthoredMapping)' as acceptance with no IR-reading label, and TC-452's synthetic procedure (a binding whose obligation scope differs from a Quire criterion row's obligation declaration) and Expected Results (ConflictingCriterionSourceBinding) adopt the conflicting-binding definition unlabelled. If the owner rules otherwise, AC-17 and TC-452 silently encode the wrong answer. Fix: mark both clauses in AC-17 and TC-452 as IR readings pending owner confirmation (FR-052 Open questions). | spec/model/functional/FR-052-source-obligation-correspondence.md:468; spec/model/matrix/TC-452-source-obligation-correspondence.md:161-162, 218-220 |
| FND-003 | medium | AC-8's end state after supersession is not specified. The code PR removes every FR-052-AC-8 test, and AC-8's own text ('the public module exposes no census or complete-census builder') becomes false once the builder lands, yet nothing says whether the AC-8 row is retired, re-worded or given a superseded status; left as is it becomes an unbacked row in coverage --strict and a criterion the code contradicts. Fix: state the row's post-transition form (for example, the code PR rewrites AC-8 to a 'Superseded by FR-052-AC-17' row without a Test verification) so the code PR has one prescribed action. | spec/model/functional/FR-052-source-obligation-correspondence.md:459, 472-474 |
| FND-004 | medium | The supersession note lists what the code PR removes but not the other envelope clauses that become untestable or false then: AC-10's last sentence ('stop_reason: None on a provisional envelope cannot be promoted to Complete'), TC-452 Test Procedure 111-113 and Expected Results 202-203, FR-052 Description 25-28, Behavior 194 and 257-263, and Outputs 81-87 (and FR-019:191's SourceInventoryEnvelope entry, which is the code PR's FR-019 update). AC-10 is Planned, so its envelope clause would be an AC no test can exercise. Fix: list these clauses in the transition note as rewritten or deleted by the census-builder change, or reword AC-10's clause now so it does not depend on the envelope. | spec/model/functional/FR-052-source-obligation-correspondence.md:25-28, 81-92, 194, 257-263, 461; spec/model/matrix/TC-452-source-obligation-correspondence.md:111-113, 202-203 |

## Examined units

```yaml
scope:
- id: FR-052-AC-8
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'A default-feature consumer can construct and borrow provisional owner rows, ... and the public module exposes no census or complete-census builder. FR-052-AC-17 supersedes this criterion when the census builder lands: that code change removes `SourceInventoryEnvelope`, the envelope-taking `SourceObligationInput` constructor, the tests tagged FR-052-AC-8, and the no-census assertions in `tests/it/source_obligation.rs` (lines 368-369); the FR-052-AC-4 inspection of `SourceObligationInput` fields remains.'
- id: FR-052-AC-10
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: '... `stop_reason: None` on a provisional envelope cannot be promoted to `Complete`, and a new owner stop type cannot be projected into a string or generic early stop.'
- id: FR-052-AC-15
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: An explicit source-author binding connects the selected Quire corpus/obligation scope to the selected IR `SourceIdentity`. A missing, foreign or conflicting binding refuses a complete census with distinct typed causes; ...
- id: FR-052-AC-16
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: With PLAT-1165's real Quire producer, normal exhaustion of the validated selected scope issues `Complete` ... Until this producer exists, no Quire `Complete` or full FR-052 census is claimed.
- id: FR-052-AC-17
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given four synthetic `Complete` outcomes and a valid binding for the AC-2 fixture shape, `build_source_obligation_census` returns a census whose inventory rows, dispositions, missing-mapping reasons, accepted rows and their provenance, and orderings follow the census construction rules; an occurrence asserted only without a production function is `ZeroMappings(ExplicitNoProductionFunction)`, and every QSL row is `ZeroMappings(NoAuthoredMapping)`. Permuting assertion order and outcome row order yields an equal census. This grants no owner-native completeness or FR-052-AC-2 full-owner credit.
- id: FR-052-AC-18
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given synthetic outcomes, a fault in each of two owner positions yields only the error for the earlier position in the order Quire binding, Quire, IR V2, QSL, `quire-code-rs`; within one position an outcome that is both in the wrong position and incomplete yields `InventoryScopeMismatch`, and one that has the wrong source and is incomplete yields `InventorySourceMismatch`. This grants no FR-052-AC-6 or AC-7 owner-native credit.
- id: FR-052-AC-19
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given synthetic `Complete` outcomes, an assertion that is both foreign and dangling is rejected only with `ForeignSource`; one repeated assertion ID with differing relation and differing provenance is rejected only with `ConflictingRelation`; two assertion IDs stating one relation with equal provenance yield one accepted row; two IDs stating one relation with differing provenance are both rejected with `ConflictingProvenance`; and an occurrence under a `ConflictingIdentity` node is rejected with `DanglingOccurrence`. This grants no FR-052-AC-3 full-owner credit.
- id: FR-052#refusal-precedence
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'The census shall report the first failing owner in fixed order: Quire, IR V2, QSL, then `quire-code-rs`. The binding belongs to the Quire position: a missing binding and then a foreign binding are checked before the Quire outcome, and a conflicting binding is checked after the Quire outcome passes and before IR V2. Within one owner position the checks run in this order: missing outcome, scope mismatch, source mismatch, then incomplete outcome.'
- id: FR-052#census-construction-rules
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: '### Census construction rules: `SourceObligationInput::new` shall take, in order, the selected `SourceIdentity`, an `Option<SourceCriterionSourceBinding>`, one `Option<SourceInventoryOutcome>` for each of the Quire, IR V2, QSL and `quire-code-rs` positions, and a `Vec<SourceCorrespondenceAssertion>`. ... A binding conflicts when a `Criterion` row of the Quire outcome has an `obligation_declaration()` different from the binding''s `obligation_scope()`. ... This definition of a conflicting binding is an IR reading pending owner confirmation (see Open questions).'
- id: FR-052#dispositions
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: A `SemanticNode` row in a conflicting key group is `ConflictingIdentity`. Otherwise a row is `Mapped` when an accepted row names it ... A row that is not `Mapped` is `ZeroMappings(ExplicitNoProductionFunction)` when it is an occurrence, or the node of an occurrence, named by an accepted row with no production function. Every other row, including an empty module and every QSL `RequirementRecord` and `PendingObligation` row, is `ZeroMappings(NoAuthoredMapping)`; no assertion endpoint can name a QSL row. That QSL rows are always `ZeroMappings(NoAuthoredMapping)` is an IR reading pending owner confirmation (see Open questions).
- id: FR-052#outputs-transition
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'Neither part exports a census or census builder. The census builder replaces the provisional envelope port: the code change that adds `build_source_obligation_census` removes `SourceInventoryEnvelope` and the envelope-taking `SourceObligationInput` constructor, and FR-052-AC-17 supersedes FR-052-AC-8 from that change on.'
- id: FR-052#behavior-census-input
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: The census accepts no `SourceInventoryEnvelope`. A well-formed outcome supplied in another owner's position shall be refused with `InventoryScopeMismatch`.
- id: FR-052#open-questions
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: '### Open questions: IR V2 semantic-node identity ...; Conflicting binding. ... FR-052-AC-7 names the bound corpus''s "declared obligation sources" in the plural. Whether one binding may cover several obligation declarations ... awaits owner confirmation.; QSL rows. ... awaits owner confirmation.'
- id: FR-052#status-paragraph
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: FR-052-AC-8 stays Implemented until the census builder lands; FR-052-AC-17 then supersedes it and the envelope port and its tests are removed. ... FR-052-AC-17 through AC-19 are Planned/UNRUN until the census builder exists; they exercise the neutral census over synthetic outcomes only, and passing them shall not change the status of AC-1 through AC-3, AC-5 through AC-7, or AC-9 through AC-11.
- id: TC-452#synthetic-procedure
  path: spec/model/matrix/TC-452-source-obligation-correspondence.md
  role: examined
  excerpt: Separately, build the same fixture from four synthetic `Complete` outcomes and a valid binding. ... Supply a missing binding, a binding naming another source, and a binding whose obligation scope differs from a Quire criterion row's obligation declaration. ... These checks use synthetic outcomes only.
- id: TC-452#synthetic-expected
  path: spec/model/matrix/TC-452-source-obligation-correspondence.md
  role: examined
  excerpt: The synthetic census matches the census construction rules field by field and is equal under permutation. Only the earliest failing position's error is returned ... and the occurrence under a conflicting node is `DanglingOccurrence`. None of these observations grants full-owner AC-1 through AC-3, AC-5 through AC-7, or AC-9 through AC-11 credit.
- id: TC-452#provisional-steps
  path: spec/model/matrix/TC-452-source-obligation-correspondence.md
  role: examined
  excerpt: 'Pass a provisional envelope with `stop_reason: None` and verify no API promotes it to `Complete`. ... A missing stop cause on the provisional input remains provisional.'
- id: tests.md#TC-452
  path: spec/model/matrix/tests.md
  role: examined
  excerpt: '| TC-452 | ... | FR-052-AC-1, ..., FR-052-AC-17, FR-052-AC-18, FR-052-AC-19 | Planned/UNRUN: ... AC-17..19 synthetic-census checks await the census builder and grant no full-owner credit; AC-8 is covered by provisional-port tests and is superseded by AC-17 when the census builder lands, which removes the envelope port and those tests; ...'
```
