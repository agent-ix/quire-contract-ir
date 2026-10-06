//! Typed, position-preserving operands of admitted scalar integer applications.

use super::operation_catalog::operation_catalog;
use super::{CheckedNodeId, CheckedOccurrence, CheckedPackageV2};
use serde_json::Value;
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
        let body = &node.body;
        if body.get("term").and_then(Value::as_str) != Some("application") {
            return Err(Error::NotApplication);
        }
        if !node.occurrences.contains(occurrence) {
            return Err(Error::MissingOccurrence);
        }
        let identity = body
            .pointer("/operation/identity")
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
        let arguments = body
            .get("arguments")
            .and_then(Value::as_array)
            .ok_or(Error::MissingRange)?;
        arguments
            .iter()
            .enumerate()
            .map(|(position, argument)| {
                let ordinal = u64::try_from(position).map_err(|_| Error::MissingRange)?;
                let (child, range) = match argument.get("term").and_then(Value::as_str) {
                    Some("reference") => {
                        let target: CheckedNodeId = serde_json::from_value(
                            argument.get("target").cloned().ok_or(Error::MissingChild)?,
                        )
                        .map_err(|_| Error::MissingChild)?;
                        let child = nodes
                            .iter()
                            .find(|node| node.node_id == target)
                            .ok_or(Error::MissingChild)?;
                        let range = if child.node_tag.as_ref() == "value"
                            && child.semantic_form.as_ref() == "literal"
                        {
                            literal_range(&child.body)?
                        } else {
                            type_range(nodes, &child.semantic_type)?
                        };
                        (CheckedScalarOperandChild::GraphChild(target), range)
                    }
                    Some("literal") => (
                        CheckedScalarOperandChild::InlineLiteral {
                            application: application.clone(),
                            occurrence: occurrence.clone(),
                            ordinal,
                        },
                        literal_range(argument)?,
                    ),
                    _ => return Err(Error::MissingRange),
                };
                Ok(CheckedScalarOperand {
                    ordinal,
                    child,
                    range,
                })
            })
            .collect()
    }
}

fn literal_range(term: &Value) -> Result<CheckedScalarOperandRange, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;
    if term.get("value_kind").and_then(Value::as_str) != Some("integer") {
        return Err(Error::MissingRange);
    }
    let text = term
        .get("value")
        .and_then(Value::as_str)
        .ok_or(Error::MissingRange)?;
    let value = text.parse::<i128>().map_err(|_| Error::RangeOutOfI128)?;
    Ok(CheckedScalarOperandRange {
        lower: value,
        upper: value,
    })
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
        CheckedPackageV2 {
            wire,
            kinds: Vec::new(),
            bytes: 1 << 20,
            models: RetainedModels::default(),
        }
    }

    /// Tracing: TC-048, FR-038-AC-162, FR-038-AC-163
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
        package.wire.semantic_graph.nodes[3].body["operation"]["identity"] = json!("test.unknown");
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::UnknownOperator)
        );
    }
}
