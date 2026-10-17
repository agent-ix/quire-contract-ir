//! Typed destination end of an admitted `quire.op.model.reaches` application.

use super::{CheckedNodeId, CheckedPackageV2};
use crate::checked_package::common::NODE_DOMAIN;
use serde_json::Value;
use thiserror::Error;

/// The relationship end a reaches application names, as the reader resolved it.
///
/// `destination_type` is the model declaration node the reader derived for the
/// declared end type; the graph need not hold that node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedRelationshipEnd {
    /// The reaches application node that was read.
    pub application: CheckedNodeId,
    /// The relationship declaration node the application names.
    pub relationship: CheckedNodeId,
    /// The destination end's authored role; `None` when it carries none.
    pub role: Option<Box<str>>,
    /// The derived model declaration node of the destination end's type.
    pub destination_type: CheckedNodeId,
    /// The destination end's declared lower bound.
    pub lower: u64,
    /// The destination end's declared upper bound; `None` when unbounded.
    pub upper: Option<u64>,
    /// Whether the destination end is declared ordered.
    pub ordered: bool,
    /// Whether the destination end is declared unique.
    pub unique: bool,
    /// Whether the application names the source end's role (destination is
    /// the target end) rather than the target end's role.
    pub forward: bool,
}

/// Why a reaches application's relationship end cannot be read.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CheckedRelationshipEndError {
    /// The requested node is absent.
    #[error("unknown checked node")]
    UnknownNode,
    /// The requested node's body root is not a `quire.op.model.reaches`
    /// application.
    #[error("checked node is not a reaches application")]
    NotReachesApplication,
    /// The application's member no longer resolves to a retained relationship
    /// end as admission resolved it.
    #[error("reaches application's relationship end is unresolved")]
    UnresolvedEnd,
}

impl CheckedPackageV2 {
    /// Reads the destination end of an admitted `quire.op.model.reaches`
    /// application from the relationship record the reader retained.
    ///
    /// `application` is the node whose body root is the application. The
    /// accessor charges no work and recomputes no identity. On failure this
    /// returns one typed error and no partial end.
    pub fn relationship_end(
        &self,
        application: &CheckedNodeId,
    ) -> Result<CheckedRelationshipEnd, CheckedRelationshipEndError> {
        use CheckedRelationshipEndError as Error;

        let node_at = |id: &CheckedNodeId| {
            let position = *self.node_index.get(id)?;
            self.wire
                .semantic_graph
                .nodes
                .get(position)
                .filter(|candidate| &candidate.node_id == id)
        };
        let found = node_at(application).ok_or(Error::UnknownNode)?;
        let is_reaches = found.body.get("term").and_then(Value::as_str) == Some("application")
            && found
                .body
                .pointer("/operation/identity")
                .and_then(Value::as_str)
                == Some("quire.op.model.reaches");
        if !is_reaches {
            return Err(Error::NotReachesApplication);
        }
        let member = found
            .body
            .pointer("/operation/member")
            .ok_or(Error::UnresolvedEnd)?;
        if member.get("kind").and_then(Value::as_str) != Some("relationship_end") {
            return Err(Error::UnresolvedEnd);
        }
        let relationship: CheckedNodeId = member
            .get("declaration")
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .ok_or(Error::UnresolvedEnd)?;
        let name = member
            .get("name")
            .and_then(Value::as_str)
            .ok_or(Error::UnresolvedEnd)?;
        let retained = node_at(&relationship)
            .and_then(|declaration| self.models.relationship(declaration))
            .ok_or(Error::UnresolvedEnd)?;
        let declared = &retained.declaration;
        let (destination, key, forward) = match (
            declared.source.role.as_deref() == Some(name),
            declared.target.role.as_deref() == Some(name),
        ) {
            (true, false) => (&declared.target, &retained.target_key, true),
            (false, true) => (&declared.source, &retained.source_key, false),
            (false, false) | (true, true) => return Err(Error::UnresolvedEnd),
        };
        let digest = key.clone().ok_or(Error::UnresolvedEnd)?;
        Ok(CheckedRelationshipEnd {
            application: application.clone(),
            relationship,
            role: destination.role.clone(),
            destination_type: CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest,
            },
            lower: destination.multiplicity.lower,
            upper: destination.multiplicity.upper,
            ordered: destination.multiplicity.ordered,
            unique: destination.multiplicity.unique,
            forward,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::model_fields::RetainedRelationship;
    use crate::checked_package::v2::model_members::{
        Multiplicity, RelationshipDecl, RelationshipDirection, RelationshipEnd,
    };
    use crate::checked_package::v2::{
        CheckedNodeKind, CheckedNodeTag, CheckedPackageWireV2, RetainedModels,
    };
    use ix_trace_rs::trace;
    use serde_json::json;

    type Mutation = Box<dyn Fn(&mut CheckedPackageV2)>;

    fn id(digit: char) -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": digit.to_string().repeat(64)})
    }

    fn node_id(digit: char) -> CheckedNodeId {
        serde_json::from_value(id(digit)).expect("node id")
    }

    fn end(role: Option<&str>, lower: u64, upper: Option<u64>, unique: bool) -> RelationshipEnd {
        RelationshipEnd {
            role: role.map(Box::from),
            type_ref: "T".into(),
            multiplicity: Multiplicity {
                lower,
                upper,
                ordered: false,
                unique,
            },
        }
    }

    fn retained(source: RelationshipEnd, target: RelationshipEnd) -> RetainedRelationship {
        RetainedRelationship {
            declaration: RelationshipDecl {
                source,
                target,
                direction: RelationshipDirection::Bidirectional,
            },
            source_key: Some("8".repeat(64).into()),
            target_key: Some("9".repeat(64).into()),
        }
    }

    /// The accessor reads the semantic graph and the retained relationship
    /// record. This private fixture has the other wire members needed to
    /// construct a package; tests mutate the graph in states admission would
    /// reject. Nodes: `1` scalar type, `5` the relationship declaration (the
    /// retained one), `6` a reaches application naming role `links`, `7` an
    /// application of another operation.
    fn fixture_package(relationship: RetainedRelationship) -> CheckedPackageV2 {
        let node = |digit: char, tag: &str, form: &str, ty: char, body: Value| {
            json!({
                "node_id": id(digit), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag, "semantic_form": form, "semantic_type": id(ty),
                "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
                "body": body,
            })
        };
        let empty = || json!({"term": "aggregate", "members": []});
        let application = |identity: &str| {
            json!({"term": "application", "operator": "reaches", "operation": {
                "identity": identity, "laws": [], "mode": null, "leaves": [],
                "member": {"kind": "relationship_end", "declaration": id('5'), "name": "links"},
            }, "result_type": id('1'), "arguments": []})
        };
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
                node('1', "scalar_type", "boolean", '1', empty()),
                node('5', "relation", "relationship", '5', empty()),
                node('6', "expression", "reachability", '1', application("quire.op.model.reaches")),
                node('7', "expression", "binary", '1', application("quire.op.integer.add")),
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
        CheckedPackageV2 {
            node_index: super::super::retained_node_index(&wire),
            wire,
            kinds,
            bytes: 1 << 20,
            models: RetainedModels::with_relationship(&"5".repeat(64), 0, "REL", relationship),
        }
    }

    fn operation(package: &mut CheckedPackageV2) -> &mut Value {
        &mut package.wire.semantic_graph.nodes[2].body["operation"]
    }

    fn unequal() -> RetainedRelationship {
        retained(
            end(Some("links"), 1, Some(1), false),
            end(Some("peer"), 2, Some(3), true),
        )
    }

    /// Trace: TC-048, FR-038-AC-262, FR-038-AC-264
    #[trace("TC-048", "FR-038-AC-262", "FR-038-AC-264")]
    #[test]
    fn tc_048_relationship_end_reads_the_retained_destination_end() {
        let package = fixture_package(unequal());
        let forward = package.relationship_end(&node_id('6')).expect("forward");
        assert_eq!(forward.application, node_id('6'));
        assert_eq!(forward.relationship, node_id('5'));
        assert_eq!(forward.role.as_deref(), Some("peer"));
        assert_eq!(forward.destination_type.digest.as_ref(), "9".repeat(64));
        assert_eq!(forward.destination_type.domain.as_ref(), NODE_DOMAIN);
        assert_eq!((forward.lower, forward.upper), (2, Some(3)));
        assert_eq!(
            (forward.ordered, forward.unique, forward.forward),
            (false, true, true)
        );

        let mut inverse = fixture_package(unequal());
        inverse.wire.semantic_graph.nodes[2].body["operation"]["member"]["name"] = json!("peer");
        let inverse = inverse.relationship_end(&node_id('6')).expect("inverse");
        assert_eq!(inverse.role.as_deref(), Some("links"));
        assert_eq!(inverse.destination_type.digest.as_ref(), "8".repeat(64));
        assert_eq!((inverse.lower, inverse.upper), (1, Some(1)));
        assert_eq!(
            (inverse.ordered, inverse.unique, inverse.forward),
            (false, false, false)
        );

        // The retained record is outside package equality.
        let mut without_record = package.clone();
        without_record.models = RetainedModels::default();
        assert_eq!(package, without_record);
        assert_eq!(package.relationship_end(&node_id('6')), Ok(forward));
        assert_eq!(
            without_record.relationship_end(&node_id('6')),
            Err(CheckedRelationshipEndError::UnresolvedEnd)
        );

        assert_eq!(
            package.relationship_end(&node_id('9')),
            Err(CheckedRelationshipEndError::UnknownNode)
        );
        for other in ['1', '5', '7'] {
            assert_eq!(
                package.relationship_end(&node_id(other)),
                Err(CheckedRelationshipEndError::NotReachesApplication),
                "{other}"
            );
        }
    }

    /// Trace: TC-048, FR-038-AC-265
    #[trace("TC-048", "FR-038-AC-265")]
    #[test]
    fn tc_048_post_admission_mutations_refuse_unresolved_end() {
        let mutations: [(&str, Mutation); 8] = [
            (
                "member absent",
                Box::new(|package| operation(package)["member"] = Value::Null),
            ),
            (
                "member kind is not relationship_end",
                Box::new(|package| operation(package)["member"]["kind"] = json!("field")),
            ),
            (
                "declaration is not a node id",
                Box::new(|package| operation(package)["member"]["declaration"] = json!(7)),
            ),
            (
                "declaration resolves to no retained relationship",
                Box::new(|package| operation(package)["member"]["declaration"] = id('1')),
            ),
            (
                "name is not text",
                Box::new(|package| operation(package)["member"]["name"] = json!(3)),
            ),
            (
                "name matches neither role",
                Box::new(|package| operation(package)["member"]["name"] = json!("other")),
            ),
            (
                "name matches both roles",
                Box::new(|package| {
                    package.models = RetainedModels::with_relationship(
                        &"5".repeat(64),
                        0,
                        "REL",
                        retained(
                            end(Some("links"), 1, Some(1), false),
                            end(Some("links"), 2, Some(3), true),
                        ),
                    );
                }),
            ),
            (
                "retained destination key is absent",
                Box::new(|package| {
                    package.models = RetainedModels::with_relationship(
                        &"5".repeat(64),
                        0,
                        "REL",
                        RetainedRelationship {
                            target_key: None,
                            ..unequal()
                        },
                    );
                }),
            ),
        ];
        for (label, mutate) in mutations {
            let mut package = fixture_package(unequal());
            mutate(&mut package);
            assert_eq!(
                package.relationship_end(&node_id('6')),
                Err(CheckedRelationshipEndError::UnresolvedEnd),
                "{label}"
            );
        }
    }
}
