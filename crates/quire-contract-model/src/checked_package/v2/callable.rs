// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded read of admitted callable bodies and operation frames (FR-354).

use super::model_members::member_name;
use super::structural::typed_anchor_body;
use super::{
    CheckedBindingNameKind, CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2,
    CheckedTerm, FunctionForm, LiteralKind, StateForm, ValueForm,
};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// The admitted identity under which a callable body is read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedCallableIdentity {
    /// A declared function or a selected model-owned body function.
    Function(CheckedNodeId),
    /// An operation anchor joined to its selected original member identity.
    Operation {
        /// The admitted operation anchor.
        anchor: CheckedNodeId,
        /// The selected model operation member identity.
        member: Box<str>,
    },
}

/// A complete admitted callable read. Handles resolve only in the package read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCallable {
    /// The function or operation identity.
    pub identity: CheckedCallableIdentity,
    /// Parameter node handles in producer order.
    pub parameters: Vec<CheckedNodeId>,
    /// The admitted expression body handle.
    pub body: CheckedNodeId,
    /// The model context for an operation, absent for a plain function.
    pub context: Option<CheckedNodeId>,
    /// The admitted frame node, absent for a pure function. An empty frame is
    /// represented by `Some` pointing to an empty `CheckedTerm::Frame`.
    pub frame: Option<CheckedNodeId>,
}

/// One complete table or no table at all on refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCallableTable {
    rows: Vec<CheckedCallable>,
    work: u64,
}

impl CheckedCallableTable {
    /// Callable rows in graph order, with functions before operation anchors.
    pub fn rows(&self) -> &[CheckedCallable] {
        &self.rows
    }

    /// Number of charged projection visits.
    pub const fn work(&self) -> u64 {
        self.work
    }
}

/// Stable reason a callable projection cannot return a complete table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedCallableReadErrorCode {
    /// The requested node does not belong to this admitted package.
    UnknownNode,
    /// A selected callable has no body reference.
    MissingBody,
    /// A body or parameter reference is absent or of the wrong admitted form.
    MalformedBody,
    /// More than one body claims the same selected operation member.
    AmbiguousBody,
    /// An operation anchor has no selected operation identity in the package.
    UnsupportedSelectedOperation,
    /// A selected model operation has no matching body function.
    MissingSelectedJoin,
    /// A body function or anchor claims a different selected owner.
    ForeignSelectedJoin,
    /// An operation anchor has no valid admitted frame node.
    MissingFrame,
    /// The caller's work limit was exceeded.
    WorkLimit,
}

/// Typed refusal with the first failing node and charged work.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("callable projection {code:?} at {locus:?} after {work} work units")]
pub struct CheckedCallableReadError {
    /// Stable refusal code.
    pub code: CheckedCallableReadErrorCode,
    /// First failing admitted node.
    pub locus: CheckedNodeId,
    /// Charged work, including the charge that exceeded the limit.
    pub work: u64,
}

struct Budget {
    work: u64,
    limit: u64,
}

impl Budget {
    fn charge(&mut self, locus: &CheckedNodeId) -> Result<(), CheckedCallableReadError> {
        self.work = self.work.saturating_add(1);
        if self.work > self.limit {
            return Err(self.error(CheckedCallableReadErrorCode::WorkLimit, locus));
        }
        Ok(())
    }

    fn error(
        &self,
        code: CheckedCallableReadErrorCode,
        locus: &CheckedNodeId,
    ) -> CheckedCallableReadError {
        CheckedCallableReadError {
            code,
            locus: locus.clone(),
            work: self.work,
        }
    }
}

struct ParsedFunction<'a> {
    parameters: Vec<CheckedNodeId>,
    body: Option<CheckedNodeId>,
    clause: Option<&'a str>,
}

fn function_body<'a>(
    package: &'a CheckedPackageV2,
    node: &CheckedNodeId,
    budget: &mut Budget,
) -> Result<ParsedFunction<'a>, CheckedCallableReadError> {
    let Some(CheckedTerm::Aggregate { members }) = package.term(node) else {
        return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
    };
    let mut parameters = None;
    let mut body = None;
    let mut clause = None;
    for member in members {
        budget.charge(node)?;
        let CheckedTerm::Binding { name, value } = member else {
            return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
        };
        match name.kind() {
            CheckedBindingNameKind::Parameters => {
                let CheckedTerm::Aggregate { members } = value.as_ref() else {
                    return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                };
                let mut ordered = Vec::new();
                for parameter in members {
                    budget.charge(node)?;
                    let CheckedTerm::Reference { target } = parameter else {
                        return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                    };
                    if package.node_kind(target)
                        != Some(CheckedNodeKind::Value(ValueForm::Parameter))
                    {
                        return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                    }
                    ordered.push(target.clone());
                }
                if parameters.replace(ordered).is_some() {
                    return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                }
            }
            CheckedBindingNameKind::Body => {
                let CheckedTerm::Reference { target } = value.as_ref() else {
                    return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                };
                if package.term(target).is_none() || body.replace(target.clone()).is_some() {
                    return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                }
            }
            CheckedBindingNameKind::Clause => {
                let CheckedTerm::Literal {
                    value_kind: LiteralKind::Text,
                    value,
                    ..
                } = value.as_ref()
                else {
                    return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                };
                if clause
                    .replace(value.as_str().ok_or_else(|| {
                        budget.error(CheckedCallableReadErrorCode::MalformedBody, node)
                    })?)
                    .is_some()
                {
                    return Err(budget.error(CheckedCallableReadErrorCode::MalformedBody, node));
                }
            }
            CheckedBindingNameKind::Optional | CheckedBindingNameKind::Other(_) => {}
        }
    }
    Ok(ParsedFunction {
        parameters: parameters.unwrap_or_default(),
        body,
        clause,
    })
}

impl CheckedPackageV2 {
    /// Read one executable source-declared function without operation rows.
    ///
    /// A present node that is not an executable source function returns
    /// `None`. This read does not scan unrelated operation anchors.
    pub fn source_callable(
        &self,
        node: &CheckedNodeId,
        work_limit: u64,
    ) -> Result<Option<CheckedCallable>, CheckedCallableReadError> {
        let mut budget = Budget {
            work: 0,
            limit: work_limit,
        };
        budget.charge(node)?;
        let found = self
            .node(node)
            .ok_or_else(|| budget.error(CheckedCallableReadErrorCode::UnknownNode, node))?;
        if !matches!(
            self.node_kind(node),
            Some(CheckedNodeKind::Function(
                FunctionForm::PureFunction
                    | FunctionForm::Predicate
                    | FunctionForm::RecursiveFunction
            ))
        ) || found.declaration.is_none()
            || !matches!(found.owner, Some(CheckedNodeOwner::Source { .. }))
        {
            return Ok(None);
        }
        let parsed = function_body(self, node, &mut budget)?;
        Ok(parsed.body.map(|body| CheckedCallable {
            identity: CheckedCallableIdentity::Function(node.clone()),
            parameters: parsed.parameters,
            body,
            context: None,
            frame: None,
        }))
    }

    /// Read selected function bodies and model operation anchors as one bounded table.
    ///
    /// Source-owned anchors without a witnessed selected operation member
    /// return `UnsupportedSelectedOperation`. No row is exposed after any refusal.
    pub fn callable_table(
        &self,
        work_limit: u64,
    ) -> Result<CheckedCallableTable, CheckedCallableReadError> {
        let mut budget = Budget {
            work: 0,
            limit: work_limit,
        };
        let mut rows = Vec::new();
        let mut model_bodies = BTreeMap::<(&str, &str), usize>::new();
        let mut selected_operations = BTreeSet::<(&str, &str)>::new();
        for model in self.models.selected_models() {
            let Some(root) = self.graph().nodes.get(model.root) else {
                continue;
            };
            budget.charge(&root.node_id)?;
            for declaration in model.object_types.values() {
                budget.charge(&root.node_id)?;
                for operation in &declaration.operations {
                    budget.charge(&root.node_id)?;
                    selected_operations.insert((&model.identity, &operation.identity));
                }
            }
        }
        for (position, node) in self.graph().nodes.iter().enumerate() {
            budget.charge(&node.node_id)?;
            if !matches!(
                self.kinds.get(position),
                Some(CheckedNodeKind::Function(
                    FunctionForm::PureFunction
                        | FunctionForm::Predicate
                        | FunctionForm::RecursiveFunction
                ))
            ) {
                continue;
            }
            let selected = node.declaration.is_some()
                || matches!(node.owner, Some(CheckedNodeOwner::Model { .. }));
            if !selected {
                continue;
            }
            let ParsedFunction {
                parameters,
                body,
                clause,
            } = function_body(self, &node.node_id, &mut budget)?;
            if let Some(CheckedNodeOwner::Model {
                identity,
                node: member,
            }) = &node.owner
            {
                if clause != Some("body") {
                    continue;
                }
                if !selected_operations.contains(&(identity.as_ref(), member.as_ref())) {
                    return Err(budget.error(
                        CheckedCallableReadErrorCode::ForeignSelectedJoin,
                        &node.node_id,
                    ));
                }
                if model_bodies
                    .insert((identity, member), rows.len())
                    .is_some()
                {
                    return Err(
                        budget.error(CheckedCallableReadErrorCode::AmbiguousBody, &node.node_id)
                    );
                }
            }
            let Some(body) = body else {
                // A declaration without an executable body is not a callable
                // definition. A selected operation body clause must have one.
                if matches!(node.owner, Some(CheckedNodeOwner::Model { .. })) {
                    return Err(
                        budget.error(CheckedCallableReadErrorCode::MissingBody, &node.node_id)
                    );
                }
                continue;
            };
            rows.push(CheckedCallable {
                identity: CheckedCallableIdentity::Function(node.node_id.clone()),
                parameters,
                body,
                context: None,
                frame: None,
            });
        }
        for (position, node) in self.graph().nodes.iter().enumerate() {
            budget.charge(&node.node_id)?;
            if self.kinds.get(position) != Some(&CheckedNodeKind::State(StateForm::OperationAnchor))
            {
                continue;
            }
            let anchor = self
                .term(&node.node_id)
                .and_then(typed_anchor_body)
                .ok_or_else(|| {
                    budget.error(CheckedCallableReadErrorCode::MalformedBody, &node.node_id)
                })?;
            let (document, original_context, _) = self
                .models
                .selected_declaration(&anchor.context)
                .ok_or_else(|| {
                    budget.error(
                        CheckedCallableReadErrorCode::UnsupportedSelectedOperation,
                        &node.node_id,
                    )
                })?;
            let model = self
                .models
                .selected_models()
                .iter()
                .find(|model| model.identity.as_ref() == document)
                .ok_or_else(|| {
                    budget.error(
                        CheckedCallableReadErrorCode::MissingSelectedJoin,
                        &node.node_id,
                    )
                })?;
            let operation = model
                .object_types
                .get(original_context)
                .and_then(|decl| {
                    decl.operations
                        .iter()
                        .find(|operation| member_name(&operation.identity) == anchor.operation)
                })
                .ok_or_else(|| {
                    budget.error(
                        CheckedCallableReadErrorCode::MissingSelectedJoin,
                        &node.node_id,
                    )
                })?;
            let function = model_bodies
                .get(&(document, operation.identity.as_ref()))
                .and_then(|index| rows.get(*index))
                .ok_or_else(|| {
                    budget.error(
                        CheckedCallableReadErrorCode::MissingSelectedJoin,
                        &node.node_id,
                    )
                })?;
            if self.node_kind(&anchor.frame) != Some(CheckedNodeKind::State(StateForm::Frame))
                || !matches!(self.term(&anchor.frame), Some(CheckedTerm::Frame { .. }))
            {
                return Err(budget.error(CheckedCallableReadErrorCode::MissingFrame, &node.node_id));
            }
            rows.push(CheckedCallable {
                identity: CheckedCallableIdentity::Operation {
                    anchor: node.node_id.clone(),
                    member: operation.identity.clone(),
                },
                parameters: function.parameters.clone(),
                body: function.body.clone(),
                context: Some(anchor.context),
                frame: Some(anchor.frame),
            });
        }
        Ok(CheckedCallableTable {
            rows,
            work: budget.work,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::test_support::{id, node_id, private_wire};
    use crate::checked_package::v2::{
        retained_node_index, retained_source_index, CheckedNodeTag, CheckedPackageWireV2,
        RetainedModels,
    };
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    fn fixture(with_anchor: bool) -> CheckedPackageV2 {
        let empty = || json!({"term": "aggregate", "members": []});
        let reference = |target: char| json!({"term": "reference", "target": id(target)});
        let node = |digit: char, tag: &str, form: &str, ty: char, body: Value| {
            json!({
                "node_id": id(digit), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag, "semantic_form": form, "semantic_type": id(ty),
                "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
                "body": body,
            })
        };
        let mut function = node(
            '4',
            "function",
            "pure_function",
            '1',
            json!({"term": "aggregate", "members": [
                {"term": "binding", "name": "parameters", "value": {
                    "term": "aggregate", "members": [reference('2')]}},
                {"term": "binding", "name": "body", "value": reference('3')},
            ]}),
        );
        function["declaration"] = json!({"qualified_name": ["calculate"]});
        function["owner"] = json!({
            "kind": "source", "authority": "test", "identity": "source"
        });
        let mut nodes = vec![
            node('1', "scalar_type", "text", '1', empty()),
            node('2', "value", "parameter", '1', empty()),
            node('3', "expression", "reference", '1', reference('1')),
            function,
            // A function-tagged application is not a callable definition.
            node('7', "function", "predicate", '1', empty()),
        ];
        if with_anchor {
            nodes.push(node(
                '5',
                "state",
                "frame",
                '1',
                json!({
                    "term": "frame", "modifies": [], "creates": [],
                    "deletes": [], "deletes_scoped": []
                }),
            ));
            nodes.push(node('6', "state", "operation_anchor", '1', json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "context", "value": reference('1')},
                    {"term": "binding", "name": "operation", "value": {
                        "term": "literal", "type": id('1'), "value_kind": "text", "value": "advance"}},
                    {"term": "binding", "name": "frame", "value": reference('5')},
                ]
            })));
        }
        let wire: CheckedPackageWireV2 = private_wire(json!({
            "contract_version": "quire.checked-package/v2", "name": "test",
            "lock": {
                "sources": [],
                "edition": {"role": "edition", "definition": {
                    "authority": "test", "identity": "edition"}},
                "profile_selections": [], "definition_selections": [],
                "required_features": [], "dependency_selections": [],
            },
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"},
                            "entries": []},
        }));
        let kinds = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| {
                let tag = CheckedNodeTag::from_wire(&node.node_tag).expect("fixture tag");
                CheckedNodeKind::decode(tag, &node.semantic_form).expect("fixture form")
            })
            .collect();
        let terms = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| node.body.clone())
            .collect();
        CheckedPackageV2 {
            node_index: retained_node_index(&wire),
            structural_keys: Vec::new(),
            source_index: retained_source_index(&wire),
            lowering_facts: Vec::new(),
            wire,
            terms,
            kinds,
            bytes: 1 << 20,
            models: RetainedModels::default(),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
        }
    }

    /// Trace: FR-354-AC-3, FR-354-AC-7.
    #[trace("FR-354-AC-3", "FR-354-AC-7")]
    #[test]
    fn selected_function_body_is_ordered_bounded_and_clone_stable() {
        let package = fixture(false);
        let table = package.callable_table(100).expect("selected function");
        assert_eq!(table.rows().len(), 1);
        assert_eq!(
            table.rows()[0].identity,
            CheckedCallableIdentity::Function(node_id('4'))
        );
        assert_eq!(table.rows()[0].parameters, vec![node_id('2')]);
        assert_eq!(table.rows()[0].body, node_id('3'));
        assert_eq!(table.rows()[0].frame, None);
        assert_eq!(
            package.callable_table(100),
            package.clone().callable_table(100)
        );
        let denied = package
            .callable_table(table.work() - 1)
            .expect_err("one under work");
        assert_eq!(denied.code, CheckedCallableReadErrorCode::WorkLimit);
        assert_eq!(denied.work, table.work());
    }

    /// Trace: FR-354-AC-3.
    #[trace("FR-354-AC-3")]
    #[test]
    fn bodyless_declaration_is_not_a_callable_and_dangling_body_refuses() {
        let mut package = fixture(false);
        package.terms[3] = CheckedTerm::Aggregate { members: vec![] };
        assert!(package
            .callable_table(100)
            .expect("declaration only")
            .rows()
            .is_empty());
        package.terms[3] = CheckedTerm::Aggregate {
            members: vec![CheckedTerm::Binding {
                name: "body".into(),
                value: Box::new(CheckedTerm::Reference {
                    target: node_id('9'),
                }),
            }],
        };
        assert_eq!(
            package.callable_table(100).expect_err("dangling body").code,
            CheckedCallableReadErrorCode::MalformedBody,
        );
    }

    /// Trace: FR-354-AC-3.
    #[trace("FR-354-AC-3")]
    #[test]
    fn source_owned_anchor_refuses_without_a_selected_operation_witness() {
        let package = fixture(true);
        let function = package
            .source_callable(&node_id('4'), 100)
            .expect("source function read")
            .expect("body function");
        assert_eq!(function.body, node_id('3'));
        let error = package.callable_table(100).expect_err("source anchor");
        assert_eq!(
            error.code,
            CheckedCallableReadErrorCode::UnsupportedSelectedOperation
        );
        assert_eq!(error.locus, node_id('6'));
        assert_eq!(
            package
                .source_callable(&node_id('9'), 100)
                .expect_err("foreign id")
                .code,
            CheckedCallableReadErrorCode::UnknownNode,
        );
    }
}
