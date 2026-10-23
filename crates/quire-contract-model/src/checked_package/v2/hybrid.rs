//! QSpec FR-481 hybrid node wire shapes and model joins.

use super::{
    CheckedNodeKind, CheckedSemanticNodeV2, HybridForm, ScalarTypeForm, TemporalForm, ValueForm,
    WorkMeter,
};
use crate::checked_package::common::{exact_members, node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, JsonPointer,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(Code::MalformedWire, at)
}

fn members(value: &Value) -> &[Value] {
    value
        .get("members")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn bound(value: &Value, index: usize) -> &Value {
    &members(value)[index]["value"]
}

fn target(value: &Value) -> Option<CheckedNodeId> {
    serde_json::from_value(value.get("target")?.clone()).ok()
}

fn number(value: &Value) -> Option<usize> {
    value.get("value")?.as_str()?.parse().ok()
}

fn node_at<'a>(
    target: &CheckedNodeId,
    nodes: &'a [CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<(usize, &'a CheckedSemanticNodeV2)> {
    let position = *index.get(target)?;
    Some((position, nodes.get(position)?))
}

fn issue(
    code: Code,
    cause: Option<Cause>,
    at: JsonPointer,
    node: &CheckedSemanticNodeV2,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, at, cause, node.node_id.clone())
}

fn term_type(
    key: &CheckedNodeId,
    required: ScalarTypeForm,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> bool {
    let Some((position, node)) = node_at(key, nodes, index) else {
        return false;
    };
    if !matches!(
        kinds.get(position),
        Some(CheckedNodeKind::Expression(_) | CheckedNodeKind::Value(ValueForm::Literal))
    ) {
        return false;
    }
    index
        .get(&node.semantic_type)
        .and_then(|&position| kinds.get(position))
        == Some(&CheckedNodeKind::ScalarType(required))
}

fn rational_or_variable(
    key: &CheckedNodeId,
    variables: &BTreeSet<CheckedNodeId>,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> bool {
    variables.contains(key) || term_type(key, ScalarTypeForm::Rational, nodes, kinds, index)
}

fn numeric_or_boolean_type(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<bool, ValidationFailure> {
    let base = scalar_base_type(type_id, nodes, kinds, index, meter, at)?;
    Ok(base
        .as_ref()
        .and_then(|base| index.get(base))
        .and_then(|&position| kinds.get(position))
        .is_some_and(|kind| {
            matches!(
                kind,
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Boolean
                        | ScalarTypeForm::Integer
                        | ScalarTypeForm::Rational
                        | ScalarTypeForm::Decimal
                        | ScalarTypeForm::Float32
                        | ScalarTypeForm::Float64
                )
            )
        }))
}

fn scalar_base_type(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<Option<CheckedNodeId>, ValidationFailure> {
    let mut current = type_id;
    for _ in 0..=nodes.len() {
        meter.charge(1, || at.clone())?;
        let Some((position, node)) = node_at(current, nodes, index) else {
            return Ok(None);
        };
        match kinds.get(position) {
            Some(CheckedNodeKind::ScalarType(_)) => return Ok(Some(current.clone())),
            Some(CheckedNodeKind::BoundedDomain(_)) => current = &node.semantic_type,
            _ => return Ok(None),
        }
    }
    Ok(None)
}

fn model_pointer(position: usize, binding: usize) -> JsonPointer {
    node_pointer(position)
        .key("body")
        .key("members")
        .index(binding)
        .key("value")
}

fn parameter_name_level(node: &CheckedSemanticNodeV2) -> Option<(&str, usize)> {
    let body = members(&node.body);
    let name = body.first()?.get("value")?.get("value")?.as_str()?;
    let level = number(body.get(1)?.get("value")?)?;
    Some((name, level))
}

struct ReadScope {
    variables: BTreeSet<CheckedNodeId>,
    named_reads: BTreeMap<(String, usize), CheckedNodeId>,
}

fn reads_allowed(
    start: &CheckedNodeId,
    allowed: &ReadScope,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    at: JsonPointer,
    source: &CheckedSemanticNodeV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![start];
    let mut invalid_term = false;
    while let Some(key) = pending.pop() {
        if !seen.insert(key) {
            continue;
        }
        meter.charge(1, || at.clone())?;
        let Some((position, node)) = node_at(key, nodes, index) else {
            return Err(issue(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                at,
                source,
            ));
        };
        match kinds.get(position) {
            Some(CheckedNodeKind::Value(ValueForm::Parameter)) => {
                if !allowed.variables.contains(key) {
                    let expected = parameter_name_level(node).and_then(|(name, level)| {
                        allowed.named_reads.get(&(name.to_owned(), level))
                    });
                    match expected {
                        Some(expected)
                            if scalar_base_type(
                                &node.semantic_type,
                                nodes,
                                kinds,
                                index,
                                meter,
                                &at,
                            )? == Some(expected.clone()) => {}
                        Some(_) => {
                            return Err(issue(
                                Code::IllTyped,
                                Some(Cause::TypeMismatch),
                                at,
                                source,
                            ));
                        }
                        None => {
                            return Err(issue(
                                Code::MissingDeclaration,
                                Some(Cause::MissingName),
                                at,
                                source,
                            ));
                        }
                    }
                }
                // Parameter metadata contains its name and index literals; they
                // are not terms read by the enclosing hybrid expression.
                continue;
            }
            Some(CheckedNodeKind::Function(_)) => {
                // A called function is checked at its call; its own body does not
                // read the enclosing model's parameters.
                if !numeric_or_boolean_type(&node.semantic_type, nodes, kinds, index, meter, &at)? {
                    return Err(issue(Code::IllTyped, Some(Cause::TypeMismatch), at, source));
                }
                let parameters = members(&node.body)
                    .first()
                    .filter(|member| member["name"] == "parameters")
                    .map(|member| members(&member["value"]))
                    .unwrap_or(&[]);
                for parameter in parameters {
                    meter.charge(1, || at.clone())?;
                    if let Some((position, parameter_node)) =
                        target(parameter).and_then(|key| node_at(&key, nodes, index))
                    {
                        if kinds.get(position)
                            == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                            && !numeric_or_boolean_type(
                                &parameter_node.semantic_type,
                                nodes,
                                kinds,
                                index,
                                meter,
                                &at,
                            )?
                        {
                            return Err(issue(
                                Code::IllTyped,
                                Some(Cause::TypeMismatch),
                                at,
                                source,
                            ));
                        }
                    }
                }
                continue;
            }
            Some(CheckedNodeKind::Expression(form)) => {
                let operation = node.body["operation"]["identity"].as_str();
                let admitted = matches!(
                    form,
                    super::ExpressionForm::Conditional | super::ExpressionForm::Call
                ) || matches!(
                    operation,
                    Some(
                        "quire.op.rational.negate"
                            | "quire.op.rational.add"
                            | "quire.op.rational.sub"
                            | "quire.op.rational.mul"
                            | "quire.op.rational.div"
                            | "quire.op.rational.eq"
                            | "quire.op.rational.ne"
                            | "quire.op.rational.lt"
                            | "quire.op.rational.le"
                            | "quire.op.rational.ge"
                            | "quire.op.rational.gt"
                            | "quire.op.boolean.not"
                            | "quire.op.boolean.and"
                            | "quire.op.boolean.or"
                            | "quire.op.boolean.implies"
                    )
                );
                invalid_term |= !admitted;
            }
            Some(CheckedNodeKind::Value(ValueForm::Literal)) => {
                invalid_term |= !matches!(
                    index
                        .get(&node.semantic_type)
                        .and_then(|&position| kinds.get(position)),
                    Some(CheckedNodeKind::ScalarType(
                        ScalarTypeForm::Rational | ScalarTypeForm::Boolean
                    ))
                );
            }
            Some(CheckedNodeKind::Value(_)) => invalid_term = true,
            _ => {}
        }
        pending.extend(node.dependencies.iter());
    }
    if invalid_term {
        return Err(issue(
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            at,
            source,
        ));
    }
    Ok(())
}

/// Joins every hybrid model to its variables, modes, transitions and initial entries.
/// The strict wire and key stages have already authenticated each node body.
pub(super) fn validate_models(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut reached_modes = BTreeSet::new();
    let mut reached_transitions = BTreeSet::new();
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Hybrid(HybridForm::Model)) {
            continue;
        }
        let model = &nodes[position];
        meter.charge(1, || node_pointer(position))?;
        let variables_body = bound(&model.body, 0);
        let modes_body = bound(&model.body, 1);
        let mode_refs = members(modes_body);
        let mut seen_modes = BTreeSet::new();
        for (slot, mode) in mode_refs.iter().enumerate() {
            let at = model_pointer(position, 1).key("members").index(slot);
            let Some(mode_id) = target(mode) else {
                return Err(malformed(at));
            };
            if !seen_modes.insert(mode_id) {
                return Err(issue(Code::InvalidSemanticGraph, None, at, model));
            }
        }
        if members(&model.body).len() == 3 {
            let initial = bound(&model.body, 2);
            if members(initial).len() == 1 {
                let item = &members(initial)[0];
                if let Some(condition) = target(bound(item, 1)) {
                    if number(bound(item, 0)) == Some(0)
                        && node_at(&condition, nodes, index).is_some_and(|(_, node)| {
                            node.body["value"] == Value::Bool(true)
                                && node.body["value_kind"] == "boolean"
                        })
                    {
                        return Err(issue(
                            Code::InvalidSemanticGraph,
                            None,
                            model_pointer(position, 2),
                            model,
                        ));
                    }
                }
            }
        }
        let mut variables = BTreeSet::new();
        let mut variable_names = BTreeSet::new();
        let mut classes = Vec::new();
        for (slot, variable) in members(variables_body).iter().enumerate() {
            let at = model_pointer(position, 0).key("members").index(slot);
            let Some(key) = target(bound(variable, 0)) else {
                return Err(malformed(at));
            };
            let class = bound(variable, 1)["value"].as_str().unwrap_or_default();
            let valid = node_at(&key, nodes, index).is_some_and(|(target_position, node)| {
                kinds.get(target_position) == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                    && index
                        .get(&node.semantic_type)
                        .and_then(|&position| kinds.get(position))
                        == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Rational))
                    && number(bound(&node.body, 1)) == Some(slot)
                    && node.body["members"][0]["value"]["value"]
                        .as_str()
                        .is_some_and(|name| variable_names.insert(name.to_owned()))
            });
            if !valid || !variables.insert(key.clone()) {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    at.key("members").index(0).key("value").key("target"),
                    model,
                ));
            }
            classes.push((
                key,
                class == "clock",
                at.key("members").index(1).key("value").key("value"),
            ));
        }
        let mut rates_by_mode = Vec::new();
        for (slot, mode_ref) in mode_refs.iter().enumerate() {
            let at = model_pointer(position, 1).key("members").index(slot);
            let Some(key) = target(mode_ref) else {
                return Err(malformed(at));
            };
            let Some((mode_position, mode)) = node_at(&key, nodes, index) else {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    at.key("target"),
                    model,
                ));
            };
            let declared = model
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.as_slice())
                .unwrap_or(&[]);
            let mode_name = mode
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.as_slice())
                .unwrap_or(&[]);
            if kinds.get(mode_position) != Some(&CheckedNodeKind::Hybrid(HybridForm::Mode))
                || mode_name.len() != declared.len() + 1
                || !mode_name.starts_with(declared)
                || mode.owner != model.owner
            {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    at.key("target"),
                    model,
                ));
            }
            reached_modes.insert(key);
            let rates = members(bound(&mode.body, 1));
            if rates.len() != variables.len() {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    model_pointer(mode_position, 1),
                    mode,
                ));
            }
            let invariant = target(bound(&mode.body, 0)).expect("strict hybrid reference");
            if !term_type(&invariant, ScalarTypeForm::Boolean, nodes, kinds, index) {
                return Err(issue(
                    Code::IllTyped,
                    Some(Cause::NonBooleanRoot),
                    model_pointer(mode_position, 0).key("target"),
                    mode,
                ));
            }
            let mut rate_keys = Vec::new();
            for (rate_slot, rate) in rates.iter().enumerate() {
                let key = target(rate).expect("strict hybrid reference");
                if !rational_or_variable(&key, &variables, nodes, kinds, index) {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::TypeMismatch),
                        model_pointer(mode_position, 1)
                            .key("members")
                            .index(rate_slot)
                            .key("target"),
                        mode,
                    ));
                }
                rate_keys.push(key);
            }
            rates_by_mode.push(rate_keys);
            for (transition_slot, transition_ref) in
                members(bound(&mode.body, 2)).iter().enumerate()
            {
                let at = model_pointer(mode_position, 2)
                    .key("members")
                    .index(transition_slot);
                let key = target(transition_ref).expect("strict hybrid reference");
                let Some((transition_position, transition)) = node_at(&key, nodes, index) else {
                    return Err(issue(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        at.key("target"),
                        mode,
                    ));
                };
                if kinds.get(transition_position)
                    != Some(&CheckedNodeKind::Hybrid(HybridForm::Transition))
                {
                    return Err(issue(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        at.key("target"),
                        mode,
                    ));
                }
                reached_transitions.insert(key);
                let target_mode = number(bound(&transition.body, 0)).unwrap_or(usize::MAX);
                if target_mode >= mode_refs.len() {
                    return Err(issue(
                        Code::MissingDeclaration,
                        Some(Cause::MissingName),
                        model_pointer(transition_position, 0),
                        transition,
                    ));
                }
                let guard = target(bound(&transition.body, 1)).expect("strict hybrid reference");
                if !term_type(&guard, ScalarTypeForm::Boolean, nodes, kinds, index) {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::NonBooleanRoot),
                        model_pointer(transition_position, 1).key("target"),
                        transition,
                    ));
                }
                let mut assigned = BTreeSet::new();
                let mut bounded = BTreeSet::new();
                for (reset_slot, reset) in members(bound(&transition.body, 2)).iter().enumerate() {
                    let at = model_pointer(transition_position, 2)
                        .key("members")
                        .index(reset_slot);
                    let variable = number(bound(reset, 0)).unwrap_or(usize::MAX);
                    if variable >= variables.len() {
                        return Err(issue(
                            Code::MissingDeclaration,
                            Some(Cause::MissingName),
                            at.key("members").index(0).key("value"),
                            transition,
                        ));
                    }
                    let assignment = members(reset).len() == 2;
                    if (assignment && (assigned.contains(&variable) || bounded.contains(&variable)))
                        || (!assignment && assigned.contains(&variable))
                    {
                        return Err(issue(
                            Code::InvalidPackage,
                            Some(Cause::ConflictingDefinition),
                            at,
                            transition,
                        ));
                    }
                    if assignment {
                        assigned.insert(variable);
                    } else {
                        bounded.insert(variable);
                    }
                    let value = target(bound(reset, 1)).expect("strict hybrid reference");
                    if !rational_or_variable(&value, &variables, nodes, kinds, index) {
                        return Err(issue(
                            Code::IllTyped,
                            Some(Cause::TypeMismatch),
                            model_pointer(transition_position, 2)
                                .key("members")
                                .index(reset_slot)
                                .key("members")
                                .index(1)
                                .key("value")
                                .key("target"),
                            transition,
                        ));
                    }
                }
            }
        }
        if let Some(initial) = members(&model.body).get(2).map(|_| bound(&model.body, 2)) {
            for (slot, item) in members(initial).iter().enumerate() {
                let at = model_pointer(position, 2).key("members").index(slot);
                let mode = number(bound(item, 0)).unwrap_or(usize::MAX);
                if mode >= mode_refs.len() {
                    return Err(issue(
                        Code::MissingDeclaration,
                        Some(Cause::MissingName),
                        at.key("members").index(0).key("value"),
                        model,
                    ));
                }
                let condition = target(bound(item, 1)).expect("strict hybrid reference");
                if !term_type(&condition, ScalarTypeForm::Boolean, nodes, kinds, index) {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::NonBooleanRoot),
                        at.key("members").index(1).key("value").key("target"),
                        model,
                    ));
                }
            }
        }
        // QSpec FR-481 reader check 5: the referenced term and its dependency
        // closure may read only this model's continuous variables.
        let model_scope = ReadScope {
            variables: variables.clone(),
            named_reads: BTreeMap::new(),
        };
        for mode_ref in mode_refs {
            let Some(mode_position) = target(mode_ref).and_then(|key| index.get(&key).copied())
            else {
                continue;
            };
            let mode = &nodes[mode_position];
            for binding_index in [0, 1] {
                let term_refs = if binding_index == 0 {
                    vec![bound(&mode.body, 0)]
                } else {
                    members(bound(&mode.body, 1)).iter().collect()
                };
                for (slot, term) in term_refs.iter().enumerate() {
                    if let Some(key) = target(term) {
                        reads_allowed(
                            &key,
                            &model_scope,
                            nodes,
                            kinds,
                            index,
                            if binding_index == 0 {
                                model_pointer(mode_position, 0)
                            } else {
                                model_pointer(mode_position, 1).key("members").index(slot)
                            },
                            mode,
                            meter,
                        )?;
                    }
                }
            }
            for (transition_slot, transition_ref) in
                members(bound(&mode.body, 2)).iter().enumerate()
            {
                let Some(key) = target(transition_ref) else {
                    continue;
                };
                let Some((transition_position, transition)) = node_at(&key, nodes, index) else {
                    continue;
                };
                if let Some(guard) = target(bound(&transition.body, 1)) {
                    reads_allowed(
                        &guard,
                        &model_scope,
                        nodes,
                        kinds,
                        index,
                        model_pointer(mode_position, 2)
                            .key("members")
                            .index(transition_slot),
                        mode,
                        meter,
                    )?;
                }
                for (reset_slot, reset) in members(bound(&transition.body, 2)).iter().enumerate() {
                    if let Some(value) = target(bound(reset, 1)) {
                        reads_allowed(
                            &value,
                            &model_scope,
                            nodes,
                            kinds,
                            index,
                            model_pointer(transition_position, 2)
                                .key("members")
                                .index(reset_slot),
                            transition,
                            meter,
                        )?;
                    }
                }
            }
        }
        if members(&model.body).len() == 3 {
            for (slot, initial) in members(bound(&model.body, 2)).iter().enumerate() {
                if let Some(key) = target(bound(initial, 1)) {
                    reads_allowed(
                        &key,
                        &model_scope,
                        nodes,
                        kinds,
                        index,
                        model_pointer(position, 2).key("members").index(slot),
                        model,
                        meter,
                    )?;
                }
            }
        }
        for (slot, (_, clock, at)) in classes.iter().enumerate() {
            if *clock
                && rates_by_mode.iter().any(|rates| {
                    node_at(&rates[slot], nodes, index).is_none_or(|(position, rate)| {
                        kinds.get(position) != Some(&CheckedNodeKind::Value(ValueForm::Literal))
                            || rate.body["value"] != "1"
                            || rate.body["value_kind"] != "rational"
                    })
                })
            {
                return Err(issue(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    at.clone(),
                    model,
                ));
            }
        }
        if classes.iter().all(|(_, clock, _)| *clock) {
            return Err(issue(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                model_pointer(position, 0),
                model,
            ));
        }
    }
    for (&key, &position) in index {
        match kinds.get(position) {
            Some(CheckedNodeKind::Hybrid(HybridForm::Mode)) if !reached_modes.contains(key) => {
                return Err(issue(
                    Code::InvalidSemanticGraph,
                    None,
                    node_pointer(position),
                    &nodes[position],
                ));
            }
            Some(CheckedNodeKind::Hybrid(HybridForm::Transition))
                if !reached_transitions.contains(key) =>
            {
                return Err(issue(
                    Code::InvalidSemanticGraph,
                    None,
                    node_pointer(position),
                    &nodes[position],
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Checks temporal clauses whose over parameter is typed by a hybrid model.
/// The ordinary temporal step has already admitted placement and profile fit.
pub(super) fn validate_claims(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::TemporalClause)) {
            continue;
        }
        let claim = &nodes[position];
        meter.charge(1, || node_pointer(position))?;
        let Some(arguments) = claim.body.get("arguments").and_then(Value::as_array) else {
            continue;
        };
        let Some(over) = arguments.first().and_then(target) else {
            continue;
        };
        let Some((_, over_node)) = node_at(&over, nodes, index) else {
            continue;
        };
        let Some((model_position, model)) = node_at(&over_node.semantic_type, nodes, index) else {
            continue;
        };
        if kinds.get(model_position) != Some(&CheckedNodeKind::Hybrid(HybridForm::Model)) {
            continue;
        }
        if arguments.len() != 6 {
            continue;
        }
        let argument_at = |slot| {
            node_pointer(position)
                .key("body")
                .key("arguments")
                .index(slot)
        };
        if !matches!(
            arguments[1]["value"].as_str(),
            Some("model-time" | "model-steps")
        ) {
            return Err(issue(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                argument_at(1),
                claim,
            ));
        }
        let activation = members(&arguments[2]);
        let each = activation
            .first()
            .is_some_and(|kind| kind["name"] == "kind" && kind["value"]["value"] == "each");
        if each {
            let trigger = activation
                .iter()
                .find(|member| member["name"] == "trigger")
                .and_then(|member| target(&member["value"]));
            let valid = trigger
                .as_ref()
                .and_then(|key| node_at(key, nodes, index))
                .is_some_and(|(target_position, node)| {
                    kinds.get(target_position)
                        == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                        && node.semantic_type == model.node_id
                        && parameter_name_level(node).is_some_and(|(_, level)| level == 1)
                });
            if !valid {
                return Err(issue(
                    Code::IllTyped,
                    Some(Cause::TypeMismatch),
                    argument_at(2).key("members"),
                    claim,
                ));
            }
        }
        let variable_entries = members(bound(&model.body, 0));
        let variables = variable_entries
            .iter()
            .filter_map(|entry| target(bound(entry, 0)))
            .collect::<BTreeSet<_>>();
        let mut allowed = ReadScope {
            variables,
            named_reads: BTreeMap::new(),
        };
        if each {
            for (slot, entry) in variable_entries.iter().enumerate() {
                let Some(variable) = target(bound(entry, 0)) else {
                    continue;
                };
                let Some((_, variable_node)) = node_at(&variable, nodes, index) else {
                    continue;
                };
                let Some((name, _)) = parameter_name_level(variable_node) else {
                    continue;
                };
                allowed.named_reads.insert(
                    (name.to_owned(), variable_entries.len() + slot),
                    variable_node.semantic_type.clone(),
                );
            }
        }
        if let Some((slot, guard)) = activation
            .iter()
            .enumerate()
            .find(|(_, member)| member["name"] == "guard")
        {
            if let Some(key) = target(&guard["value"]) {
                let at = argument_at(2).key("members").index(slot);
                if !term_type(&key, ScalarTypeForm::Boolean, nodes, kinds, index) {
                    return Err(issue(Code::IllTyped, Some(Cause::TypeMismatch), at, claim));
                }
                reads_allowed(&key, &allowed, nodes, kinds, index, at, claim, meter)?;
            }
        }
        let captures = members(&arguments[3]);
        for (slot, capture) in captures.iter().enumerate() {
            let Some(value) = target(&capture["value"]) else {
                continue;
            };
            let at = argument_at(3).key("members").index(slot);
            let initializer = node_at(&value, nodes, index);
            if !initializer.is_some_and(|(position, _)| {
                matches!(
                    kinds.get(position),
                    Some(
                        CheckedNodeKind::Expression(_)
                            | CheckedNodeKind::Value(ValueForm::Literal | ValueForm::Parameter)
                    )
                )
            }) {
                return Err(issue(Code::IllTyped, Some(Cause::TypeMismatch), at, claim));
            }
            let Some(initializer_type) = initializer
                .map(|(_, node)| {
                    scalar_base_type(&node.semantic_type, nodes, kinds, index, meter, &at)
                })
                .transpose()?
                .flatten()
            else {
                return Err(issue(Code::IllTyped, Some(Cause::TypeMismatch), at, claim));
            };
            reads_allowed(&value, &allowed, nodes, kinds, index, at, claim, meter)?;
            allowed.named_reads.insert(
                (
                    capture["name"].as_str().unwrap_or_default().to_owned(),
                    2 * variable_entries.len() + slot,
                ),
                initializer_type,
            );
        }
        let mode_keys = members(bound(&model.body, 1))
            .iter()
            .filter_map(target)
            .collect::<Vec<_>>();
        for fair_ref in members(&arguments[4]) {
            let Some(fair_key) = target(fair_ref) else {
                continue;
            };
            let Some((fair_position, fair)) = node_at(&fair_key, nodes, index) else {
                continue;
            };
            let member = &fair.body["operation"]["member"];
            let declaration: Option<CheckedNodeId> =
                serde_json::from_value(member["declaration"].clone()).ok();
            let name = member["name"].as_str().unwrap_or_default();
            let mut resolved = false;
            for mode_key in &mode_keys {
                let Some((_, mode)) = node_at(mode_key, nodes, index) else {
                    continue;
                };
                let mode_name = mode
                    .declaration
                    .as_ref()
                    .and_then(|decl| decl.qualified_name.last())
                    .map(AsRef::as_ref);
                if declaration.as_ref() == Some(&model.node_id)
                    && mode_name == Some(name)
                    && !members(bound(&mode.body, 2)).is_empty()
                {
                    resolved = true;
                }
                if declaration.as_ref() == Some(mode_key) {
                    for transition_ref in members(bound(&mode.body, 2)) {
                        let Some(transition_key) = target(transition_ref) else {
                            continue;
                        };
                        let Some((_, transition)) = node_at(&transition_key, nodes, index) else {
                            continue;
                        };
                        let Some(target_slot) = number(bound(&transition.body, 0)) else {
                            continue;
                        };
                        if mode_keys
                            .get(target_slot)
                            .and_then(|key| node_at(key, nodes, index))
                            .and_then(|(_, target_mode)| target_mode.declaration.as_ref())
                            .and_then(|decl| decl.qualified_name.last())
                            .is_some_and(|part| part.as_ref() == name)
                        {
                            resolved = true;
                        }
                    }
                }
            }
            if !resolved {
                return Err(issue(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    node_pointer(fair_position)
                        .key("body")
                        .key("operation")
                        .key("member")
                        .key("name"),
                    fair,
                ));
            }
        }
        let mut pending = arguments
            .get(5)
            .and_then(target)
            .into_iter()
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        while let Some(formula_key) = pending.pop() {
            if !seen.insert(formula_key.clone()) {
                continue;
            }
            let Some((formula_position, formula)) = node_at(&formula_key, nodes, index) else {
                continue;
            };
            if kinds.get(formula_position)
                != Some(&CheckedNodeKind::Temporal(TemporalForm::Formula))
            {
                continue;
            }
            if formula.body["operation"]["identity"] == "quire.op.temporal.holds" {
                if let Some(operand) = formula.body["arguments"]
                    .as_array()
                    .and_then(|arguments| arguments.first())
                    .and_then(target)
                {
                    reads_allowed(
                        &operand,
                        &allowed,
                        nodes,
                        kinds,
                        index,
                        node_pointer(formula_position),
                        formula,
                        meter,
                    )?;
                }
            }
            pending.extend(formula.dependencies.iter().cloned());
        }
    }
    Ok(())
}

fn aggregate<'a>(value: &'a Value, at: &JsonPointer) -> Result<&'a [Value], ValidationFailure> {
    let object = value.as_object().ok_or_else(|| malformed(at.clone()))?;
    if !exact_members(object, &["term", "members"])
        || object.get("term").and_then(Value::as_str) != Some("aggregate")
    {
        return Err(malformed(at.clone()));
    }
    object
        .get("members")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| malformed(at.clone().key("members")))
}

fn binding<'a>(
    value: &'a Value,
    name: &str,
    at: &JsonPointer,
) -> Result<&'a Value, ValidationFailure> {
    let object = value.as_object().ok_or_else(|| malformed(at.clone()))?;
    if !exact_members(object, &["term", "name", "value"])
        || object.get("term").and_then(Value::as_str) != Some("binding")
        || object.get("name").and_then(Value::as_str) != Some(name)
    {
        return Err(malformed(at.clone()));
    }
    object
        .get("value")
        .ok_or_else(|| malformed(at.clone().key("value")))
}

fn entry<'a>(
    members: &'a [Value],
    index: usize,
    name: &str,
    at: &JsonPointer,
) -> Result<(&'a Value, JsonPointer), ValidationFailure> {
    let member_at = at.clone().key("members").index(index);
    let member = members
        .get(index)
        .ok_or_else(|| malformed(member_at.clone()))?;
    Ok((
        binding(member, name, &member_at)?,
        member_at.clone().key("value"),
    ))
}

fn reference(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    let object = value.as_object().ok_or_else(|| malformed(at.clone()))?;
    if !exact_members(object, &["term", "target"])
        || object.get("term").and_then(Value::as_str) != Some("reference")
    {
        return Err(malformed(at.clone()));
    }
    Ok(())
}

fn literal<'a>(
    value: &'a Value,
    kind: &str,
    at: &JsonPointer,
) -> Result<&'a str, ValidationFailure> {
    let object = value.as_object().ok_or_else(|| malformed(at.clone()))?;
    if !exact_members(object, &["term", "type", "value_kind", "value"])
        || object.get("term").and_then(Value::as_str) != Some("literal")
        || object.get("value_kind").and_then(Value::as_str) != Some(kind)
    {
        return Err(malformed(at.clone()));
    }
    object
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| malformed(at.clone().key("value")))
}

fn integer(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    let text = literal(value, "integer", at)?;
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty()
        || (digits.starts_with('0') && digits.len() > 1)
        || text == "-0"
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(malformed(at.clone().key("value")));
    }
    Ok(())
}

fn list_of_references(
    value: &Value,
    at: &JsonPointer,
    minimum: usize,
) -> Result<(), ValidationFailure> {
    let members = aggregate(value, at)?;
    if members.len() < minimum {
        return Err(malformed(at.clone().key("members")));
    }
    for (index, member) in members.iter().enumerate() {
        reference(member, &at.clone().key("members").index(index))?;
    }
    Ok(())
}

/// The published FR-481 body and occurrence schema, before package identity.
pub(super) fn check_wire(
    node: &CheckedSemanticNodeV2,
    form: HybridForm,
    position: usize,
) -> Result<(), ValidationFailure> {
    let at = node_pointer(position);
    if node.recursion_group.is_some() {
        return Err(malformed(at.clone().key("recursion_group")));
    }
    if node.nominal_identity_preimage.is_some() {
        return Err(malformed(at.clone().key("nominal_identity_preimage")));
    }
    let expected_role = match form {
        HybridForm::Model | HybridForm::Mode => {
            if node.declaration.is_none() {
                return Err(malformed(at.clone().key("declaration")));
            }
            if node.owner.is_none() {
                return Err(malformed(at.clone().key("owner")));
            }
            if node.occurrences.len() != 1 {
                return Err(malformed(at.clone().key("occurrences")));
            }
            CheckedOccurrenceRole::Declaration
        }
        HybridForm::Transition => {
            if node.declaration.is_some() {
                return Err(malformed(at.clone().key("declaration")));
            }
            if node.owner.is_some() {
                return Err(malformed(at.clone().key("owner")));
            }
            if node.occurrences.is_empty() {
                return Err(malformed(at.clone().key("occurrences")));
            }
            CheckedOccurrenceRole::Expression
        }
    };
    for (index, occurrence) in node.occurrences.iter().enumerate() {
        if occurrence.role != expected_role {
            return Err(malformed(
                at.clone().key("occurrences").index(index).key("role"),
            ));
        }
    }
    let body_at = at.clone().key("body");
    let members = aggregate(&node.body, &body_at)?;
    match form {
        HybridForm::Model => {
            if !matches!(members.len(), 2 | 3) {
                return Err(malformed(body_at.clone().key("members")));
            }
            let (variables, variables_at) = entry(members, 0, "variables", &body_at)?;
            let variables = aggregate(variables, &variables_at)?;
            if variables.is_empty() {
                return Err(malformed(variables_at.clone().key("members")));
            }
            for (index, variable) in variables.iter().enumerate() {
                let at = variables_at.clone().key("members").index(index);
                let members = aggregate(variable, &at)?;
                if members.len() != 2 {
                    return Err(malformed(at.clone().key("members")));
                }
                let (parameter, parameter_at) = entry(members, 0, "parameter", &at)?;
                reference(parameter, &parameter_at)?;
                let (class, class_at) = entry(members, 1, "class", &at)?;
                if !matches!(literal(class, "text", &class_at)?, "clock" | "hybrid") {
                    return Err(malformed(class_at.clone().key("value")));
                }
            }
            let (modes, modes_at) = entry(members, 1, "modes", &body_at)?;
            list_of_references(modes, &modes_at, 1)?;
            if members.len() == 3 {
                let (initial, initial_at) = entry(members, 2, "initial", &body_at)?;
                let initial = aggregate(initial, &initial_at)?;
                if initial.is_empty() {
                    return Err(malformed(initial_at.clone().key("members")));
                }
                for (index, item) in initial.iter().enumerate() {
                    let at = initial_at.clone().key("members").index(index);
                    let item = aggregate(item, &at)?;
                    if item.len() != 2 {
                        return Err(malformed(at.clone().key("members")));
                    }
                    let (mode, mode_at) = entry(item, 0, "mode", &at)?;
                    integer(mode, &mode_at)?;
                    let (condition, condition_at) = entry(item, 1, "condition", &at)?;
                    reference(condition, &condition_at)?;
                }
            }
        }
        HybridForm::Mode => {
            if members.len() != 3 {
                return Err(malformed(body_at.clone().key("members")));
            }
            let (invariant, invariant_at) = entry(members, 0, "invariant", &body_at)?;
            reference(invariant, &invariant_at)?;
            let (rates, rates_at) = entry(members, 1, "rates", &body_at)?;
            list_of_references(rates, &rates_at, 1)?;
            let (transitions, transitions_at) = entry(members, 2, "transitions", &body_at)?;
            list_of_references(transitions, &transitions_at, 0)?;
        }
        HybridForm::Transition => {
            if members.len() != 3 {
                return Err(malformed(body_at.clone().key("members")));
            }
            let (target, target_at) = entry(members, 0, "target", &body_at)?;
            integer(target, &target_at)?;
            let (guard, guard_at) = entry(members, 1, "guard", &body_at)?;
            reference(guard, &guard_at)?;
            let (resets, resets_at) = entry(members, 2, "resets", &body_at)?;
            for (index, reset) in aggregate(resets, &resets_at)?.iter().enumerate() {
                let at = resets_at.clone().key("members").index(index);
                let reset = aggregate(reset, &at)?;
                if !matches!(reset.len(), 2 | 3) {
                    return Err(malformed(at.clone().key("members")));
                }
                let (variable, variable_at) = entry(reset, 0, "variable", &at)?;
                integer(variable, &variable_at)?;
                let (value, value_at) = entry(reset, 1, "value", &at)?;
                reference(value, &value_at)?;
                if reset.len() == 3 {
                    let (relation, relation_at) = entry(reset, 2, "relation", &at)?;
                    if !matches!(
                        literal(relation, "text", &relation_at)?,
                        "ne" | "lt" | "le" | "ge" | "gt"
                    ) {
                        return Err(malformed(relation_at.clone().key("value")));
                    }
                }
            }
        }
    }
    Ok(())
}
