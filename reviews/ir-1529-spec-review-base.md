---
id: SR-202611081512300012
title: "spec-review (base) of IR-1529 source-declared object reader requirements"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #1019; spec/checked_package/functional/FR-362..FR-368, spec/checked_package/matrix/TC-475-source-declared-object-reader.md, spec/checked_package/matrix/tests.md (7 new rows)"
review_set: subset
---

## Summary

Ticket: IR-1529. Base spec review of the PR diff only (`git diff origin/main...HEAD`):
seven new FRs (FR-362 umbrella, FR-363 to FR-368 one per QSL ADR-013 QC-30 item 4
reader change), TC-475 and seven matrix rows. Source requirements read by id from
the public QSL checkout at `origin/main` (ADR-013 QC-30, FR-569, FR-390); QSpec
consulted read-only and cited by id only. `quire validate` (quire 0.44.0, engine
0.59.1, `QSPEC_DIR` set) passes the nine changed files with EARS warnings only
(see the ears-conformance SpecReview).

Every QC-30 item 4 change has its own FR with Test ACs, and every AC is mapped to
TC-475 in the matrix. The ids are free, the relationships resolve, no QSpec text,
path or internal name beyond existing precedent is copied, and no pin, SHA,
digest or compatibility layer is introduced. Four requirements are wrong as
written against this repo's existing FR-038/FR-064 rules or against QSL FR-569,
and several refusal oracles are unnamed, so TC-475 cannot compare the codes and
loci it says it compares.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-363 widens the protocol owner join without amending FR-064, whose Description requires the edge to name a selected model declaration and whose AC-4 still requires a source-local declaration to refuse `invalid_model_binding`/`malformed-declaration` at `owner_model`; the two FRs contradict. FR-363 also places the refusal "at the same locus (the clause or role)", but FR-064's missing-selection refusal is at the target's own `node_id`. | FR-363, FR-064-AC-4 |
| FND-002 | high | FR-363-AC-3 is unreachable: FR-038-AC-61/AC-155 already join every `SourceOwner` to `lock.sources` in the owner-join step and refuse `missing_declaration`/`missing-selection` at the source type's own `node_id`, before any protocol clause check, so a clause over a source type with no lock entry can never refuse at the clause. A node has one owner, so "a `SourceOwner` whose authority or identity differs from the entry" (AC-3, AC-4) names no constructible case. The protocol step needs only: the edge target is an admitted source-declared object type. | FR-363-AC-3, FR-363-AC-4, FR-038-AC-61, FR-038-AC-155 |
| FND-003 | high | FR-365 rules 3 and 4 have no data source. A source type's package node binds only fields, operations have no node (QSL FR-569 "Lowering and identity"), and the source-object row carries fields and populations but no operations (QSL FR-569 "Source-object identity rows"). The reader therefore cannot know a type's own operations, tell own from inherited-only, detect a missing row, or check `effect` for an operation no `state`/`operation_anchor` frame names. AC-3's "operation the type does not declare", "inherited-only operation" and "missing row for an own operation", and AC-4 for unanchored operations, cannot be implemented. | FR-365-AC-3, FR-365-AC-4, FR-365 |
| FND-004 | high | FR-366 reverses the dispatch family direction: it requires every candidate's declaring type to be "the context or a source supertype of it", but QSL FR-569-AC-5 has `Order.size` and `Sub.size` as candidates for context `Order` with `Sub extends Order`, so a redefining subtype is a valid candidate and FR-366 would refuse the target emission. FR-366 also speaks of a per-candidate declaring type, which the dispatch row shape owned by QSpec FR-322 (AC-290, AC-296) does not carry; restate the check over that row's actual members. | FR-366, FR-366-AC-3, FR-366-AC-1 |
| FND-005 | medium | Supertype checks have no package oracle. A source type node carries no `extends` information (body = field bindings, dependencies = bound type nodes), so FR-367-AC-4 "`supertypes` are not the node's direct supertypes" cannot be checked, and FR-366's family check needs the FR-367 stage rows, which are supplied beside the package, not to the strict reader. The FRs do not say which entry point runs FR-366's family check or what is checkable without the rows. | FR-367-AC-4, FR-366 |
| FND-006 | medium | FR-367's row classification ("classified in this order and the first failure SHALL refuse") reads as per-row first failure, while QSL FR-569 classifies every supplied row, reports missing only after all rows are classified, and reports the first reason in FR-390-AC-67's reason order. With an unknown row before a foreign row the two give different reasons. The added duplicate-key ("conflicting witness") reason is not in FR-569 and its place in the reason order is not stated. | FR-367, FR-367-AC-3 |
| FND-007 | medium | Refusal oracles are unnamed. FR-364-AC-4/AC-5, FR-365-AC-2..AC-4, FR-366-AC-3/AC-4, FR-367-AC-3/AC-4 and FR-368-AC-3/AC-4 say only "refuses"; FR-367 and FR-368 name "foreign/unknown/conflicting/missing witness" without the closed typed outcome. TC-475 step 2 says "Compare code, cause and locus with the requirement", which the requirements do not supply. IR owns these: reader checks reuse the existing code of the analogous model check (FR-038 binds operation effects to the QSpec TC-977 codes), and stage-row joins use FR-352's existing closed outcomes. | FR-367-AC-3, FR-363-AC-3, FR-364-AC-4, FR-365-AC-2, FR-365-AC-3, FR-366-AC-3, FR-368-AC-3, TC-475 |
| FND-008 | medium | FR-365 adds a second origin and FR-367/FR-368 add typed reads and stage-row inputs, but FR-038 still says "the fixed provenance origin is `model_document_operation_frame`" for the encoder and binds the reader to QSpec STD-640's closed schema, and FR-019 (public inventory, AC-21 "the fixed provenance origin is emitted") lists neither the new variant nor the new row and read types. The encoder change QSL needs to emit source rows is not specified. | FR-365, FR-038, FR-019-AC-21, FR-367-AC-5, FR-368 |
| FND-009 | medium | The umbrella AC FR-362-AC-1 is real in intent but its oracle is "as on the commit before this change set", a baseline snapshot that no TC step can produce without recording old outputs. The real check is that the named existing suites pass with no expectation edits, including their exact work-charge assertions (FR-362 Behavior also promises no new charge). The list omits TC-472 (inherited-operation dispatch) and FR-064's protocol-owner tests, which cover the paths FR-363 and FR-366 change. | FR-362-AC-1 |
| FND-010 | medium | TC-475 has no step that builds a package mixing a selected domain model with source types, which FR-362-AC-3, FR-363-AC-5, FR-365-AC-5/AC-6, FR-366-AC-5, FR-367-AC-6 and FR-368-AC-4/AC-5 need. Step 2's edit list omits population positions and duplicates (FR-364-AC-3..AC-5), the clause-function forms (FR-366-AC-2) and mixed dispatch rows (FR-366-AC-4). The Description's range "FR-362-AC-1 through FR-368-AC-5" leaves FR-365-AC-6 and FR-367-AC-6 implicit. | TC-475 |
| FND-011 | medium | FR-364-AC-4 "one with both a selected-document owner and a `SourceOwner`" cannot be constructed: `owner` is one closed `NodeOwner`. The real adverse cases, a population node with a `declaration` and a `ModelOwner`, and one with a `SourceOwner` and no `declaration`, are not named, and "refuse as the ownerless case refuses today" gives the wrong locus (FR-038 refuses a missing owner at the node object and a wrong owner kind at `owner`). | FR-364-AC-4, FR-364 |
| FND-012 | medium | FR-368's input row diverges from the QSL source-operation row it consumes (QSL FR-390-AC-70: `source_authority`, `source_identity` `<type>::<operation>`, `anchor_id`, `structural_node`). FR-368 adds a separate "opaque selected source-operation identity", a declaring node and an operation name, and "its `SourceOwner` authority and identity" collides with the row's `source_identity`. FR-368 also does not say what the join compares when the optional `operation_effects` carrier is absent. | FR-368 |
| FND-013 | low | FR-367 requires a row's `members` to "equal the node body's bindings in order", but the row's members use QSL FR-531's `source_records` member encoding; the comparison (name, type node, optional presence) is not defined. | FR-367-AC-4 |
| FND-014 | low | FR-368 does not state its extent. QC-30 item 4 asks only for the operation-anchor witness kind; FR-368 should say that Exact type projection of a source-declared type (a `Reference` to it, a record view) keeps FR-352's unavailable-witness outcome and is not part of this change. | FR-368 |
| FND-015 | low | FR-367 Dependencies says QSL FR-390 and FR-531 own the row content; the source-object row's content is QSL FR-569's ("Source-object identity rows"), with FR-531 owning the file. | FR-367 |

## Open questions from the author, classified

1. QSpec has no `source_declaration_operation_frame` origin. (b) QSpec dependency, does not block merge. Build FR-365 in IR's reader and encoder now; FND-008 makes FR-038 say IR admits the variant ahead of QSpec's closed schema, with the model-row vectors unchanged.
2. Population reference admission. (b) The QSpec FR-322 exception (QC-30 item 2c) is not adopted yet; does not block. Build per FR-364 after FND-011 is fixed.
3. `owner_model` wire for a `SourceOwner`. (a) Resolved: `owner_model` is a local `NodeRef`, so no wire change is needed. The fix is FND-001 and FND-002.
4. Stage-row Rust names shared with QSL. (b) IR owns its public API names, as it does `SelectedOperationMemberRow` (FR-352). The coder names them in IR and QSL adapts. Field sets follow QSL FR-569/FR-390 (FND-012).
5. Error code names. (a) Spec defect here (FND-007). IR can name every code from existing vocabularies.
6. FR-352 witness extent. (a) Settled by QC-30 item 4's text: the operation-anchor witness only. Make it explicit (FND-014).

FND-003 needs a choice that is the author's to make here: narrow FR-365 to the checks the package and rows can support (rows for anchored operations checked against the anchor frame; coverage and own/inherited taken as producer facts). Full coverage would need QSL's source-object row to carry operations, which is a QSL ask and non-blocking.

## Verdict

Not mergeable: four high findings (FND-001 to FND-004), all fixable in this PR by editing FR-363, FR-064, FR-365 and FR-366. Clean units: FR-362-AC-2, FR-362-AC-3, FR-363-AC-1, FR-363-AC-2, FR-363-AC-5, FR-364-AC-1, FR-364-AC-2, FR-364-AC-3, FR-364-AC-5, FR-365-AC-1, FR-365-AC-2 (apart from FND-007), FR-365-AC-5, FR-365-AC-6, FR-366-AC-2, FR-366-AC-4, FR-366-AC-5, FR-367-AC-1, FR-367-AC-2, FR-367-AC-5, FR-367-AC-6, FR-368-AC-1, FR-368-AC-2, FR-368-AC-5, and the seven matrix rows. No QSpec text or internal path is copied into the public files, and there are no pins, SHAs, digests or compatibility layers.
