---
id: SR-2171
title: "Planless gap analysis of IR-661 PR 309"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@d03adb1dbada4626e44f50a14d2ec62dcfdb6d60; base 7c4addafb28b4d49fd12a0b5e44c27322aaa1256; crates/quire-contract-model/src/checked_package/shared.rs; crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs; crates/quire-contract-model/src/checked_package/v2/model_members.rs; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs; crates/quire-contract-model/src/checked_package/v2/operations.rs; crates/quire-contract-model/src/checked_package/v2/owner.rs; tests/conformance_qspec/main.rs; tests/it/checked_package_v2_model_members.rs; tests/it/checked_package_v2_owners.rs; tests/it/checked_package_v2_temporal.rs; tests/it/support/checked_package.rs; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; FCD FR-095/FR-094, QSpec FR-152/FR-154/FR-341 read-only"
review_set: subset
---

## Summary

Ticket: IR-661. Reviewed the frozen PR diff and merged FR-038 AC-165..173 at d03adb1dbada4626e44f50a14d2ec62dcfdb6d60.

## Verdict

**CONDITIONAL** — Two tagged criteria lack required complete-reader evidence.

## Examined units

- `FR-038-AC-165` (`examined`): PLANNED / UNRUN (IR-661). A selected, independently authored FCD-shaped document with a global `relationship` slot identity admits, while an owner-nested relationship, a foreign-package identity and a wrong-slot identity each refuse `invalid_model_bi
- `FR-038-AC-166` (`examined`): PLANNED / UNRUN (IR-661). Changing only an authored role makes the old name refuse `missing_declaration`/`missing-name` and the new name resolve on the same selected relationship. Destination-type and multiplicity changes are observed through the adm
- `FR-038-AC-167` (`examined`): PLANNED / UNRUN (IR-661). An unresolved `sourceEnd.type` or `targetEnd.type` refuses `missing_declaration`/`missing-name`; a type naming a relationship or another declaration of the wrong meaning refuses `invalid_model_binding`/`malformed-declaration
- `FR-038-AC-168` (`examined`): PLANNED / UNRUN (IR-661). Repeating a relationship identity, including under two different owners, refuses `invalid_model_binding`/`conflicting-binding` at the selection row. Two missing relationship identities are malformed declarations, not conflic
- `FR-038-AC-169` (`examined`): PLANNED / UNRUN (IR-661). `relationship_end.declaration` names the selected relationship's `relation`/`relationship` graph node and resolves through its content-only `ModelOwner`, with an empty body; a valid source or target object node used as its d
- `FR-038-AC-170` (`examined`): PLANNED / UNRUN (IR-661). Forward and inverse navigation choose opposite receiver/destination endpoints and admit the canonical result nodes specified by QSpec FR-152 Navigation. The independently authored finite cases in TC-048 exercise Reference, O
- `FR-038-AC-171` (`examined`): PLANNED / UNRUN (IR-661). For `quire.op.model.navigate`, the new relationship-end checks follow the specified operation-step order, explicit calling-node locus and RFC 6901 paths: owner recovery and declaration-kind eligibility before roles; roles be
- `FR-038-AC-172` (`examined`): PLANNED / UNRUN (IR-661). A homogeneous self-relationship with a resolved forward or inverse end and an eligible Reference or Option destination admits `quire.op.model.reaches` with two references conforming to its static endpoint owner and Boolean r
- `FR-038-AC-173` (`examined`): PLANNED / UNRUN (IR-661). Removing direction, category, composite or origin, setting each to null or a wrong type, using an unsupported direction/category value, or supplying a malformed common-schema origin branch refuses `invalid_model_binding`/`ma
- `crates/quire-contract-model/src/checked_package/shared.rs` (`examined`): //! Version-neutral I04 `CheckedPackage` vocabulary. //! //! These types carry no contract-version-specific shape of their own: they //! are the caller-facing limits, refusal/incomplete outcomes, and locked
- `crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs` (`examined`): use super::super::model_members::tests::{     differential_document, differential_documents, document, field, multiplicity, object_type,     read, BYTES, ORDERS, RECORDED, };
- `crates/quire-contract-model/src/checked_package/v2/model_members.rs` (`examined`): //! FR-322 "Model-owned members" and "Reference conformance" (QSpec STD-100, //! STD-101, STD-102): a `field` or `operation` member whose declaring node is //! a model declaration node resolves through the domain package the lock //! selects, because
- `crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs` (`examined`): use super::super::model_fields::build_field_tables; use super::*; use crate::checked_package::common::ValidationFailure; use crate::checked_package::shared::{
- `crates/quire-contract-model/src/checked_package/v2/operations.rs` (`examined`): //! FR-038/QSpec #76 operation-law validation: an `application` term's //! `operation` member is admitted opaquely nowhere past this module — its //! identity, laws, mode, member and leading operand shape are checked //! against the upstream `quire.c
- `crates/quire-contract-model/src/checked_package/v2/owner.rs` (`examined`): //! Closed node-owner wire and selected-evidence joins for FR-038-AC-153..155.  use super::{     CheckedDeclaration, CheckedNodeKind, CheckedNodeOwner, CheckedPackageWireV2,
- `tests/conformance_qspec/main.rs` (`examined`): // SPDX-License-Identifier: AGPL-3.0-or-later // Copyright (C) 2026 Agent-IX  //! FR-038-AC-107, AC-112 and AC-113: QSpec's CheckedPackage V2 fixtures and
- `tests/it/checked_package_v2_model_members.rs` (`examined`): // SPDX-License-Identifier: AGPL-3.0-or-later // Copyright (C) 2026 Agent-IX  //! IR-285: `CheckedPackageV2::read` over a package whose lock selects a real
- `tests/it/checked_package_v2_owners.rs` (`examined`): // SPDX-License-Identifier: AGPL-3.0-or-later // Copyright (C) 2026 Agent-IX  //! Owner-wire admission and refusal cases built from this crate's public wire vocabulary.
- `tests/it/checked_package_v2_temporal.rs` (`examined`): // SPDX-License-Identifier: AGPL-3.0-or-later // Copyright (C) 2026 Agent-IX  //! FR-038-AC-96 through AC-105 and AC-108, at the package: the temporal step
- `tests/it/support/checked_package.rs` (`examined`): // SPDX-License-Identifier: AGPL-3.0-or-later // Copyright (C) 2026 Agent-IX  //! Shared helpers for the I04 CheckedPackage integration tests.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The Set and Bag finite-destination rows call RelationshipDecl::navigation_type directly and assert only the enum variant. No complete-reader package test admits their canonical result nodes or checks that changing destination type or bounds changes the admitted node, as AC-170 requires. | crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:350 | correct-requirement-no-evidence |
| FND-002 | medium | The complete-reader reaches fixture uses the same receiver node as both operands. Its tagged test cannot fail if operand 1 is never checked, and it does not test an unrelated second operand or distinct subtype operands required by AC-172. | tests/it/checked_package_v2_model_members.rs:318 | correct-requirement-no-evidence |

## Coverage

Plan completion: not assessed
Matrix: FR-038-AC-165..173 each tagged; static bindings alone do not establish behavioral coverage.
Semantic review: examined the nine IR-661 criteria against changed tests and source.
Full CI, Kani, replay, cargo build and test were not run by this reviewer.
IR-663 refusal metadata retention is an explicit current gap; no conformance claim is made.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 8cde11e40fc1fd7180158721e66a48b47d1e57b3 |
| FND-002 | fixed | 8cde11e40fc1fd7180158721e66a48b47d1e57b3 |

## Disposition verdict

**PASS (round 1, scoped)** — All original findings are fixed at 8cde11e40fc1fd7180158721e66a48b47d1e57b3; no new defects found in the fix diff. This does not replace the initial verdict or assert full gate results.
