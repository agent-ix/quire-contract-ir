// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete native witness checks through the public default-feature API.

use crate::checked_package_v2_owners::{complete_witness_fixture, CompleteWitnessFixture};
use ix_trace_rs::trace;
use quire_contract_model::{
    ExactTypeProjectionErrorCode as Code, NativeJoinBudget, NativeJoinCharge, NativeJoinChargeKind,
    SelectedExactIdentities,
};

fn ample() -> NativeJoinBudget {
    NativeJoinBudget::new(u64::MAX, u64::MAX)
}

fn acquire(
    fixture: &CompleteWitnessFixture,
    omit: Option<usize>,
    budget: &mut NativeJoinBudget,
) -> SelectedExactIdentities {
    let declarations: Vec<_> = fixture.identities.declaration_rows().cloned().collect();
    let fields = if omit == Some(1) {
        Vec::new()
    } else {
        fixture.identities.field_rows().to_vec()
    };
    let declarations = if omit == Some(0) {
        Vec::new()
    } else {
        declarations
    };
    let fields = if omit == Some(0) { Vec::new() } else { fields };
    let mut selected =
        SelectedExactIdentities::from_stage_rows(&fixture.package, declarations, fields, budget)
            .unwrap();
    selected = selected
        .with_admitted_type_rows(
            &fixture.package,
            if omit == Some(2) {
                &[]
            } else {
                &fixture.tuples
            },
            if omit == Some(3) { &[] } else { &fixture.units },
            budget,
        )
        .unwrap();
    if omit != Some(4) {
        selected = selected
            .with_source_union_rows(
                &fixture.package,
                fixture.identities.source_union_rows().cloned(),
                budget,
            )
            .unwrap();
    }
    if omit != Some(5) {
        selected = selected
            .with_source_record_rows(
                &fixture.package,
                fixture.identities.source_record_rows().cloned(),
                budget,
            )
            .unwrap();
    }
    if omit != Some(6) {
        selected = selected
            .with_source_tuple_rows(
                &fixture.package,
                fixture.identities.source_tuple_rows().cloned(),
                budget,
            )
            .unwrap();
    }
    if omit != Some(7) {
        selected = selected
            .with_source_enum_rows(
                &fixture.package,
                fixture.identities.source_enum_rows().cloned(),
                budget,
            )
            .unwrap();
    }
    if omit != Some(8) {
        selected = selected
            .with_source_collection_rows(
                &fixture.package,
                fixture.identities.source_collection_rows().cloned(),
                budget,
            )
            .unwrap();
    }
    selected
}

#[trace("TC-467", "FR-352-AC-22", "FR-352-AC-28")]
#[test]
fn complete_identity_witness_preserves_every_family_and_cloned_admission() {
    let fixture = complete_witness_fixture();
    let original = fixture.identities.clone();
    let mut budget = ample();
    let rebound = original
        .validate_and_rebind(&fixture.package, &mut budget)
        .expect("complete mixed native fixture");
    assert_eq!(rebound, original);
    assert_eq!(rebound.declaration_rows().count(), 2);
    assert_eq!(
        rebound.field_rows().len(),
        1,
        "unread selected field remains present"
    );
    assert_eq!(rebound.source_union_rows().count(), 1);
    assert_eq!(rebound.source_record_rows().count(), 1);
    assert_eq!(rebound.source_tuple_rows().count(), 1);
    assert_eq!(rebound.source_enum_rows().count(), 1);
    assert_eq!(rebound.source_collection_rows().count(), 1);
    let mut cloned_budget = ample();
    assert_eq!(
        original.validate_and_rebind(&fixture.package.clone(), &mut cloned_budget),
        Ok(rebound)
    );
    assert_eq!(
        budget, cloned_budget,
        "clone preserves binding, decisions, and exact charges"
    );
    assert_eq!(original, fixture.identities, "input remains immutable");
}

#[trace("TC-467", "FR-352-AC-23", "FR-352-AC-26")]
#[test]
fn complete_identity_witness_refuses_each_missing_family_and_private_channel() {
    let fixture = complete_witness_fixture();
    for (family, form) in [
        (0, "selected_declaration"),
        (1, "selected_effective_field"),
        (2, "selected_model_tuple"),
        (3, "selected_source_unit"),
        (4, "selected_source_union"),
        (5, "selected_source_record"),
        (6, "selected_source_tuple"),
        (7, "selected_source_enum"),
        (8, "selected_collection"),
    ] {
        let partial = acquire(&fixture, Some(family), &mut ample());
        let before = partial.clone();
        let mut budget = ample();
        let refusal = partial
            .validate_and_rebind(&fixture.package, &mut budget)
            .expect_err("valid subset cannot certify completeness");
        assert_eq!(
            refusal.code,
            Code::IncompleteIdentityWitness,
            "family {family}"
        );
        assert_eq!(refusal.form.as_ref(), form, "first missing family {family}");
        let expected = match family {
            0 => fixture
                .identities
                .declaration_rows()
                .map(|row| &row.structural_node)
                .min()
                .unwrap(),
            1 => &fixture.identities.field_rows()[0].record_node,
            2 => fixture
                .tuples
                .iter()
                .map(|row| &row.structural_node)
                .min()
                .unwrap(),
            3 => fixture
                .units
                .iter()
                .map(|row| &row.structural_node)
                .min()
                .unwrap(),
            4 => {
                &fixture
                    .identities
                    .source_union_rows()
                    .next()
                    .unwrap()
                    .structural_node
            }
            5 => {
                &fixture
                    .identities
                    .source_record_rows()
                    .next()
                    .unwrap()
                    .structural_node
            }
            6 => {
                &fixture
                    .identities
                    .source_tuple_rows()
                    .next()
                    .unwrap()
                    .structural_node
            }
            7 => {
                &fixture
                    .identities
                    .source_enum_rows()
                    .next()
                    .unwrap()
                    .declaration
                    .structural_node
            }
            8 => {
                &fixture
                    .identities
                    .source_collection_rows()
                    .next()
                    .unwrap()
                    .structural_node
            }
            _ => unreachable!(),
        };
        assert_eq!(refusal.at.as_deref(), Some(expected));
        assert_eq!(
            budget.last_success_delta(),
            None,
            "late census refusal clears nested join success"
        );
        assert_eq!(
            budget.first_unpaid(),
            None,
            "structural refusal is not budget exhaustion"
        );
        assert_eq!(partial, before);
    }
}

#[trace("TC-467", "FR-352-AC-24", "FR-352-AC-26")]
#[test]
fn complete_identity_witness_binding_precedes_content_and_missing_families() {
    let fixture = complete_witness_fixture();
    let foreign = complete_witness_fixture();
    assert_eq!(
        fixture.package, foreign.package,
        "independently admitted contents and local selectors collide"
    );
    let subset = acquire(&fixture, Some(5), &mut ample());
    let mut budget = ample();
    let refusal = subset
        .validate_and_rebind(&foreign.package, &mut budget)
        .expect_err("foreign binding wins over missing record");
    assert_eq!(refusal.code, Code::ForeignNode);
    assert_eq!(refusal.form.as_ref(), "selected_identity");
    assert_eq!(
        budget.consumed(),
        NativeJoinCharge {
            work: 1,
            storage: 0
        }
    );
    assert_eq!(budget.last_success_delta(), None);
    assert_eq!(budget.first_unpaid(), None);
    macro_rules! refuses_empty_join {
        ($method:ident) => {
            assert_eq!(
                subset
                    .$method(&foreign.package, [], &mut ample())
                    .unwrap_err()
                    .code,
                Code::ForeignNode
            );
        };
    }
    refuses_empty_join!(with_source_union_rows);
    refuses_empty_join!(with_source_record_rows);
    refuses_empty_join!(with_source_tuple_rows);
    refuses_empty_join!(with_source_enum_rows);
    refuses_empty_join!(with_source_collection_rows);
    assert_eq!(
        subset
            .with_admitted_type_rows(&foreign.package, &[], &[], &mut ample())
            .unwrap_err()
            .code,
        Code::ForeignNode
    );
    assert_eq!(
        subset
            .validate_and_rebind(&fixture.package, &mut ample())
            .unwrap_err()
            .code,
        Code::IncompleteIdentityWitness,
        "no refused join launders the original binding"
    );
}

#[trace("TC-467", "FR-352-AC-25", "FR-352-AC-27")]
#[test]
fn complete_identity_witness_exact_limits_cumulative_charges_and_atomic_refusals() {
    let fixture = complete_witness_fixture();
    let original = fixture.identities.clone();
    let mut ample_budget = ample();
    let expected = original
        .validate_and_rebind(&fixture.package, &mut ample_budget)
        .unwrap();
    let required = ample_budget.consumed();
    assert_eq!(ample_budget.last_success_delta(), Some(required));
    assert!(required.work > 0 && required.storage >= 10);
    for kind in [NativeJoinChargeKind::Work, NativeJoinChargeKind::Storage] {
        let limits = match kind {
            NativeJoinChargeKind::Work => (required.work, u64::MAX),
            NativeJoinChargeKind::Storage => (u64::MAX, required.storage),
        };
        let mut exact = NativeJoinBudget::new(limits.0, limits.1);
        assert_eq!(
            original.validate_and_rebind(&fixture.package, &mut exact),
            Ok(expected.clone())
        );
        assert_eq!(exact.consumed(), required);
        let limits = match kind {
            NativeJoinChargeKind::Work => (required.work - 1, u64::MAX),
            NativeJoinChargeKind::Storage => (u64::MAX, required.storage - 1),
        };
        let mut short = NativeJoinBudget::new(limits.0, limits.1);
        let refusal = original
            .validate_and_rebind(&fixture.package, &mut short)
            .unwrap_err();
        assert_eq!(
            refusal.code,
            match kind {
                NativeJoinChargeKind::Work => Code::ExhaustedWork,
                NativeJoinChargeKind::Storage => Code::ExhaustedStorage,
            }
        );
        assert_eq!(short.first_unpaid().unwrap().kind, kind);
        assert!(short.first_unpaid().unwrap().amount > 0);
        assert_eq!(short.last_success_delta(), None);
        assert_eq!(refusal.work, short.consumed().work);
        assert_eq!(original, fixture.identities);
        let mut repeated = NativeJoinBudget::new(limits.0, limits.1);
        assert_eq!(
            original.validate_and_rebind(&fixture.package.clone(), &mut repeated),
            Err(refusal)
        );
        assert_eq!(repeated, short, "same first native stop and unpaid amount");
        if kind == NativeJoinChargeKind::Storage {
            assert_eq!(
                short.first_unpaid().unwrap().amount,
                10,
                "last returned-handle copy is precharged atomically"
            );
        }
    }
    let acquisition = fixture.budget.consumed();
    let mut shared = fixture.budget.clone();
    assert_eq!(
        original.validate_and_rebind(&fixture.package, &mut shared),
        Ok(expected)
    );
    assert_eq!(shared.last_success_delta(), Some(required));
    assert_eq!(
        shared.consumed(),
        NativeJoinCharge {
            work: acquisition.work + required.work,
            storage: acquisition.storage + required.storage
        }
    );
    let mut short = NativeJoinBudget::new(acquisition.work + required.work - 1, u64::MAX);
    let acquired = acquire(&fixture, None, &mut short);
    let before = short.consumed();
    assert_eq!(
        acquired
            .validate_and_rebind(&fixture.package, &mut short)
            .unwrap_err()
            .code,
        Code::ExhaustedWork
    );
    assert!(short.consumed().work >= before.work);
    assert!(short.consumed().storage >= before.storage);
    assert_eq!(short.last_success_delta(), None);
    assert_eq!(
        short.first_unpaid().unwrap().kind,
        NativeJoinChargeKind::Work
    );
}

#[trace("TC-467", "FR-352-AC-29", "FR-352-AC-24")]
#[test]
fn complete_identity_witness_private_acquisition_is_checked_incremental_and_bounded() {
    let fixture = complete_witness_fixture();
    let stage = SelectedExactIdentities::from_stage_rows(
        &fixture.package,
        fixture.identities.declaration_rows().cloned(),
        fixture.identities.field_rows().iter().cloned(),
        &mut ample(),
    )
    .unwrap();
    let mut budget = ample();
    let first = stage
        .with_admitted_type_rows(&fixture.package, &fixture.tuples, &[], &mut budget)
        .unwrap();
    let first_charge = budget.consumed();
    let both = first
        .with_admitted_type_rows(&fixture.package, &[], &fixture.units, &mut budget)
        .unwrap();
    assert!(
        budget.consumed().work > first_charge.work
            && budget.consumed().storage > first_charge.storage
    );
    assert_eq!(
        both.declaration_rows().count(),
        stage.declaration_rows().count()
    );
    assert_eq!(both.field_rows(), stage.field_rows());
    assert_eq!(
        both.with_admitted_type_rows(&fixture.package, &fixture.tuples, &[], &mut ample()),
        Ok(both.clone())
    );
    assert_eq!(
        both.with_admitted_type_rows(&fixture.package, &[], &fixture.units, &mut ample()),
        Ok(both.clone())
    );
    let duplicate_tuple = [fixture.tuples[0].clone(), fixture.tuples[0].clone()];
    assert_eq!(
        stage
            .with_admitted_type_rows(&fixture.package, &duplicate_tuple, &[], &mut ample())
            .unwrap_err()
            .code,
        Code::ConflictingIdentityWitness
    );
    let duplicate_unit = [fixture.units[0].clone(), fixture.units[0].clone()];
    assert_eq!(
        stage
            .with_admitted_type_rows(&fixture.package, &[], &duplicate_unit, &mut ample())
            .unwrap_err()
            .code,
        Code::ConflictingIdentityWitness
    );
    let mut all_budget = ample();
    let expected = stage
        .with_admitted_type_rows(
            &fixture.package,
            &fixture.tuples,
            &fixture.units,
            &mut all_budget,
        )
        .unwrap();
    let required = all_budget.consumed();
    for (work, storage, code) in [
        (required.work - 1, u64::MAX, Code::ExhaustedWork),
        (u64::MAX, required.storage - 1, Code::ExhaustedStorage),
    ] {
        let mut short = NativeJoinBudget::new(work, storage);
        assert_eq!(
            stage
                .with_admitted_type_rows(
                    &fixture.package,
                    &fixture.tuples,
                    &fixture.units,
                    &mut short
                )
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(short.last_success_delta(), None);
        assert!(short.first_unpaid().is_some());
    }
    let mut exact = NativeJoinBudget::new(required.work, required.storage);
    assert_eq!(
        stage.with_admitted_type_rows(
            &fixture.package,
            &fixture.tuples,
            &fixture.units,
            &mut exact
        ),
        Ok(expected)
    );
    let first_unit = stage
        .with_admitted_type_rows(&fixture.package, &[], &fixture.units[..1], &mut ample())
        .unwrap();
    let original_first = first_unit.clone();
    let mut limit = 0;
    let mut saw_copy = false;
    for _ in 0..64 {
        let mut short = NativeJoinBudget::new(u64::MAX, limit);
        let error = first_unit
            .with_admitted_type_rows(&fixture.package, &[], &fixture.units[1..], &mut short)
            .unwrap_err();
        let denied = short.first_unpaid().unwrap();
        assert_eq!(error.code, Code::ExhaustedStorage);
        if error.form.as_ref() == "admitted_source_unit_copy" {
            assert!(
                denied.amount > 2,
                "nonempty map keys/entries/handles are charged before COW"
            );
            assert_eq!(short.last_success_delta(), None);
            saw_copy = true;
            break;
        }
        limit = short.consumed().storage + u64::try_from(denied.amount).unwrap();
    }
    assert!(saw_copy);
    assert_eq!(
        first_unit, original_first,
        "failed COW publishes no extension"
    );
    let mut compound = fixture.units[0].clone();
    compound.unit_id = quire_exact::UnitId::compound(Vec::new());
    assert_eq!(
        stage
            .with_admitted_type_rows(&fixture.package, &[], &[compound], &mut ample())
            .unwrap_err()
            .code,
        Code::MismatchedIdentityWitness
    );
    let mut changed_tuple = fixture.tuples[0].clone();
    changed_tuple.original_identity = "other/original".into();
    assert_eq!(
        stage
            .with_admitted_type_rows(&fixture.package, &[changed_tuple], &[], &mut ample())
            .unwrap_err()
            .code,
        Code::MismatchedIdentityWitness
    );
    let mut changed_unit = fixture.units[0].clone();
    changed_unit.source_identity = "unselected".into();
    assert_eq!(
        stage
            .with_admitted_type_rows(&fixture.package, &[], &[changed_unit], &mut ample())
            .unwrap_err()
            .code,
        Code::MismatchedIdentityWitness
    );
    let mut foreign = fixture.tuples[0].clone();
    foreign.structural_node.package = "foreign".into();
    assert_eq!(
        stage
            .with_admitted_type_rows(&fixture.package, &[foreign], &[], &mut ample())
            .unwrap_err()
            .code,
        Code::ForeignNode
    );
    let mut conflict = fixture.tuples[0].clone();
    conflict.node_key = match &fixture
        .identities
        .declaration_rows()
        .find(|row| row.form == quire_contract_model::SelectedExactDeclarationForm::RecordValueType)
        .unwrap()
        .exact_identity
    {
        quire_contract_model::SelectedExactIdentity::Record(key) => key.clone(),
        _ => unreachable!(),
    };
    assert_eq!(
        stage
            .with_admitted_type_rows(&fixture.package, &[conflict], &[], &mut ample())
            .unwrap_err()
            .code,
        Code::ConflictingIdentityWitness
    );
}

#[trace("TC-467", "FR-352-AC-22")]
#[test]
fn complete_identity_witness_obligation_free_package_accepts_empty_witness() {
    let package = crate::checked_package_v2_model_fields::integer_add_control();
    let empty = SelectedExactIdentities::empty();
    let rebound = empty.validate_and_rebind(&package, &mut ample()).unwrap();
    assert_eq!(rebound.declaration_rows().count(), 0);
    assert!(rebound.field_rows().is_empty());
    assert_eq!(rebound.source_union_rows().count(), 0);
    assert_eq!(
        rebound.validate_and_rebind(&package.clone(), &mut ample()),
        Ok(rebound.clone())
    );
    let foreign = crate::checked_package_v2_model_fields::integer_add_control();
    assert_eq!(
        rebound
            .validate_and_rebind(&foreign, &mut ample())
            .unwrap_err()
            .code,
        Code::ForeignNode
    );
}
