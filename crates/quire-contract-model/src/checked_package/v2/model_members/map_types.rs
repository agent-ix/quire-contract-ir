// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Package map type admission (FR-154 "Map types"): a map declares a scalar
//! key type, a value type and an entry-count bound.

use super::option_types::{at, reference};
use super::*;

/// Members another kind declares, refused on a map.
const OTHER_KIND_MEMBERS: [&str; 10] = [
    "fields",
    "identityFields",
    "operations",
    "supertypes",
    "clauses",
    "positions",
    "variants",
    "items",
    "inner",
    "target",
];

/// Refuses a `keys` or `bound` member on a sequence, which only a map declares.
pub(super) fn forbid_map_members(
    declared: View<'_>,
    position: usize,
    budget: &mut Budget<'_, '_>,
) -> Result<(), SelectionFailure> {
    for member in ["keys", "bound"] {
        if declared.get(member).is_some() {
            let pointer = JsonPointer::root().key("types").index(position).key(member);
            return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
        }
    }
    Ok(())
}

/// The refusal a sequence element or map value reference draws, if any.
pub(super) fn member_refusal(
    selected: &SelectedTypeRef,
    model: &DomainModel,
    scope: &Scope<'_>,
) -> Option<ModelRefusal> {
    match selected {
        SelectedTypeRef::Map(..) => Some(ModelRefusal::malformed()),
        SelectedTypeRef::Unit(name) if !model.unit_refs.contains_key(name) => {
            Some(ModelRefusal::missing_name())
        }
        SelectedTypeRef::Declared(name) if scope.relationships.contains(name.as_ref()) => {
            Some(ModelRefusal::malformed())
        }
        SelectedTypeRef::Declared(name) if !scope.all_types.contains(name.as_ref()) => {
            Some(ModelRefusal::missing_name())
        }
        _ => None,
    }
}

/// The refusal a key type draws: an unresolved or non-type reference, or a
/// type with no canonical key.
fn key_refusal(
    key: &SelectedTypeRef,
    model: &DomainModel,
    scope: &Scope<'_>,
    declared: &Declared<'_>,
) -> Option<ModelRefusal> {
    if let Some(refusal) = member_refusal(key, model, scope) {
        return Some(refusal);
    }
    let eligible = match key {
        SelectedTypeRef::Native(native) => !matches!(
            native,
            NativeTypeRef::Rational | NativeTypeRef::Decimal | NativeTypeRef::Clock
        ),
        SelectedTypeRef::Unit(_) => true,
        SelectedTypeRef::Map(..) => false,
        SelectedTypeRef::Declared(name) => {
            let name = name.as_ref();
            let row = declared.group(name).first().map(|row| row.2);
            if model.value_types.contains_key(name) {
                row.and_then(|row| row.get("scalar"))
                    .and_then(View::as_str)
                    .is_some_and(|scalar| matches!(scalar, "integer" | "string" | "rational"))
            } else if model.variant_types.contains_key(name) {
                row.and_then(|row| row.get("variants"))
                    .and_then(View::as_array)
                    .is_some_and(|cases| cases.iter().all(|case| case.get("payloadType").is_none()))
            } else if scope.records.contains(name)
                || scope.nominal_objects.contains(name)
                || scope.value_definitions.contains(name)
                || model.tuple_types.contains_key(name)
            {
                false
            } else {
                // An event, state machine, process, persistence interface or
                // namespace is no type at all.
                return Some(ModelRefusal::malformed());
            }
        }
    };
    (!eligible).then(ModelRefusal::ineligible)
}

/// A map's bound: `lower` and an optional `upper`, no other member.
#[ix_trace_rs::string_edge]
fn bound_of(declared: View<'_>) -> Result<MapBound, ModelRefusal> {
    let bound = declared
        .get("bound")
        .and_then(View::as_object)
        .ok_or_else(ModelRefusal::malformed)?;
    if bound
        .keys()
        .any(|member| member != "lower" && member != "upper")
    {
        return Err(ModelRefusal::malformed());
    }
    let lower = bound
        .get("lower")
        .and_then(View::as_u64)
        .ok_or_else(ModelRefusal::malformed)?;
    let upper = match bound.get("upper") {
        None => None,
        Some(upper) => Some(upper.as_u64().ok_or_else(ModelRefusal::malformed)?),
    };
    if upper.map_or(lower > 0, |upper| lower > upper) {
        return Err(ModelRefusal::new(
            CheckedPackageRefusalCode::InvalidModelBinding,
            CheckedPackageRefusalCause::UnpreservedModelMeaning,
        ));
    }
    Ok(MapBound { lower, upper })
}

pub(super) fn read(
    declared: View<'_>,
    position: usize,
    model: &DomainModel,
    scope: &Scope<'_>,
    rows: &Declared<'_>,
    budget: &mut Budget<'_, '_>,
) -> Result<SelectedMap, SelectionFailure> {
    let root = JsonPointer::root().key("types").index(position);
    for member in OTHER_KIND_MEMBERS {
        if declared.get(member).is_some() {
            return Err(at(
                ModelRefusal::malformed(),
                declared,
                root.key(member),
                budget,
            )?
            .into());
        }
    }
    let pointer = root.clone().key("keys");
    let Some(keys_name) = declared.get("keys").and_then(View::as_str) else {
        return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
    };
    let Some(keys) = reference(model, keys_name, budget)? else {
        return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
    };
    if let Some(refusal) = key_refusal(&keys, model, scope, rows) {
        return Err(at(refusal, declared, pointer, budget)?.into());
    }
    let pointer = root.clone().key("values");
    let Some(values_name) = declared.get("values").and_then(View::as_str) else {
        return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
    };
    let Some(values) = reference(model, values_name, budget)? else {
        return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
    };
    if let Some(refusal) = member_refusal(&values, model, scope) {
        return Err(at(refusal, declared, pointer, budget)?.into());
    }
    let bound = match bound_of(declared) {
        Ok(bound) => bound,
        Err(refusal) => {
            return Err(at(refusal, declared, root.key("bound"), budget)?.into());
        }
    };
    Ok(SelectedMap {
        keys,
        keys_name: budget.text(keys_name, SET_FORM)?,
        values,
        bound,
    })
}
