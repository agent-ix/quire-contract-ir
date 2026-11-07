// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic logical storage footprints for native identity joins.

use super::exact_projection::{
    charge_native_work, ExactTypeProjectionError, SelectedExactDeclarationRow,
    SelectedExactFieldRow, SelectedExactIdentity, SelectedSourceCollectionRow,
    SelectedSourceEnumRow, SelectedSourceRecordRow, SelectedSourceTupleRow, SelectedSourceUnionRow,
};
use super::{CheckedNodeId, CheckedNodePathSegment, NativeJoinBudget};
use quire_exact::{
    EffectiveId, EnumShape, MemberId, MemberOwner, QuantityCarrier, UnitId, ValueType,
};
use std::collections::BTreeMap;
use std::sync::Arc;

fn length(value: usize) -> u128 {
    u128::try_from(value).unwrap_or(u128::MAX)
}

fn add(left: u128, right: u128) -> u128 {
    left.saturating_add(right)
}

/// One owned text buffer assembled from borrowed parts, including its container.
pub(super) fn text_parts(parts: &[&str]) -> u128 {
    parts
        .iter()
        .try_fold(1_u128, |units, part| units.checked_add(length(part.len())))
        .unwrap_or(u128::MAX)
}

fn text(value: &str) -> u128 {
    text_parts(&[value])
}

fn text_vec(values: &[Box<str>]) -> u128 {
    values
        .iter()
        .fold(1, |units, value| add(units, text(value)))
}

pub(super) fn node(value: &CheckedNodeId) -> u128 {
    value
        .path
        .iter()
        .fold(add(2, length(value.package.len())), |units, segment| {
            add(
                units,
                match segment {
                    CheckedNodePathSegment::Name(name) => text(name),
                    CheckedNodePathSegment::Node(_)
                    | CheckedNodePathSegment::Index(_)
                    | CheckedNodePathSegment::Operand(_) => 1,
                },
            )
        })
}

pub(super) fn enum_shape(value: &EnumShape) -> u128 {
    value
        .variants()
        .fold(1, |units, variant| add(units, add(1, text(variant.case()))))
}

pub(super) fn preflight_enum_shape(
    value: &EnumShape,
    budget: &mut NativeJoinBudget,
    at: &CheckedNodeId,
) -> Result<(), ExactTypeProjectionError> {
    for variant in value.variants() {
        charge_native_work(
            budget,
            at,
            at,
            "native_enum_footprint",
            (variant.case().len() as u128).saturating_add(2),
        )?;
    }
    Ok(())
}

fn integer(value: &quire_exact::Integer) -> u128 {
    add(1, u128::from(value.magnitude_bits()).div_ceil(8))
}

fn interval(value: &quire_exact::IntegerInterval) -> u128 {
    add(integer(value.lower()), integer(value.upper()))
}

fn effective(value: &EffectiveId) -> u128 {
    value.owner_chain().iter().fold(1, |units, original| {
        add(units, add(text(original.package()), text(original.node())))
    })
}

pub(super) fn effective_identity(value: &EffectiveId) -> u128 {
    effective(value)
}

pub(super) fn member_identity(value: &MemberId) -> u128 {
    add(
        text(value.member()),
        match value.owner() {
            MemberOwner::Node(_) => 1,
            MemberOwner::Effective(id) => effective(id),
        },
    )
}

pub(super) fn unit(value: &UnitId) -> u128 {
    match value {
        UnitId::Declared(_) => 1,
        UnitId::Compound(terms) => terms.iter().fold(1, |units, (_, exponent)| {
            add(units, add(1, integer(exponent)))
        }),
    }
}

/// Pay for the subsequent footprint walk before visiting each nested type.
/// Two steps cover this preflight and the allocation-free footprint walk.
pub(super) fn preflight_value_type(
    value: &ValueType,
    budget: &mut NativeJoinBudget,
    at: &CheckedNodeId,
) -> Result<(), ExactTypeProjectionError> {
    let mut current = value;
    loop {
        charge_native_work(budget, at, at, "native_type_footprint", 2)?;
        match current {
            ValueType::Option(payload) => current = payload,
            ValueType::Collection(collection) => current = collection.element(),
            ValueType::Map(map) => current = map.values(),
            ValueType::Enum(shape) => {
                for variant in shape.variants() {
                    charge_native_work(
                        budget,
                        at,
                        at,
                        "native_enum_footprint",
                        u128::try_from(variant.case().len())
                            .unwrap_or(u128::MAX)
                            .saturating_add(2),
                    )?;
                }
                return Ok(());
            }
            ValueType::Reference(effective) => {
                preflight_effective(effective, budget, at)?;
                return Ok(());
            }
            ValueType::Quantity(unit_id) => {
                preflight_unit(unit_id, budget, at)?;
                return Ok(());
            }
            ValueType::CarriedQuantity(carried) => {
                preflight_unit(carried.unit(), budget, at)?;
                let numeric_work = match carried.carrier() {
                    QuantityCarrier::Decimal(_) | QuantityCarrier::Integer { .. } => 2,
                    QuantityCarrier::Rational(_) => 4,
                };
                charge_native_work(budget, at, at, "native_numeric_footprint", numeric_work)?;
                return Ok(());
            }
            ValueType::BoundedQuantity(bounded) => {
                preflight_unit(bounded.unit(), budget, at)?;
                for _ in bounded.min().into_iter().chain(bounded.max()) {
                    charge_native_work(budget, at, at, "native_quantity_footprint", 2)?;
                }
                return Ok(());
            }
            ValueType::Int(_) | ValueType::Decimal(_) => {
                charge_native_work(budget, at, at, "native_numeric_footprint", 2)?;
                return Ok(());
            }
            ValueType::Rational(_) => {
                charge_native_work(budget, at, at, "native_numeric_footprint", 4)?;
                return Ok(());
            }
            ValueType::Boolean
            | ValueType::Integer
            | ValueType::Float(_)
            | ValueType::Text(_)
            | ValueType::String
            | ValueType::Uuid
            | ValueType::Timestamp
            | ValueType::Composite(_)
            | ValueType::Population(_) => return Ok(()),
        }
    }
}

pub(super) fn preflight_effective(
    value: &EffectiveId,
    budget: &mut NativeJoinBudget,
    at: &CheckedNodeId,
) -> Result<(), ExactTypeProjectionError> {
    for original in value.owner_chain() {
        charge_native_work(
            budget,
            at,
            at,
            "native_effective_footprint",
            u128::try_from(
                original
                    .package()
                    .len()
                    .saturating_add(original.node().len()),
            )
            .unwrap_or(u128::MAX)
            .saturating_add(2),
        )?;
    }
    Ok(())
}

pub(super) fn preflight_unit(
    value: &UnitId,
    budget: &mut NativeJoinBudget,
    at: &CheckedNodeId,
) -> Result<(), ExactTypeProjectionError> {
    if let UnitId::Compound(terms) = value {
        for _ in terms.iter() {
            charge_native_work(budget, at, at, "native_unit_footprint", 2)?;
        }
    }
    Ok(())
}

pub(super) fn value_type(value: &ValueType) -> u128 {
    let mut units = 0_u128;
    let mut at = value;
    loop {
        units = add(units, 1);
        match at {
            ValueType::Int(bounds) => return add(units, interval(bounds)),
            ValueType::Rational(bounds) => {
                return add(
                    units,
                    add(interval(bounds.numerator()), interval(bounds.denominator())),
                );
            }
            ValueType::Decimal(bounds) => {
                return add(units, add(integer(bounds.lower()), integer(bounds.upper())));
            }
            ValueType::Enum(shape) => {
                return shape.variants().fold(add(units, 1), |sum, variant| {
                    add(sum, add(1, text(variant.case())))
                });
            }
            ValueType::Option(payload) => {
                units = add(units, 1);
                at = payload;
            }
            ValueType::Collection(collection) => {
                units = add(units, 2);
                at = collection.element();
            }
            ValueType::Map(map) => {
                units = add(units, 2);
                at = map.values();
            }
            ValueType::BoundedQuantity(bounded) => {
                let bounds = bounded.min().into_iter().chain(bounded.max());
                return bounds.fold(add(units, unit(bounded.unit())), |sum, quantity| {
                    let magnitude = quantity.magnitude();
                    add(
                        sum,
                        add(
                            integer(magnitude.numerator()),
                            integer(magnitude.denominator()),
                        ),
                    )
                });
            }
            ValueType::Quantity(unit_id) => return add(units, unit(unit_id)),
            ValueType::CarriedQuantity(carried) => {
                let carrier = match carried.carrier() {
                    QuantityCarrier::Decimal(bounds) => {
                        add(integer(bounds.lower()), integer(bounds.upper()))
                    }
                    QuantityCarrier::Integer { domain, .. } => interval(domain),
                    QuantityCarrier::Rational(bounds) => {
                        add(interval(bounds.numerator()), interval(bounds.denominator()))
                    }
                };
                return add(units, add(unit(carried.unit()), add(1, carrier)));
            }
            ValueType::Reference(id) => return add(units, effective(id)),
            ValueType::Boolean
            | ValueType::Integer
            | ValueType::String
            | ValueType::Uuid
            | ValueType::Timestamp
            | ValueType::Float(_)
            | ValueType::Text(_)
            | ValueType::Composite(_)
            | ValueType::Population(_) => return units,
        }
    }
}

pub(super) fn stage_declaration(row: &SelectedExactDeclarationRow) -> u128 {
    let base = add(
        add(
            add(2, node(&row.structural_node)),
            text(&row.selected_document),
        ),
        text(&row.original_identity),
    );
    add(
        base,
        match &row.exact_identity {
            SelectedExactIdentity::Object(id) => effective(id),
            SelectedExactIdentity::DeclaredUnit(id) => unit(id),
            SelectedExactIdentity::Record(_) | SelectedExactIdentity::Union(_) => 1,
        },
    )
}

pub(super) fn stage_owner_key(row: &SelectedExactDeclarationRow) -> u128 {
    add(
        add(1, text(&row.selected_document)),
        text(&row.original_identity),
    )
}

pub(super) fn stage_field(row: &SelectedExactFieldRow) -> u128 {
    add(
        add(
            add(add(2, node(&row.record_node)), text(&row.selected_document)),
            text(&row.record_original_identity),
        ),
        add(
            text(&row.field_original_identity),
            text(row.member_id.member()),
        ),
    )
}

pub(super) fn stage_field_key(row: &SelectedExactFieldRow) -> u128 {
    add(
        add(1, node(&row.record_node)),
        text(&row.field_original_identity),
    )
}

pub(super) fn source_union(row: &SelectedSourceUnionRow) -> u128 {
    let mut units = add(4, node(&row.structural_node));
    units = add(units, text(&row.source_authority));
    units = add(units, text(&row.source_identity));
    units = add(units, text_vec(&row.qualified_declaration));
    units = add(units, 1);
    for member in &row.members {
        units = add(
            units,
            add(
                2,
                add(
                    text(member.member.identifier()),
                    text(member.member.variant().case()),
                ),
            ),
        );
        units = add(
            units,
            member
                .payload_nodes
                .iter()
                .fold(1, |sum, id| add(sum, node(id))),
        );
        units = add(
            units,
            member
                .payload
                .iter()
                .fold(1, |sum, value| add(sum, value_type(value))),
        );
    }
    units
}

pub(super) fn source_record(row: &SelectedSourceRecordRow) -> u128 {
    let mut units = add(3, node(&row.structural_node));
    units = add(units, text(&row.source_authority));
    units = add(units, text(&row.source_identity));
    units = add(units, text_vec(&row.qualified_declaration));
    units = add(units, 1);
    for member in &row.members {
        units = add(units, 2);
        units = add(units, text(&member.original_member_identity));
        units = add(units, text(&member.admitted_name));
        units = add(units, node(&member.type_node));
        units = add(units, value_type(&member.value_type));
    }
    units
}

pub(super) fn source_tuple(row: &SelectedSourceTupleRow) -> u128 {
    let mut units = add(3, node(&row.structural_node));
    units = add(units, text(&row.source_authority));
    units = add(units, text(&row.source_identity));
    units = add(units, text_vec(&row.qualified_declaration));
    units = add(units, 1);
    for position in &row.positions {
        units = add(units, 1);
        units = add(units, node(&position.type_node));
        units = add(units, value_type(&position.value_type));
    }
    units
}

pub(super) fn source_enum(row: &SelectedSourceEnumRow) -> u128 {
    let declaration = &row.declaration;
    let mut units = add(4, node(&declaration.structural_node));
    units = add(units, text(&declaration.source_authority));
    units = add(units, text(&declaration.source_identity));
    units = add(units, text_vec(&declaration.qualified_declaration));
    units = declaration
        .shape
        .variants()
        .fold(units, |sum, variant| add(sum, add(1, text(variant.case()))));
    for member in &row.members {
        units = add(units, 3);
        units = add(units, node(&member.structural_node));
        units = add(units, text(&member.source_authority));
        units = add(units, text(&member.source_identity));
        units = add(units, node(&member.declaration_node));
        units = add(units, text_vec(&member.declaration_qualified_name));
        units = add(units, text(&member.case));
        units = add(units, text(member.variant.case()));
    }
    units
}

pub(super) fn source_collection(row: &SelectedSourceCollectionRow) -> u128 {
    add(
        add(
            add(1, node(&row.structural_node)),
            add(node(&row.base_node), node(&row.element_node)),
        ),
        value_type(&row.value_type),
    )
}

/// One new map plus every copied key, entry and shared row handle.
pub(super) fn prior_arc_map<T>(map: &BTreeMap<CheckedNodeId, Arc<T>>) -> u128 {
    map.keys()
        .fold(1, |units, key| add(units, add(2, node(key))))
}

/// One selected value with its nine shared maps/vectors and admission handle.
pub(super) const fn selected_clone() -> u128 {
    10
}

/// Nine newly owned map/vector containers and their nine Arc handles, five
/// temporary uniqueness containers, and the retained admission-binding handle.
pub(super) const fn stage_selection() -> u128 {
    9 * 2 + 5 + 1
}

pub(super) fn runtime_tuple(row: &super::SelectedTupleDeclarationRow) -> u128 {
    add(
        add(
            add(node(&row.structural_node), text(&row.selected_document)),
            text(&row.original_identity),
        ),
        2,
    )
}
pub(super) fn runtime_source_unit(row: &super::SelectedSourceUnitRow) -> u128 {
    add(
        add(
            add(
                add(node(&row.structural_node), text(&row.source_authority)),
                text(&row.source_identity),
            ),
            text_vec(&row.qualified_declaration),
        ),
        add(unit(&row.unit_id), 1),
    )
}
pub(super) fn model_enum_declaration(row: &super::SelectedModelEnumDeclarationRow) -> u128 {
    add(
        add(
            add(node(&row.structural_node), text(&row.selected_document)),
            text(&row.original_identity),
        ),
        2,
    )
}
pub(super) fn model_enum_member(row: &super::SelectedModelEnumMemberRow) -> u128 {
    let base = add(node(&row.structural_node), node(&row.declaration_node));
    [
        &row.selected_document,
        &row.enum_original_identity,
        &row.member_original_identity,
        &row.case,
    ]
    .iter()
    .fold(add(base, 2), |units, value| add(units, text(value)))
}
pub(super) fn model_enum_shape(row: &super::SelectedModelEnumShapeRow) -> u128 {
    let base = add(
        add(
            add(node(&row.structural_node), text(&row.selected_document)),
            text(&row.original_identity),
        ),
        add(enum_shape(&row.shape), 3),
    );
    row.cases.iter().fold(base, |units, case| {
        add(
            units,
            add(
                add(
                    add(node(&case.structural_node), text(&case.case)),
                    text(&case.member_original_identity),
                ),
                add(2, text(case.variant.case())),
            ),
        )
    })
}
