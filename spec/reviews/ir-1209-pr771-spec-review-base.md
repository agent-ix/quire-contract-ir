---
id: SR-202610310124
title: "spec-review/base review of PR #771 QSV boundary"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md; spec/model/functional/FR-019-rust-library-interface.md; spec/model/functional/FR-028-separate-cycle-free-contract-model.md; IR-1209"
review_set: subset
---

## Summary

Reviewed FR-353 QSV admission boundary, FR-019 public inventory and FR-028 planned dependency against current QSV/IR source and the existing TC-058 inventory parser. One high inventory defect remains. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- `FR-353` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): When a consumer reads an equality application from an admitted `CheckedPackageV2`, Contract IR SHALL return QSV's opaque `AdmittedEqualityDecision` for the decision made by the producing checking stage. That value SHALL preserve the decision needed to form the consumer's `CheckedEquality` node without running a type checker in the reader. An operation identity, operand types, structural domain projection, or catalog law alone SHALL NOT be treated as that decision.
- `FR-353` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): The public read SHALL return a complete QSV-owned opaque `AdmittedEqualityDecision` or a closed typed refusal. The value's read-only typed API SHALL expose the admitted equality application node, occurrence, equality or inequality operator, each operand's source type and optional conversion target, the selected text, enum, quantity or plan schedule, and the complete resolved unit-table and filtered enum-member closure retained for that schedule. A quantity value SHALL preserve compound unit dimensions, canonical root-unit terms/exponents and exact conversion data as well as top-level operand u
- `FR-353` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): The QSpec-owned `checked_term_data.equality_decisions` carrier SHALL identify each decision by the equality application's own `NodeRef` and exact `occurrence`, with an empty `term_path`. Every composite application is its own graph node. An `and`, `or`, aggregate or binding parent SHALL reference a child equality application by that child's `NodeRef`; the equality row SHALL name the child, not the parent. The reader SHALL reject a foreign or absent application, a node of another form, a non-empty `term_path`, and duplicate (`application`, `occurrence`) equality rows. The reader SHALL check the
- `FR-353-AC-1` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): A public external consumer reads admitted `=` and `!=` applications by each equality application's full node identity and exact occurrence and receives QSV `AdmittedEqualityDecision`; its read-only typed API exposes the producer-selected operator, original operand positions, source and conversion target types, and exact schedule in source order. `CheckedEquality::from_admitted_decision` consumes that value without a checking pass. Two equalities referenced by one `and` parent, one referenced by `or`, and equality children referenced by aggregate-member and binding-value forms remain distinct,
- `FR-353-AC-5` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): Structural public-API evidence shows that IR's public `CheckedEqualityDecision` remains a raw wire carrier, while the admitted read returns QSV `AdmittedEqualityDecision` with private fields, no public raw-field constructor and a `#[doc(hidden)]` mint requiring QSV `ReaderSeal`; neither mint nor seal is re-exported through IR's public API. Cargo metadata and public-API evidence show QSV below the IR model, no QSL crate dependency in either IR package, and no IR-owned opaque wrapper. The audit records that the public hidden mint remains callable from another crate with a seal, so provenance is
- `FR-353` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): QSpec [STD-613](https://linear.app/agent-ix/issue/STD-613) and [STD-619](https://linear.app/agent-ix/issue/STD-619) supply the FR-322 `checked_term_data.equality_decisions` wire and vectors for the authoritative, bounded carrier. FR-322-AC-288/289/292 require each equality row to identify its own application node and occurrence with an empty `term_path`; parent bodies reference child application nodes. Current IR admission does not yet retain the producer's schedule, conversion choices, complete resolved unit-table closure or filtered enum member index. This FR allocates no further V2 wire mem
- `FR-019` (examined, `spec/model/functional/FR-019-rust-library-interface.md`):  `CheckedPackageV2` returning QSV's opaque `AdmittedEqualityDecision`. Its exact method signature depends on QSV IR-1223; neither the QSV type nor its hidden mint or `ReaderSeal` is re-exported from the model crate. The existing public `CheckedEqualityDecision` remains the raw wire carrier, not an admission proof. The planned read is excluded from the implemented public inventory until its code lands. FR-052 and FR-053 specify additional public model APIs; their owning requirements retain those API obligations, inc
- `FR-019` (examined, `spec/model/functional/FR-019-rust-library-interface.md`): | `checked_package` (V2 admitted equality read; planned) | associated admitted-equality read returning QSV `AdmittedEqualityDecision`; exact signature pending QSV IR-1223, with no IR re-export of the QSV type, mint or `ReaderSeal` | FR-353 |
- `FR-019-AC-5` (context_only, `spec/model/functional/FR-019-rust-library-interface.md`): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands; exporting a planned item fails the inventory until its row is marked implemented. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path. The retired `PACKAGE_DOMAIN_V2` export is absent.
- `FR-028` (examined, `spec/model/functional/FR-028-separate-cycle-free-contract-model.md`):   ```text quire-verification-contracts -> quire-contract-model quire-contract-model -> quire-contract-ir quire-semantic-value -> quire-contract-model  (planned for FR-353 / IR-1223) ```  Arrows point from dependency to consumer. No package may depend, directly or transitively, on itself. The model package admits no optional feature that reintroduces an owner/bridge dependency. The bridge accepts owner values only through their constructor-private public APIs; the package split adds no callback, trait-object validator, wire mirror, trust flag or local owner parser. The planned QSV semantic-valu
- `FR-028-AC-1` (context_only, `spec/model/functional/FR-028-separate-cycle-free-contract-model.md`): Cargo metadata for every production feature combination is acyclic and contains no owner or TL dependency reachable from `quire-contract-model`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-019 marks the new read row planned, but TC-058 parses it as implemented and expects nonexistent QSV exports. | spec/model/functional/FR-019-rust-library-interface.md:248 |

## Evidence

The new Public items row says `(V2 admitted equality read; planned)`, but TC-058 `expected_items_from` skips rows only when the module cell contains `planned until` (tests/it/public_interface.rs:883-884). This row lacks that marker. The test therefore counts `AdmittedEqualityDecision` and `ReaderSeal` as implemented `checked_package` exports, although FR-019 says neither is re-exported and QSV IR-1223 has not published the API. Mark the row with the inventory parser's planned form or keep it outside the implemented table.

A direct reproduction of TC-058's parser rule yields `skip_by_TC_058: false` and inventoried names `AdmittedEqualityDecision`, `ReaderSeal` for the new row.

## Verdict

FAIL: the planned inventory row would make TC-058 expect unimplemented exports.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | FR-353 requires a complete QSV checked equality but does not require the selected Exact identity/descriptors or a QSV admission construction path needed to turn structural wire facts into its operand types, unit table and enum index. | spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md:31 |

### FND-002 evidence

FR-353 Inputs name only the admitted package, application, occurrence, work limit and sidecar decision. The current `CheckedEqualityOperandDecision` carries `CheckedNodeId` source and conversion types, and its unit and enum rows likewise carry structural nodes and literal facts (`checked_term_data.rs:57-76, 100-179`). QSV `CheckedEquality` instead stores `EqualityOperand` values with Exact `ValueType`, `UnitTable` and `EnumMemberIndex` (`quire-semantic-value/src/declaration.rs:2176-2240`). The current IR `project_exact_type` needs `SelectedExactIdentities`, rejects union, does not cover enum or compound quantity, and has a general `UnsupportedExactType` branch (`exact_projection.rs:825-1080`); FR-353 also expressly forbids IR to project `ValueType`s (`FR-353:96-101`). QSV's current held-id unit insertion is crate-private, and `Unit`, `CompoundUnit` and `EnumValue` have no public admitted-field constructors (`quantity.rs:171-174`, `unit.rs:159-168, 688-713`, `enumeration.rs:119-127`). Thus a wire row that is complete in structural facts is not yet a constructible QSV decision, especially for enum and quantity cases required by AC-1. The QSV value's application/occurrence read-only provenance API is already explicitly required by FR-353 Outputs; QSV IR-1223 must carry it in the eventual mint contract.

Require an authoritative QSL-stage selected Exact identity/descriptor input, joined to the same admitted application and all source/conversion type, unit and enum references, with a typed unavailable-witness outcome when incomplete. Allocate to QSV IR-1223 a mint/constructor contract that can preserve those held identities and full resolved tables without rechecking or rebuilding authority; its read-only application and occurrence accessors must retain the joined provenance. State how that admitted input reaches the IR reader while preserving the FR-028 dependency direction. A general FR-352 projection call cannot silently fill this gap, because FR-353 forbids such reconstruction and the current projection does not cover all required shapes.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-019's module cell says `planned until QSV IR-1223 and IR-1209 code land`, which the existing TC-058 `expected_items_from` rule skips. The row therefore no longer invents implemented exports. |
| FND-002 | still-open | FR-353 still lists no selected Exact identity/descriptor input or admitted construction path for the QSV operand types, unit table and enum index required by AC-1; QSV IR-1223's mint and provenance API are not yet specified. |
| FND-002 | fixed | Round 2, head `180d1dc0`: FR-353 Inputs now require QSL-stage source and compound `UnitId`, model enum declaration/member `NodeKey`, Text and Plan identity/descriptor rows alongside the package. Behavior joins selected documents, original identities and admitted nodes; missing rows yield `UnavailableEqualityWitness` before minting. QSV owns held-ID unit/enum construction and application/occurrence getters. AC-6/7/8 cover positive, missing/conflicting and bounded affine-fold cases without semantic recheck or identity/schedule reselection. |

## Author disposition (FND-002)

Addressed in PR #771 pending reviewer verification. FR-353 now requires the
same QSL stage's source and compound `UnitId`, model enum declaration/member
keys, Text type identity and Plan type identity rows alongside the admitted
package. The IR read joins them to the selected documents and exact admitted
nodes, charges bounded representation mapping, and returns typed
`UnavailableEqualityWitness` without QSV minting when a required row is
absent. Mismatched rows refuse at their structural locus; no Exact key or
semantic decision is reconstructed. QSV IR-1223 owns the descriptor path that
retains held unit identities, filtered enum facts and application/occurrence
provenance in its opaque value. That path may mechanically fold a producer
selected ordered exact affine conversion path for QSV's private canonical
unit scalar, with typed failure on arithmetic or work exhaustion and no new
identity or schedule selection. QSL-1398 owns source unit/model enum rows;
QSL-1390 owns compound unit/Text/Plan rows. The preceding still-open entry
records the independent review verdict before this author fix.
