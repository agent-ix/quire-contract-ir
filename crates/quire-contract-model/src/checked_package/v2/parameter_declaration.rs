//! Typed declared name and level of an admitted `value`/`parameter` node.

use super::structural::typed_parameter_binding;
use super::{CheckedNodeId, CheckedNodeKind, CheckedPackageV2, ValueForm};
use thiserror::Error;

/// The declared name and level of one parameter node.
///
/// The node carries no function, owner or declaration identity: parameters of
/// equal name, level and type share one node (QSL FR-092).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedParameterDeclaration {
    /// The parameter node that was read.
    pub node: CheckedNodeId,
    /// The admitted identifier text of the `"name"` binding.
    pub name: Box<str>,
    /// The canonical non-negative decimal text of the `"level"` binding.
    pub level: Box<str>,
}

/// Why a parameter node's declaration cannot be read.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CheckedParameterDeclarationError {
    /// The requested node is absent.
    #[error("unknown checked node")]
    UnknownNode,
    /// The requested node is not a `value`/`parameter` node.
    #[error("checked node is not a parameter")]
    NotParameter,
    /// The parameter body is not the closed name-and-level shape.
    #[error("parameter body is malformed")]
    MalformedBody,
}

impl CheckedPackageV2 {
    /// Reads the declared name and level of an admitted `value`/`parameter` node.
    ///
    /// The node is identified by its id alone; no enclosing function is
    /// returned. On failure this returns one typed error and no partial
    /// declaration.
    pub fn parameter_declaration(
        &self,
        node: &CheckedNodeId,
    ) -> Result<CheckedParameterDeclaration, CheckedParameterDeclarationError> {
        use CheckedParameterDeclarationError as Error;

        let kind_of = |id: &CheckedNodeId| {
            let position = *self.node_index.get(id)?;
            self.kinds.get(position).copied()
        };
        let found = self.node(node).ok_or(Error::UnknownNode)?;
        if kind_of(node) != Some(CheckedNodeKind::Value(ValueForm::Parameter)) {
            return Err(Error::NotParameter);
        }
        let position = *self
            .node_index
            .get(&found.node_id)
            .ok_or(Error::UnknownNode)?;
        let (name, level) = self
            .terms
            .get(position)
            .and_then(|term| typed_parameter_binding(term, &kind_of))
            .ok_or(Error::MalformedBody)?;
        Ok(CheckedParameterDeclaration {
            node: node.clone(),
            name: name.into(),
            level: level.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::test_support::{id, node_id};
    use crate::checked_package::v2::{
        CheckedNodeTag, CheckedPackageWireV2, CheckedTerm, RetainedModels,
    };
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    type Mutation = Box<dyn Fn(&mut CheckedPackageV2)>;

    fn literal(ty: char, value_kind: &str, value: &str) -> Value {
        json!({"term": "literal", "type": id(ty), "value_kind": value_kind, "value": value})
    }

    fn parameter_body(name: Value, level: Value) -> Value {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": name},
            {"term": "binding", "name": "level", "value": level},
        ]})
    }

    /// The accessor reads the retained typed terms. This private fixture has
    /// the other wire members needed to construct a package, then tests may
    /// mutate the graph in states admission would ordinarily reject. The
    /// package is never admitted (its keys are not derived); only the shapes
    /// and the decoded kinds are those of an admitted one. Nodes:
    /// `1` text, `2` integer, `3` parameter `a` at level `10`, `4` a literal.
    fn fixture_package() -> CheckedPackageV2 {
        let node = |digit: char, tag: &str, form: &str, ty: char, body: Value| {
            json!({
                "node_id": id(digit), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag, "semantic_form": form, "semantic_type": id(ty),
                "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
                "body": body,
            })
        };
        let empty = || json!({"term": "aggregate", "members": []});
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
                node('1', "scalar_type", "text", '1', empty()),
                node('2', "scalar_type", "integer", '2', empty()),
                node('3', "value", "parameter", '2', parameter_body(
                    literal('1', "text", "a"), literal('2', "integer", "10"))),
                node('4', "value", "literal", '2', literal('2', "integer", "7")),
            ]},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []},
        }))
        .expect("private package fixture");
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
            .map(|node| CheckedTerm::decode_admitted(&node.body).expect("fixture body"))
            .collect();
        CheckedPackageV2 {
            node_index: super::super::retained_node_index(&wire),
            source_index: super::super::retained_source_index(&wire),
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

    #[trace("TC-048", "FR-038-AC-258", "FR-038-AC-259")]
    #[test]
    fn tc_048_parameter_declaration_reads_a_healthy_body_and_refuses_other_nodes() {
        let package = fixture_package();
        let declaration = package
            .parameter_declaration(&node_id('3'))
            .expect("healthy parameter");
        assert_eq!(declaration.node, node_id('3'));
        assert_eq!(declaration.name.as_ref(), "a");
        assert_eq!(declaration.level.as_ref(), "10");
        assert_eq!(
            package.parameter_declaration(&node_id('9')),
            Err(CheckedParameterDeclarationError::UnknownNode)
        );
        for other in ['1', '4'] {
            assert_eq!(
                package.parameter_declaration(&node_id(other)),
                Err(CheckedParameterDeclarationError::NotParameter),
                "{other}"
            );
        }
    }

    #[trace("TC-048", "FR-349-AC-2")]
    #[test]
    fn admitted_parameter_lookup_reads_the_retained_typed_body() {
        let mut package = fixture_package();
        package.terms[2] = CheckedTerm::Reference {
            target: node_id('4'),
        };
        assert_eq!(
            package.parameter_declaration(&node_id('3')),
            Err(CheckedParameterDeclarationError::MalformedBody)
        );
    }

    #[trace("TC-048", "FR-038-AC-259")]
    #[test]
    fn tc_048_post_admission_body_mutations_refuse_malformed_body() {
        let mutations: [(&str, Mutation); 13] = [
            (
                "missing name binding",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body = json!({"term": "aggregate",
                        "members": [{"term": "binding", "name": "level",
                            "value": literal('2', "integer", "10")}]});
                }),
            ),
            (
                "name binding renamed",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][0]["name"] =
                        json!("title");
                }),
            ),
            (
                "body is not an aggregate",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body = literal('1', "text", "a");
                }),
            ),
            (
                "body is a reference",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body =
                        json!({"term": "reference", "target": id('4')});
                }),
            ),
            (
                "name literal of integer kind typed at the text type",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][0]["value"] =
                        literal('1', "integer", "a");
                }),
            ),
            (
                "level literal of text kind typed at the integer type",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][1]["value"] =
                        literal('2', "text", "10");
                }),
            ),
            (
                "level binding renamed",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][1]["name"] =
                        json!("depth");
                }),
            ),
            (
                "non-text name",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][0]["value"] =
                        literal('2', "integer", "5");
                }),
            ),
            (
                "non-identifier name",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][0]["value"] =
                        literal('1', "text", "1x");
                }),
            ),
            (
                "name typed at a node that is not scalar_type/text",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][0]["value"] =
                        literal('2', "text", "a");
                }),
            ),
            (
                "name typed at a node that is not a scalar_type",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][0]["value"] =
                        literal('3', "text", "a");
                }),
            ),
            (
                "non-canonical level",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][1]["value"] =
                        literal('2', "integer", "010");
                }),
            ),
            (
                "level typed at the text type",
                Box::new(|package| {
                    package.wire.semantic_graph.nodes[2].body["members"][1]["value"] =
                        literal('1', "integer", "10");
                }),
            ),
        ];
        for (label, mutate) in mutations {
            let mut package = fixture_package();
            mutate(&mut package);
            package.terms[2] =
                CheckedTerm::decode_admitted(&package.wire.semantic_graph.nodes[2].body)
                    .expect("mutated fixture retains a grammar-admitted term");
            assert_eq!(
                package.parameter_declaration(&node_id('3')),
                Err(CheckedParameterDeclarationError::MalformedBody),
                "{label}"
            );
        }
    }
}
