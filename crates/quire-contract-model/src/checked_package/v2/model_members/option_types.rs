// SPDX-License-Identifier: AGPL-3.0-or-later
//! Named option admission and selected-document refusal locations (FR-356).

use super::*;

fn at(
    refusal: ModelRefusal,
    declaration: &Value,
    pointer: JsonPointer,
    budget: &mut Budget<'_>,
) -> Result<SelectionRefusal, ValidationFailure> {
    let mut selected = SelectionRefusal::located(refusal, declaration, budget)?;
    selected.document_pointer = Some(pointer);
    Ok(selected)
}

/// Resolve only the existing typeRef forms, charging every followed alias.
fn reference(
    model: &DomainModel,
    text: &str,
    budget: &mut Budget<'_>,
) -> Result<Option<SelectedTypeRef>, ValidationFailure> {
    let mut current = text;
    let mut seen = BTreeSet::new();
    while let Some(target) = model.aliases.get(current) {
        budget.charge(1)?;
        if !seen.insert(current) {
            return Ok(None);
        }
        current = target;
    }
    if !valid_semantic_identity(current) {
        return Ok(None);
    }
    Ok(SelectedTypeRef::from_admitted_name(current))
}

pub(super) fn read(
    declared: &Value,
    kind: meaning::Kind,
    position: usize,
    model: &DomainModel,
    scope: &Scope<'_>,
    budget: &mut Budget<'_>,
) -> Result<SelectedTypeDefinition, SelectionFailure> {
    let root = JsonPointer::root().key("types").index(position);
    let member = match kind {
        meaning::Kind::OptionType => "inner",
        meaning::Kind::ReferenceType => "target",
        _ => {
            return Err(at(
                ModelRefusal::malformed(),
                declared,
                root.key("kind"),
                budget,
            )?
            .into())
        }
    };
    if kind == meaning::Kind::OptionType {
        for forbidden in [
            "fields",
            "identityFields",
            "operations",
            "supertypes",
            "clauses",
            "positions",
            "variants",
            "items",
            "values",
            "target",
        ] {
            if declared.get(forbidden).is_some() {
                return Err(at(
                    ModelRefusal::malformed(),
                    declared,
                    root.clone().key(forbidden),
                    budget,
                )?
                .into());
            }
        }
        if declared
            .get("constraints")
            .is_some_and(|value| !value.as_array().is_some_and(Vec::is_empty))
        {
            return Err(at(
                ModelRefusal::malformed(),
                declared,
                root.clone().key("constraints"),
                budget,
            )?
            .into());
        }
    }
    let pointer = root.key(member);
    let Some(inner) = declared.get(member).and_then(Value::as_str) else {
        return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
    };
    let Some(selected) = reference(model, inner, budget)? else {
        return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
    };
    let failure = match &selected {
        SelectedTypeRef::Map(..) => Some(ModelRefusal::malformed()),
        SelectedTypeRef::Unit(name) if !model.unit_refs.contains_key(name) => {
            Some(ModelRefusal::missing_name())
        }
        SelectedTypeRef::Declared(name) if !scope.all_types.contains(name.as_ref()) => {
            Some(ModelRefusal::missing_name())
        }
        _ => None,
    };
    if let Some(refusal) = failure {
        return Err(at(refusal, declared, pointer, budget)?.into());
    }
    if kind == meaning::Kind::ReferenceType {
        if !matches!(&selected,SelectedTypeRef::Declared(name) if scope.objects.contains(name.as_ref()))
        {
            return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
        }
        return Ok(SelectedTypeDefinition::Reference(selected));
    }
    let value_kind = match &selected {
        SelectedTypeRef::Native(NativeTypeRef::Clock) => false,
        SelectedTypeRef::Native(_) | SelectedTypeRef::Unit(_) => true,
        SelectedTypeRef::Map(..) => false,
        SelectedTypeRef::Declared(name) => {
            !scope.option_types.contains(name.as_ref())
                && (scope.value_definitions.contains(name.as_ref())
                    || scope.records.contains(name.as_ref())
                    || model.value_types.contains_key(name)
                    || model.tuple_types.contains_key(name)
                    || model.variant_types.contains_key(name)
                    || (scope.objects.contains(name.as_ref())
                        && !scope.record_field_forbidden.contains(name.as_ref())))
        }
    };
    if !value_kind {
        return Err(at(ModelRefusal::malformed(), declared, pointer, budget)?.into());
    }
    Ok(SelectedTypeDefinition::Option(selected))
}

pub(super) fn check_constraints(
    document: &Value,
    model: &DomainModel,
    scope: &Scope<'_>,
    budget: &mut Budget<'_>,
) -> Result<(), SelectionFailure> {
    for (position, constraint) in list(document, "constraints")?.iter().enumerate() {
        budget.charge(1)?;
        let Some(applies_to) = constraint.get("appliesTo").and_then(Value::as_str) else {
            continue;
        };
        if matches!(reference(model,applies_to,budget)?,Some(SelectedTypeRef::Declared(name)) if scope.option_types.contains(name.as_ref()))
        {
            let pointer = JsonPointer::root()
                .key("constraints")
                .index(position)
                .key("appliesTo");
            return Err(at(ModelRefusal::malformed(), constraint, pointer, budget)?.into());
        }
    }
    Ok(())
}
