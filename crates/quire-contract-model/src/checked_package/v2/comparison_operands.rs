// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Typed, clause-context operands of admitted direct integer comparisons.

use super::model_fields::OperationTypes;
use super::operation_catalog::operation_catalog;
use super::structural::{
    reference_type_target_typed, typed_anchor_body, typed_clause_body, typed_reference,
};
use super::{
    ApplicationOperator, CheckedMemberType, CheckedNodeId, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedPackageV2, CheckedScalarOperandRange, CheckedSemanticNodeV2,
    CheckedTerm, LiteralKind, StateClauseKind,
};
use serde_json::Value;
use thiserror::Error;

/// The six admitted binary integer comparison identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedComparisonOperator {
    /// Equal.
    Eq,
    /// Unequal.
    Ne,
    /// Less than.
    Lt,
    /// Less than or equal.
    Le,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Ge,
}

impl CheckedComparisonOperator {
    #[ix_trace_rs::string_edge]
    fn from_identity(identity: &str) -> Option<Self> {
        match identity {
            "quire.op.integer.eq" => Some(Self::Eq),
            "quire.op.integer.ne" => Some(Self::Ne),
            "quire.op.integer.lt" => Some(Self::Lt),
            "quire.op.integer.le" => Some(Self::Le),
            "quire.op.integer.gt" => Some(Self::Gt),
            "quire.op.integer.ge" => Some(Self::Ge),
            _ => None,
        }
    }
}

/// The checked clause kind that gives a direct state read its observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedComparisonClauseKind {
    /// An invariant.
    Invariant,
    /// A precondition.
    Precondition,
    /// A postcondition.
    Postcondition,
}

impl From<StateClauseKind> for CheckedComparisonClauseKind {
    fn from(kind: StateClauseKind) -> Self {
        match kind {
            StateClauseKind::Invariant => Self::Invariant,
            StateClauseKind::Precondition => Self::Precondition,
            StateClauseKind::Postcondition => Self::Postcondition,
        }
    }
}

/// Observation derived from one operand path through the supplied clause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedComparisonObservation {
    /// Current state or an operation input.
    Current,
    /// The state before a postcondition operation.
    Pre,
    /// The state after a postcondition operation.
    Post,
}

/// The admitted source of one integer read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedComparisonRead {
    /// A selected model field reached from this clause's self parameter.
    StateField {
        /// The record.project application node.
        project: CheckedNodeId,
        /// The selected model object declaration.
        declaration: CheckedNodeId,
        /// The declared field name.
        field: Box<str>,
    },
    /// An operation parameter in the checked clause signature.
    OperationInput {
        /// The parameter's checked graph node.
        parameter: CheckedNodeId,
        /// Position among operation inputs, starting at zero.
        ordinal: u64,
    },
}

/// Identity of an operand without assigning a child source occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedComparisonOperandIdentity {
    /// A referenced graph child.
    GraphChild(CheckedNodeId),
    /// An inline term qualified by the supplied claim and argument position.
    InlineLiteral {
        /// The comparison application.
        comparison: CheckedNodeId,
        /// The containing clause.
        clause: CheckedNodeId,
        /// The authentic supplied clause occurrence.
        claim: CheckedOccurrence,
        /// Authored comparison argument position.
        ordinal: u64,
    },
}

/// A read or a literal in one direct comparison argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedComparisonOperandKind {
    /// A checked model or operation read.
    Read {
        /// The selected read declaration.
        provenance: CheckedComparisonRead,
        /// The clause-context observation.
        observation: CheckedComparisonObservation,
    },
    /// An exact integer literal.
    Literal,
}

/// One exact finite operand in authored order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedComparisonOperand {
    /// Zero-based comparison argument position.
    pub ordinal: u64,
    /// Graph or inline identity.
    pub identity: CheckedComparisonOperandIdentity,
    /// Read provenance or literal classification.
    pub kind: CheckedComparisonOperandKind,
    /// Inclusive exact bounds.
    pub range: CheckedScalarOperandRange,
}

/// A direct comparison and the context in which its reads occur.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedClauseComparison {
    /// The supplied clause node.
    pub clause: CheckedNodeId,
    /// The supplied authentic claim occurrence.
    pub claim: CheckedOccurrence,
    /// The checked clause kind.
    pub kind: CheckedComparisonClauseKind,
    /// The referenced comparison node.
    pub comparison: CheckedNodeId,
    /// Closed comparison operation.
    pub operator: CheckedComparisonOperator,
    /// Exactly two operands in authored order.
    pub operands: [CheckedComparisonOperand; 2],
}

/// The first reason a direct clause comparison cannot be projected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedComparisonRefusal {
    /// Supplied clause id is absent.
    UnknownClause,
    /// Supplied node has the wrong form.
    NotStateClause,
    /// Supplied claim is absent.
    MissingClaimOccurrence,
    /// Clause kind is outside the eligible set.
    UnsupportedClauseKind,
    /// Condition is not a reference to a present comparison.
    MissingComparison,
    /// Comparison operation is not catalogued.
    UnknownOperator,
    /// Catalogued operation is outside the six comparisons.
    IneligibleOperator,
    /// Comparison body has invalid arity or shape.
    InvalidComparisonShape,
    /// An argument is not an eligible direct read or integer literal.
    IneligibleOperand,
    /// A referenced argument node is missing.
    MissingChild,
    /// Neither argument is a read.
    NoRead,
    /// Selected model type or finite bounds are unavailable.
    MissingRange,
    /// Selected integer type has no finite bounds.
    UnboundedRange,
    /// An integer literal is outside i128.
    RangeOutOfI128,
}

/// A structural location relative to the supplied clause condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedComparisonPath {
    /// The supplied clause node.
    Clause,
    /// Its Boolean condition reference.
    Condition,
    /// The referenced comparison application.
    Comparison,
    /// One comparison argument, including its nested read path.
    Operand(u8),
}

/// Typed refusal with the caller's clause context and the last reached child.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{kind:?} at {path:?}")]
pub struct CheckedComparisonError {
    /// Closed refusal cause.
    pub kind: CheckedComparisonRefusal,
    /// Supplied clause id.
    pub clause: CheckedNodeId,
    /// Supplied claim occurrence.
    pub claim: CheckedOccurrence,
    /// Deterministic clause/condition/argument path.
    pub path: CheckedComparisonPath,
    /// Reached graph child, when one was available.
    pub child: Option<CheckedNodeId>,
}

struct Context<'a> {
    package: &'a CheckedPackageV2,
    clause: &'a CheckedNodeId,
    claim: &'a CheckedOccurrence,
    kind: CheckedComparisonClauseKind,
    parameters: Vec<CheckedNodeId>,
    operation: Option<&'a OperationTypes>,
}

impl<'a> Context<'a> {
    fn error(
        &self,
        kind: CheckedComparisonRefusal,
        path: CheckedComparisonPath,
        child: Option<CheckedNodeId>,
    ) -> CheckedComparisonError {
        CheckedComparisonError {
            kind,
            clause: self.clause.clone(),
            claim: self.claim.clone(),
            path,
            child,
        }
    }

    fn node(&self, id: &CheckedNodeId) -> Option<&'a CheckedSemanticNodeV2> {
        self.package.node(id)
    }
}

impl CheckedPackageV2 {
    /// Projects a clause's direct binary integer comparison into typed reads and literals.
    pub fn direct_integer_comparison(
        &self,
        clause: &CheckedNodeId,
        claim: &CheckedOccurrence,
    ) -> Result<CheckedClauseComparison, CheckedComparisonError> {
        use CheckedComparisonPath as Path;
        use CheckedComparisonRefusal as Refusal;
        let mut context = Context {
            package: self,
            clause,
            claim,
            kind: CheckedComparisonClauseKind::Invariant,
            parameters: Vec::new(),
            operation: None,
        };
        let node = context
            .node(clause)
            .ok_or_else(|| context.error(Refusal::UnknownClause, Path::Clause, None))?;
        if node.node_tag.as_ref() != "state" || node.semantic_form.as_ref() != "state_clause" {
            return Err(context.error(Refusal::NotStateClause, Path::Clause, Some(clause.clone())));
        }
        if claim.role != CheckedOccurrenceRole::Claim || !node.occurrences.contains(claim) {
            return Err(context.error(
                Refusal::MissingClaimOccurrence,
                Path::Clause,
                Some(clause.clone()),
            ));
        }
        if self
            .term(clause)
            .and_then(|term| match term {
                CheckedTerm::Application { operation, .. } => operation.pointer("/member/clause"),
                _ => None,
            })
            .and_then(Value::as_str)
            .is_some_and(|kind| StateClauseKind::from_wire(kind).is_none())
        {
            return Err(context.error(
                Refusal::UnsupportedClauseKind,
                Path::Clause,
                Some(clause.clone()),
            ));
        }
        let body = self
            .node_index
            .get(clause)
            .and_then(|&position| self.terms.get(position))
            .and_then(typed_clause_body)
            .ok_or_else(|| {
                context.error(Refusal::NotStateClause, Path::Clause, Some(clause.clone()))
            })?;
        context.kind = body.clause.into();
        context.parameters = body.parameters;
        if context.kind != CheckedComparisonClauseKind::Invariant {
            context.operation = context
                .node(&body.anchor)
                .and_then(|anchor| self.node_index.get(&anchor.node_id).copied())
                .and_then(|position| self.terms.get(position))
                .and_then(typed_anchor_body)
                .and_then(|anchor| {
                    context.node(&anchor.context).and_then(|owner| {
                        self.term(&owner.node_id).and_then(|term| {
                            self.models.operation_types(owner, term, anchor.operation)
                        })
                    })
                });
        }
        let condition = self
            .term(clause)
            .and_then(|term| match term {
                CheckedTerm::Application { arguments, .. } => arguments.get(2),
                _ => None,
            })
            .and_then(typed_reference)
            .cloned()
            .ok_or_else(|| context.error(Refusal::MissingComparison, Path::Condition, None))?;
        context.node(&condition).ok_or_else(|| {
            context.error(
                Refusal::MissingComparison,
                Path::Condition,
                Some(condition.clone()),
            )
        })?;
        let CheckedTerm::Application {
            operator: operator_class,
            operation,
            arguments,
            ..
        } = self.term(&condition).ok_or_else(|| {
            context.error(
                Refusal::InvalidComparisonShape,
                Path::Comparison,
                Some(condition.clone()),
            )
        })?
        else {
            return Err(context.error(
                Refusal::InvalidComparisonShape,
                Path::Comparison,
                Some(condition.clone()),
            ));
        };
        let identity = operation
            .get("identity")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                context.error(
                    Refusal::UnknownOperator,
                    Path::Comparison,
                    Some(condition.clone()),
                )
            })?;
        if operation_catalog().entry(identity).is_none() {
            return Err(context.error(
                Refusal::UnknownOperator,
                Path::Comparison,
                Some(condition.clone()),
            ));
        }
        let operator = CheckedComparisonOperator::from_identity(identity).ok_or_else(|| {
            context.error(
                Refusal::IneligibleOperator,
                Path::Comparison,
                Some(condition.clone()),
            )
        })?;
        if *operator_class != ApplicationOperator::Binary {
            return Err(context.error(
                Refusal::InvalidComparisonShape,
                Path::Comparison,
                Some(condition.clone()),
            ));
        }
        let [first, second] = arguments.as_slice() else {
            return Err(context.error(
                Refusal::InvalidComparisonShape,
                Path::Comparison,
                Some(condition.clone()),
            ));
        };
        let first = project_operand(&context, &condition, first, 0)?;
        let second = project_operand(&context, &condition, second, 1)?;
        if matches!(first.kind, CheckedComparisonOperandKind::Literal)
            && matches!(second.kind, CheckedComparisonOperandKind::Literal)
        {
            return Err(context.error(Refusal::NoRead, Path::Comparison, Some(condition.clone())));
        }
        Ok(CheckedClauseComparison {
            clause: clause.clone(),
            claim: claim.clone(),
            kind: context.kind,
            comparison: condition,
            operator,
            operands: [first, second],
        })
    }
}

fn project_operand(
    context: &Context<'_>,
    comparison: &CheckedNodeId,
    argument: &CheckedTerm,
    ordinal: u8,
) -> Result<CheckedComparisonOperand, CheckedComparisonError> {
    use CheckedComparisonOperandIdentity as Identity;
    use CheckedComparisonOperandKind as Kind;
    use CheckedComparisonPath as Path;
    use CheckedComparisonRefusal as Refusal;
    let path = Path::Operand(ordinal);
    let fail = |kind, child| context.error(kind, path, child);
    let (identity, kind, range) = match argument {
        CheckedTerm::Literal {
            value_kind, value, ..
        } => {
            let range = typed_literal_range(*value_kind, value).map_err(|kind| fail(kind, None))?;
            (
                Identity::InlineLiteral {
                    comparison: comparison.clone(),
                    clause: context.clause.clone(),
                    claim: context.claim.clone(),
                    ordinal: u64::from(ordinal),
                },
                Kind::Literal,
                range,
            )
        }
        CheckedTerm::Reference { target } => {
            let id = target.clone();
            let child = context
                .node(&id)
                .ok_or_else(|| fail(Refusal::MissingChild, Some(id.clone())))?;
            let (kind, range) = if child.node_tag.as_ref() == "value"
                && child.semantic_form.as_ref() == "literal"
            {
                let range = match context.package.term(&id) {
                    Some(CheckedTerm::Literal {
                        value_kind, value, ..
                    }) => typed_literal_range(*value_kind, value),
                    _ => Err(Refusal::IneligibleOperand),
                };
                (
                    Kind::Literal,
                    range.map_err(|kind| fail(kind, Some(id.clone())))?,
                )
            } else {
                read_range(context, child, path)?
            };
            (Identity::GraphChild(id), kind, range)
        }
        _ => return Err(fail(Refusal::IneligibleOperand, None)),
    };
    Ok(CheckedComparisonOperand {
        ordinal: u64::from(ordinal),
        identity,
        kind,
        range,
    })
}

fn read_range(
    context: &Context<'_>,
    child: &CheckedSemanticNodeV2,
    path: CheckedComparisonPath,
) -> Result<(CheckedComparisonOperandKind, CheckedScalarOperandRange), CheckedComparisonError> {
    use CheckedComparisonObservation as Observation;
    use CheckedComparisonOperandKind as Kind;
    use CheckedComparisonRead as Read;
    use CheckedComparisonRefusal as Refusal;
    let fail = |kind| context.error(kind, path, Some(child.node_id.clone()));
    if let Some(position) = context
        .parameters
        .iter()
        .position(|id| id == &child.node_id)
    {
        if position == 0
            || child.node_tag.as_ref() != "value"
            || child.semantic_form.as_ref() != "parameter"
        {
            return Err(fail(Refusal::IneligibleOperand));
        }
        let operation = context
            .operation
            .ok_or_else(|| fail(Refusal::MissingRange))?;
        let result_offset = usize::from(
            context.kind == CheckedComparisonClauseKind::Postcondition && operation.has_result,
        );
        let ordinal = position
            .checked_sub(1 + result_offset)
            .ok_or_else(|| fail(Refusal::IneligibleOperand))?;
        let ty = operation
            .parameters
            .get(ordinal)
            .and_then(Option::as_ref)
            .ok_or_else(|| fail(Refusal::MissingRange))?;
        let range = member_range(ty).map_err(fail)?;
        return Ok((
            Kind::Read {
                provenance: Read::OperationInput {
                    parameter: child.node_id.clone(),
                    ordinal: u64::try_from(ordinal).map_err(|_| fail(Refusal::MissingRange))?,
                },
                observation: Observation::Current,
            },
            range,
        ));
    }
    let (project, pre) = if operation_identity(context.package, child) == Some("quire.op.state.pre")
    {
        if context.kind != CheckedComparisonClauseKind::Postcondition {
            return Err(fail(Refusal::IneligibleOperand));
        }
        let target = one_reference(context.package, child)
            .ok_or_else(|| fail(Refusal::IneligibleOperand))?;
        (
            context
                .node(&target)
                .ok_or_else(|| context.error(Refusal::MissingChild, path, Some(target.clone())))?,
            true,
        )
    } else {
        (child, false)
    };
    if operation_identity(context.package, project) != Some("quire.op.record.project") {
        return Err(fail(Refusal::IneligibleOperand));
    }
    let deref_id =
        one_reference(context.package, project).ok_or_else(|| fail(Refusal::IneligibleOperand))?;
    let deref = context
        .node(&deref_id)
        .ok_or_else(|| context.error(Refusal::MissingChild, path, Some(deref_id.clone())))?;
    if operation_identity(context.package, deref) != Some("quire.op.model.deref")
        || one_reference(context.package, deref).as_ref() != context.parameters.first()
    {
        return Err(fail(Refusal::IneligibleOperand));
    }
    let Some(CheckedTerm::Application {
        operation,
        result_type,
        ..
    }) = context.package.term(&project.node_id)
    else {
        return Err(fail(Refusal::IneligibleOperand));
    };
    let member = operation
        .get("member")
        .ok_or_else(|| fail(Refusal::IneligibleOperand))?;
    if member.get("kind").and_then(Value::as_str) != Some("field") {
        return Err(fail(Refusal::IneligibleOperand));
    }
    let declaration: CheckedNodeId = serde_json::from_value(
        member
            .get("declaration")
            .cloned()
            .ok_or_else(|| fail(Refusal::IneligibleOperand))?,
    )
    .map_err(|_| fail(Refusal::IneligibleOperand))?;
    let self_type = context
        .parameters
        .first()
        .and_then(|id| context.node(id))
        .and_then(|parameter| context.node(&parameter.semantic_type))
        .and_then(reference_type_target_typed);
    if self_type.as_ref() != Some(&declaration)
        || deref.semantic_type != declaration
        || result_type != &project.semantic_type
    {
        return Err(fail(Refusal::IneligibleOperand));
    }
    let field = member
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| fail(Refusal::IneligibleOperand))?;
    let declared = context
        .node(&declaration)
        .ok_or_else(|| context.error(Refusal::MissingChild, path, Some(declaration.clone())))?;
    let ty = context
        .package
        .models
        .field_type(
            declared,
            context
                .package
                .term(&declared.node_id)
                .ok_or_else(|| fail(Refusal::MissingRange))?,
            field,
        )
        .flatten()
        .ok_or_else(|| fail(Refusal::MissingRange))?;
    let range = member_range(&ty).map_err(fail)?;
    let observation = if pre {
        Observation::Pre
    } else if context.kind == CheckedComparisonClauseKind::Postcondition {
        Observation::Post
    } else {
        Observation::Current
    };
    Ok((
        Kind::Read {
            provenance: Read::StateField {
                project: project.node_id.clone(),
                declaration,
                field: field.into(),
            },
            observation,
        },
        range,
    ))
}

fn operation_identity<'a>(
    package: &'a CheckedPackageV2,
    node: &CheckedSemanticNodeV2,
) -> Option<&'a str> {
    let CheckedTerm::Application { operation, .. } = package.term(&node.node_id)? else {
        return None;
    };
    operation.get("identity").and_then(Value::as_str)
}

fn one_reference(
    package: &CheckedPackageV2,
    node: &CheckedSemanticNodeV2,
) -> Option<CheckedNodeId> {
    let CheckedTerm::Application { arguments, .. } = package.term(&node.node_id)? else {
        return None;
    };
    let [argument] = arguments.as_slice() else {
        return None;
    };
    typed_reference(argument).cloned()
}

fn member_range(
    ty: &CheckedMemberType,
) -> Result<CheckedScalarOperandRange, CheckedComparisonRefusal> {
    match ty {
        CheckedMemberType::IntRange { lower, upper } => Ok(CheckedScalarOperandRange {
            lower: *lower,
            upper: *upper,
        }),
        CheckedMemberType::Integer => Err(CheckedComparisonRefusal::UnboundedRange),
        _ => Err(CheckedComparisonRefusal::MissingRange),
    }
}

#[cfg(test)]
fn literal_range(term: &Value) -> Result<CheckedScalarOperandRange, CheckedComparisonRefusal> {
    use CheckedComparisonRefusal as Refusal;
    if term.get("value_kind").and_then(Value::as_str) != Some("integer") {
        return Err(Refusal::IneligibleOperand);
    }
    integer_value_range(term.get("value").ok_or(Refusal::IneligibleOperand)?)
}

fn typed_literal_range(
    kind: LiteralKind,
    value: &Value,
) -> Result<CheckedScalarOperandRange, CheckedComparisonRefusal> {
    if kind != LiteralKind::Integer {
        return Err(CheckedComparisonRefusal::IneligibleOperand);
    }
    integer_value_range(value)
}

fn integer_value_range(
    value: &Value,
) -> Result<CheckedScalarOperandRange, CheckedComparisonRefusal> {
    use CheckedComparisonRefusal as Refusal;
    let text = value.as_str().ok_or(Refusal::IneligibleOperand)?;
    let value = text.parse::<i128>().map_err(|_| Refusal::RangeOutOfI128)?;
    Ok(CheckedScalarOperandRange {
        lower: value,
        upper: value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::{CheckedPackageWireV2, CheckedTerm, RetainedModels};
    use ix_trace_rs::trace;
    use serde_json::json;

    fn id(digit: char) -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": digit.to_string().repeat(64)})
    }

    fn node(digit: char, tag: &str, form: &str, body: Value, role: &str) -> Value {
        json!({
            "node_id": id(digit), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form, "semantic_type": id('1'),
            "dependencies": [], "occurrences": [{"role": role, "ordinal": 0}],
            "body": body,
        })
    }

    fn reference(digit: char) -> Value {
        json!({"term": "reference", "target": id(digit)})
    }

    fn literal(value: &str) -> Value {
        json!({"term": "literal", "type": id('1'), "value_kind": "integer", "value": value})
    }

    /// A synthetic post-admission graph for first-defect and panic-surface probes.
    /// It is not a reader-admitted producer assertion.
    fn fixture() -> (CheckedPackageV2, CheckedNodeId, CheckedOccurrence) {
        let clause_body = json!({
            "term": "application", "operator": "state_clause",
            "operation": {"identity": "quire.op.state.clause", "laws": [], "mode": null,
                "member": {"kind": "state_clause", "clause": "invariant"}, "leaves": []},
            "result_type": id('1'),
            "arguments": [
                {"term": "aggregate", "members": [reference('2')]},
                reference('3'), reference('4'),
            ],
        });
        let comparison = json!({
            "term": "application", "operator": "binary",
            "operation": {"identity": "quire.op.integer.eq", "laws": [], "mode": null,
                "member": null, "leaves": []},
            "result_type": id('1'), "arguments": [literal("0"), literal("1")],
        });
        let wire: CheckedPackageWireV2 = serde_json::from_value(json!({
            "contract_version": "quire.checked-package/v2",
            "identity_preimage": {
                "version": "quire.checked-package-id/v2",
                "edition": {"role": "edition", "definition": {"authority": "test", "identity": "edition"}},
                "profile_selections": [], "definition_selections": [], "model_selections": [],
                "required_features": [], "dependency_selections": [],
            },
            "package_id": {"domain": "quire.package.semantic/v2", "algorithm": "sha256",
                "digest": "a".repeat(64)},
            "lock": {
                "sources": [],
                "edition": {"role": "edition", "definition": {"authority": "test", "identity": "edition"}},
                "profile_selections": [], "definition_selections": [], "model_selections": [],
                "required_features": [], "dependency_selections": [],
            },
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": [
                node('1', "scalar_type", "boolean", json!({"term": "aggregate", "members": []}), "type"),
                node('2', "value", "parameter", json!({"term": "aggregate", "members": []}), "expression"),
                node('3', "model", "object_type", json!({"term": "aggregate", "members": []}), "declaration"),
                node('4', "expression", "binary", comparison, "expression"),
                node('5', "state", "state_clause", clause_body, "claim"),
            ]},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []},
        }))
        .expect("synthetic wire shape");
        let terms = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| node.body.clone())
            .collect();
        let package = CheckedPackageV2 {
            node_index: super::super::retained_node_index(&wire),
            source_index: super::super::retained_source_index(&wire),
            lowering_facts: Vec::new(),
            wire,
            terms,
            kinds: Vec::new(),
            bytes: 1 << 20,
            models: RetainedModels::default(),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
        };
        let clause = serde_json::from_value(id('5')).expect("node id");
        let claim = CheckedOccurrence {
            role: CheckedOccurrenceRole::Claim,
            ordinal: 0,
        };
        (package, clause, claim)
    }

    fn sync_comparison_term(package: &mut CheckedPackageV2) {
        package.terms[3] = package.wire.semantic_graph.nodes[3].body.clone();
    }

    fn edit_body(package: &mut CheckedPackageV2, position: usize, edit: impl FnOnce(&mut Value)) {
        let mut body = serde_json::to_value(&package.wire.semantic_graph.nodes[position].body)
            .expect("typed fixture body");
        edit(&mut body);
        package.wire.semantic_graph.nodes[position].body =
            CheckedTerm::decode_admitted(&body).expect("edited fixture body remains a closed term");
    }

    #[trace("FR-038-AC-207")]
    #[test]
    fn tc_445_first_refusal_and_locus_survive_post_admission_defects() {
        use CheckedComparisonPath as Path;
        use CheckedComparisonRefusal as Refusal;
        let (mut package, clause, claim) = fixture();
        let unknown: CheckedNodeId = serde_json::from_value(id('9')).expect("node id");
        let failure = package
            .direct_integer_comparison(&unknown, &claim)
            .expect_err("unknown clause");
        assert_eq!(
            (failure.kind, failure.path, failure.child),
            (Refusal::UnknownClause, Path::Clause, None)
        );
        let wrong_claim = CheckedOccurrence {
            role: CheckedOccurrenceRole::Claim,
            ordinal: 1,
        };
        let failure = package
            .direct_integer_comparison(&clause, &wrong_claim)
            .expect_err("absent claim");
        assert_eq!(
            (failure.kind, failure.path),
            (Refusal::MissingClaimOccurrence, Path::Clause)
        );
        package.wire.semantic_graph.nodes[4].semantic_form = "other_form".into();
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("wrong clause form");
        assert_eq!(
            (failure.kind, failure.path, failure.child),
            (Refusal::NotStateClause, Path::Clause, Some(clause.clone()))
        );
        assert_eq!((&failure.clause, &failure.claim), (&clause, &claim));
        package.wire.semantic_graph.nodes[4].semantic_form = "state_clause".into();
        let CheckedTerm::Application { operation, .. } = &mut package.terms[4] else {
            panic!("fixture clause term");
        };
        operation["member"]["clause"] = json!("later_clause_kind");
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("unsupported clause kind");
        assert_eq!(
            (failure.kind, failure.path),
            (Refusal::UnsupportedClauseKind, Path::Clause)
        );
        let CheckedTerm::Application { operation, .. } = &mut package.terms[4] else {
            panic!("fixture clause term");
        };
        operation["member"]["clause"] = json!("invariant");

        let CheckedTerm::Application { arguments, .. } = &mut package.terms[4] else {
            panic!("fixture clause term");
        };
        arguments[2] = CheckedTerm::Reference {
            target: unknown.clone(),
        };
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("missing condition comparison");
        assert_eq!(
            (failure.kind, failure.path, failure.child),
            (
                Refusal::MissingComparison,
                Path::Condition,
                Some(unknown.clone())
            )
        );
        assert_eq!((&failure.clause, &failure.claim), (&clause, &claim));
        let CheckedTerm::Application { arguments, .. } = &mut package.terms[4] else {
            panic!("fixture clause term");
        };
        arguments[2] = CheckedTerm::Reference {
            target: serde_json::from_value(id('4')).expect("comparison id"),
        };

        edit_body(&mut package, 3, |body| {
            body["operation"]["identity"] = json!("quire.op.integer.unknown")
        });
        sync_comparison_term(&mut package);
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("unknown catalog operation");
        let comparison_id: CheckedNodeId = serde_json::from_value(id('4')).expect("comparison id");
        assert_eq!(
            (failure.kind, failure.path, failure.child),
            (
                Refusal::UnknownOperator,
                Path::Comparison,
                Some(comparison_id.clone())
            )
        );
        assert_eq!((&failure.clause, &failure.claim), (&clause, &claim));
        edit_body(&mut package, 3, |body| {
            body["operation"]["identity"] = json!("quire.op.integer.add")
        });
        sync_comparison_term(&mut package);
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("ineligible operation");
        assert_eq!(
            (failure.kind, failure.path),
            (Refusal::IneligibleOperator, Path::Comparison)
        );
        edit_body(&mut package, 3, |body| {
            body["operation"]["identity"] = json!("quire.op.integer.eq")
        });
        edit_body(&mut package, 3, |body| {
            body["arguments"] = json!([literal("0")])
        });
        sync_comparison_term(&mut package);
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("invalid binary arity");
        assert_eq!(
            (failure.kind, failure.path, failure.child),
            (
                Refusal::InvalidComparisonShape,
                Path::Comparison,
                Some(comparison_id)
            )
        );
        assert_eq!((&failure.clause, &failure.claim), (&clause, &claim));
        edit_body(&mut package, 3, |body| {
            body["arguments"] = json!([{"term": "aggregate", "members": []}, literal("1")])
        });
        sync_comparison_term(&mut package);
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("malformed operand shape");
        assert_eq!(
            (failure.kind, failure.path, failure.child),
            (Refusal::IneligibleOperand, Path::Operand(0), None)
        );
        assert_eq!((&failure.clause, &failure.claim), (&clause, &claim));
        edit_body(&mut package, 3, |body| {
            body["arguments"] = json!([
                reference('9'),
                literal("340282366920938463463374607431768211456")
            ])
        });
        sync_comparison_term(&mut package);
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("operand zero first");
        assert_eq!(
            (failure.kind, failure.path, failure.child),
            (Refusal::MissingChild, Path::Operand(0), Some(unknown))
        );
        edit_body(&mut package, 3, |body| {
            body["arguments"] = json!([
                literal("0"),
                literal("340282366920938463463374607431768211456")
            ])
        });
        sync_comparison_term(&mut package);
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("literal outside i128");
        assert_eq!(
            (failure.kind, failure.path),
            (Refusal::RangeOutOfI128, Path::Operand(1))
        );
        edit_body(&mut package, 3, |body| {
            body["arguments"] = json!([literal("0"), literal("1")])
        });
        sync_comparison_term(&mut package);
        let failure = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("two literals");
        assert_eq!(
            (failure.kind, failure.path),
            (Refusal::NoRead, Path::Comparison)
        );
    }

    #[trace("FR-038-AC-205")]
    #[test]
    fn tc_445_integer_literal_endpoints_remain_exact_i128_singletons() {
        for value in [i128::MIN, i128::MAX] {
            let range = literal_range(&literal(&value.to_string())).expect("inclusive endpoint");
            assert_eq!((range.lower, range.upper), (value, value));
        }
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn comparison_uses_retained_terms_after_raw_condition_body_changes() {
        let (mut package, clause, claim) = fixture();
        edit_body(&mut package, 3, |body| {
            *body = json!({"term": "aggregate", "members": []})
        });
        let error = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("two retained inline literals still have no read");
        assert_eq!(error.kind, CheckedComparisonRefusal::NoRead);
        assert_eq!(error.path, CheckedComparisonPath::Comparison);
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn comparison_uses_retained_clause_condition_after_raw_body_changes() {
        let (mut package, clause, claim) = fixture();
        edit_body(&mut package, 4, |body| {
            body["arguments"][2] = reference('9')
        });
        let error = package
            .direct_integer_comparison(&clause, &claim)
            .expect_err("two retained inline literals still have no read");
        assert_eq!(error.kind, CheckedComparisonRefusal::NoRead);
        assert_eq!(error.path, CheckedComparisonPath::Comparison);
    }
}
