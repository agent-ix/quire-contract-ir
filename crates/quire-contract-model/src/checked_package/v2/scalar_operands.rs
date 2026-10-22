//! Typed, position-preserving operands of admitted scalar integer applications.

use super::operation_catalog::operation_catalog;
use super::{
    CheckedNodeId, CheckedOccurrence, CheckedPackageV2, CheckedSemanticNodeV2, CheckedTerm,
    LiteralKind,
};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde_json::Value;
use std::borrow::Cow;
use thiserror::Error;

/// The inclusive finite range of one integer operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedScalarOperandRange {
    /// Inclusive lower endpoint.
    pub lower: i128,
    /// Inclusive upper endpoint.
    pub upper: i128,
}

/// The identity of one operand, without inventing a graph node for an inline term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedScalarOperandChild {
    /// A referenced checked graph node.
    GraphChild(CheckedNodeId),
    /// A literal term within one occurrence and argument position.
    InlineLiteral {
        /// The containing application node.
        application: CheckedNodeId,
        /// The selected occurrence of that application.
        occurrence: CheckedOccurrence,
        /// Its zero-based argument position.
        ordinal: u64,
    },
}

/// One argument of an admitted scalar integer application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedScalarOperand {
    /// Zero-based position in `body.arguments`.
    pub ordinal: u64,
    /// The graph child or occurrence-qualified inline term.
    pub child: CheckedScalarOperandChild,
    /// Exact inclusive integer range.
    pub range: CheckedScalarOperandRange,
}

/// Why a scalar integer application's operands cannot be read.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CheckedScalarOperandError {
    /// The requested node is absent.
    #[error("unknown checked node")]
    UnknownNode,
    /// The requested node does not hold an application term.
    #[error("checked node is not an application")]
    NotApplication,
    /// The supplied occurrence is absent from the application.
    #[error("application occurrence is absent")]
    MissingOccurrence,
    /// The application's operation identity is absent from the catalog.
    #[error("unknown application operation")]
    UnknownOperator,
    /// The catalogued operation is outside scalar integer arithmetic.
    #[error("operation is not eligible for scalar integer operands")]
    IneligibleOperator,
    /// An argument names a graph node that is absent.
    #[error("referenced operand child is absent")]
    MissingChild,
    /// A child has no readable integer range.
    #[error("operand has no readable integer range")]
    MissingRange,
    /// An integer operand is valid but has no finite bounds.
    #[error("integer operand is unbounded")]
    UnboundedRange,
    /// A finite endpoint or literal cannot be represented as `i128`.
    #[error("integer operand is outside i128")]
    RangeOutOfI128,
}

/// One IR-owned parse of an admitted scalar argument, shared by both package views.
pub(super) struct ScalarArgument<'a> {
    pub(super) ordinal: u64,
    pub(super) child: ScalarArgumentChild<'a>,
    pub(super) own_range: Result<CheckedScalarOperandRange, CheckedScalarOperandError>,
}

pub(super) enum ScalarArgumentChild<'a> {
    Graph(&'a CheckedSemanticNodeV2),
    InlineInteger(BigInt),
}

/// Interpret the validated argument vector once, without requiring every own
/// range to be finite. The checked-package view imposes that older policy;
/// the admitted-package view retains each position when an own range is absent.
/// Per-position laziness preserves the checked view's first-error order when
/// an early range is absent and a later argument is malformed.
pub(super) fn scalar_arguments<'a>(
    nodes: &'a [CheckedSemanticNodeV2],
    node: &'a CheckedSemanticNodeV2,
    retained: Option<&'a CheckedTerm>,
) -> Result<ScalarArguments<'a>, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;

    // ContractPackageV1 does not retain a second body copy. Its admitted
    // root is decoded on demand; CheckedPackageV2 borrows its retained tree.
    let term = match retained {
        Some(term) => Cow::Borrowed(term),
        None => {
            Cow::Owned(CheckedTerm::decode_admitted(&node.body).map_err(|_| Error::NotApplication)?)
        }
    };
    let CheckedTerm::Application { operation, .. } = term.as_ref() else {
        return Err(Error::NotApplication);
    };
    let identity = operation
        .get("identity")
        .and_then(Value::as_str)
        .ok_or(Error::UnknownOperator)?;
    if operation_catalog().entry(identity).is_none() {
        return Err(Error::UnknownOperator);
    }
    if !matches!(
        identity,
        "quire.op.integer.add"
            | "quire.op.integer.sub"
            | "quire.op.integer.mul"
            | "quire.op.integer.negate"
    ) {
        return Err(Error::IneligibleOperator);
    }
    let arguments = match term {
        Cow::Borrowed(CheckedTerm::Application { arguments, .. }) => {
            Cow::Borrowed(arguments.as_slice())
        }
        Cow::Owned(CheckedTerm::Application { arguments, .. }) => Cow::Owned(arguments),
        _ => return Err(Error::NotApplication),
    };
    Ok(ScalarArguments {
        nodes,
        arguments,
        next: 0,
    })
}

/// Lazy, position-preserving scalar projection over a borrowed or decoded
/// admitted application. It holds no second package body copy.
pub(super) struct ScalarArguments<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    arguments: Cow<'a, [CheckedTerm]>,
    next: usize,
}

impl<'a> Iterator for ScalarArguments<'a> {
    type Item = Result<ScalarArgument<'a>, CheckedScalarOperandError>;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.next;
        let argument = self.arguments.get(position)?;
        self.next = self.next.saturating_add(1);
        Some(self.project(position, argument))
    }
}

impl<'a> ScalarArguments<'a> {
    fn project(
        &self,
        position: usize,
        argument: &CheckedTerm,
    ) -> Result<ScalarArgument<'a>, CheckedScalarOperandError> {
        use CheckedScalarOperandError as Error;
        let ordinal = u64::try_from(position).map_err(|_| Error::MissingRange)?;
        let (child, own_range) = match argument {
            CheckedTerm::Reference { target } => {
                let child = self
                    .nodes
                    .iter()
                    .find(|node| node.node_id == *target)
                    .ok_or(Error::MissingChild)?;
                let range = if child.node_tag.as_ref() == "value"
                    && child.semantic_form.as_ref() == "literal"
                {
                    literal_range(&child.body)
                } else {
                    type_range(self.nodes, &child.semantic_type)
                };
                (ScalarArgumentChild::Graph(child), range)
            }
            CheckedTerm::Literal { .. } => {
                let value = typed_literal_integer(argument)?;
                let range = value.to_i128().map_or(Err(Error::RangeOutOfI128), |value| {
                    Ok(CheckedScalarOperandRange {
                        lower: value,
                        upper: value,
                    })
                });
                (ScalarArgumentChild::InlineInteger(value), range)
            }
            _ => return Err(Error::MissingRange),
        };
        Ok(ScalarArgument {
            ordinal,
            child,
            own_range,
        })
    }
}

impl CheckedPackageV2 {
    /// Reads an admitted integer application's operands in wire argument order.
    ///
    /// Only `add`, `sub`, `mul` and `negate` are eligible. Every
    /// operand must have an exact finite `i128` range; on failure this returns
    /// one typed error and no partial list.
    pub fn scalar_application_operands(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
    ) -> Result<Vec<CheckedScalarOperand>, CheckedScalarOperandError> {
        use CheckedScalarOperandError as Error;

        let nodes = &self.wire.semantic_graph.nodes;
        let node = nodes
            .iter()
            .find(|node| &node.node_id == application)
            .ok_or(Error::UnknownNode)?;
        let term = self.term(application).ok_or(Error::NotApplication)?;
        if !matches!(term, CheckedTerm::Application { .. }) {
            return Err(Error::NotApplication);
        }
        if !node.occurrences.contains(occurrence) {
            return Err(Error::MissingOccurrence);
        }
        scalar_arguments(nodes, node, Some(term))?
            .map(|argument| {
                let argument = argument?;
                let child = match argument.child {
                    ScalarArgumentChild::Graph(node) => {
                        CheckedScalarOperandChild::GraphChild(node.node_id.clone())
                    }
                    ScalarArgumentChild::InlineInteger(_) => {
                        CheckedScalarOperandChild::InlineLiteral {
                            application: application.clone(),
                            occurrence: occurrence.clone(),
                            ordinal: argument.ordinal,
                        }
                    }
                };
                Ok(CheckedScalarOperand {
                    ordinal: argument.ordinal,
                    child,
                    range: argument.own_range?,
                })
            })
            .collect()
    }
}

fn literal_range(term: &Value) -> Result<CheckedScalarOperandRange, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;
    let value = literal_integer(term)?
        .to_i128()
        .ok_or(Error::RangeOutOfI128)?;
    Ok(CheckedScalarOperandRange {
        lower: value,
        upper: value,
    })
}

fn literal_integer(term: &Value) -> Result<BigInt, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;
    if term.get("value_kind").and_then(Value::as_str) != Some("integer") {
        return Err(Error::MissingRange);
    }
    let text = term
        .get("value")
        .and_then(Value::as_str)
        .ok_or(Error::MissingRange)?;
    text.parse::<BigInt>().map_err(|_| Error::RangeOutOfI128)
}

fn typed_literal_integer(term: &CheckedTerm) -> Result<BigInt, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;
    let CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    } = term
    else {
        return Err(Error::MissingRange);
    };
    value
        .as_str()
        .ok_or(Error::MissingRange)?
        .parse::<BigInt>()
        .map_err(|_| Error::RangeOutOfI128)
}

fn type_range(
    nodes: &[super::CheckedSemanticNodeV2],
    ty: &CheckedNodeId,
) -> Result<CheckedScalarOperandRange, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;
    let node = nodes
        .iter()
        .find(|node| &node.node_id == ty)
        .ok_or(Error::MissingRange)?;
    if node.node_tag.as_ref() == "scalar_type" && node.semantic_form.as_ref() == "integer" {
        return Err(Error::UnboundedRange);
    }
    if node.node_tag.as_ref() != "bounded_domain" || node.semantic_form.as_ref() != "integer_range"
    {
        return Err(Error::MissingRange);
    }
    let members = node
        .body
        .get("members")
        .and_then(Value::as_array)
        .ok_or(Error::MissingRange)?;
    let endpoint = |name: &str| {
        let text = members.iter().find_map(|member| {
            (member.get("name").and_then(Value::as_str) == Some(name))
                .then(|| member.pointer("/value/value").and_then(Value::as_str))
                .flatten()
        });
        text.ok_or(Error::MissingRange)?
            .parse::<i128>()
            .map_err(|_| Error::RangeOutOfI128)
    };
    Ok(CheckedScalarOperandRange {
        lower: endpoint("min")?,
        upper: endpoint("max")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::{CheckedPackageWireV2, RetainedModels};
    use ix_trace_rs::trace;
    use serde_json::json;

    fn id(digit: char) -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": digit.to_string().repeat(64)})
    }

    /// The accessor reads only the semantic graph. This private fixture has
    /// the other wire members needed to construct a package, then tests may
    /// mutate the graph in states admission would ordinarily reject.
    fn fixture_package() -> CheckedPackageV2 {
        let node = |digit: char, tag: &str, form: &str, ty: char, body: Value| {
            json!({
                "node_id": id(digit), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag, "semantic_form": form, "semantic_type": id(ty),
                "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
                "body": body,
            })
        };
        let application = json!({
            "term": "application", "operator": "binary",
            "operation": {"identity": "quire.op.integer.add", "laws": [], "mode": null,
                "member": null, "leaves": []},
            "result_type": id('2'),
            "arguments": [
                {"term": "reference", "target": id('3')},
                {"term": "literal", "type": id('1'), "value_kind": "integer", "value": "7"},
            ],
        });
        let wire: CheckedPackageWireV2 = serde_json::from_value(json!({
            "contract_version": "quire.checked-package/v2",
            "identity_preimage": {
                "version": "quire.checked-package-id/v2",
                "edition": {"role": "edition", "definition": {"authority": "test", "identity": "edition"}},
                "profile_selections": [], "definition_selections": [], "model_selections": [],
                "required_features": [], "dependency_selections": [], "identity_projection": [],
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
                node('1', "scalar_type", "integer", '1', json!({"term": "aggregate", "members": []})),
                node('2', "bounded_domain", "integer_range", '1', json!({"term": "aggregate", "members": [
                    {"term": "binding", "name": "min", "value": {"term": "literal", "type": id('1'), "value_kind": "integer", "value": "0"}},
                    {"term": "binding", "name": "max", "value": {"term": "literal", "type": id('1'), "value_kind": "integer", "value": "1000"}},
                ]})),
                node('3', "value", "parameter", '2', json!({"term": "aggregate", "members": []})),
                node('4', "expression", "binary", '2', application),
            ]},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []},
        }))
        .expect("private package fixture");
        let terms = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| super::super::CheckedTerm::decode_admitted(&node.body).expect("term"))
            .collect();
        CheckedPackageV2 {
            node_index: super::super::retained_node_index(&wire),
            lowering_facts: Vec::new(),
            wire,
            terms,
            kinds: Vec::new(),
            bytes: 1 << 20,
            models: RetainedModels::default(),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
        }
    }

    #[trace("TC-048", "FR-038-AC-162", "FR-038-AC-163")]
    #[test]
    fn tc_048_post_admission_graph_mutations_return_distinct_errors() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("id");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        assert_eq!(
            package
                .scalar_application_operands(&application, &occurrence)
                .expect("healthy")
                .len(),
            2
        );
        package.wire.semantic_graph.nodes.remove(2);
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::MissingChild)
        );
        let mut package = fixture_package();
        package.wire.semantic_graph.nodes[1].body = json!({"term": "aggregate", "members": []});
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::MissingRange)
        );
        let mut package = fixture_package();
        let CheckedTerm::Application { operation, .. } = &mut package.terms[3] else {
            panic!("application term");
        };
        operation["identity"] = json!("test.unknown");
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::UnknownOperator)
        );
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn admitted_scalar_arguments_use_retained_terms_after_raw_body_changes() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("id");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let expected = package
            .scalar_application_operands(&application, &occurrence)
            .expect("original operands");
        package.wire.semantic_graph.nodes[3].body = json!({"term": "aggregate", "members": []});
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Ok(expected)
        );
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn admitted_scalar_ranges_use_retained_graph_terms_after_raw_body_changes() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("id");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let expected = package
            .scalar_application_operands(&application, &occurrence)
            .expect("original ranges");
        package.wire.semantic_graph.nodes[1].body = json!({"term": "aggregate", "members": []});
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Ok(expected)
        );
    }
}
