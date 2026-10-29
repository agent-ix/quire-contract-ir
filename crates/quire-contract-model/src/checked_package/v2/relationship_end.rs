//! Typed destination end of an admitted `quire.op.model.reaches` application.

use super::model_members::SelectedEnds;
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

/// Why a node is not a reaches application.
pub(super) enum ReachesLookup {
    /// The id is absent from the graph.
    Unknown,
    /// The node's body root is not a `quire.op.model.reaches` application.
    NotReaches,
}

impl CheckedPackageV2 {
    /// The node `id` when its body root is a `quire.op.model.reaches`
    /// application: the one rule both reaches accessors share.
    pub(super) fn reaches_application(&self, id: &CheckedNodeId) -> Result<&Value, ReachesLookup> {
        self.node(id).ok_or(ReachesLookup::Unknown)?;
        let Some(super::CheckedTerm::Application { operation, .. }) = self.term(id) else {
            return Err(ReachesLookup::NotReaches);
        };
        (operation.get("identity").and_then(Value::as_str) == Some("quire.op.model.reaches"))
            .then_some(operation.record())
            .ok_or(ReachesLookup::NotReaches)
    }

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

        let node_at = |id: &CheckedNodeId| self.node(id);
        let operation = self
            .reaches_application(application)
            .map_err(|lookup| match lookup {
                ReachesLookup::Unknown => Error::UnknownNode,
                ReachesLookup::NotReaches => Error::NotReachesApplication,
            })?;
        let member = operation.get("member").ok_or(Error::UnresolvedEnd)?;
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
            .and_then(|declaration| {
                self.term(&declaration.node_id)
                    .and_then(|term| self.models.relationship(declaration, term))
            })
            .ok_or(Error::UnresolvedEnd)?;
        let SelectedEnds {
            destination,
            forward,
            ..
        } = retained
            .declaration
            .select_ends(name)
            .map_err(|_| Error::UnresolvedEnd)?;
        let digest = retained
            .destination_key(forward)
            .ok_or(Error::UnresolvedEnd)?;
        Ok(CheckedRelationshipEnd {
            application: application.clone(),
            relationship,
            role: destination.role.clone(),
            destination_type: CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: digest.into(),
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
pub(super) mod tests {
    use super::*;
    use crate::checked_package::v2::model_fields::RetainedRelationship;
    use crate::checked_package::v2::test_support::node_id;
    use crate::checked_package::v2::CheckedTerm;
    use crate::checked_package::v2::RetainedModels;
    use crate::{CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2ReadResult};
    use ix_trace_rs::trace;
    use serde_json::json;
    use std::collections::BTreeMap;

    type Mutation = Box<dyn Fn(&mut CheckedPackageV2, &BTreeMap<char, CheckedNodeId>)>;

    const IDENTITY: &str = "acme/orders";
    const ORDER: &str = "ix://acme/orders/Order";
    const LINKS: &str = "ix://acme/orders/relationship/Order-links-Order";
    const SAME: &str = "ix://acme/orders/relationship/Order-same-Order";

    pub(in crate::checked_package::v2) fn raw_id(digit: char) -> Value {
        json!({"domain": NODE_DOMAIN, "digest": digit.to_string().repeat(64)})
    }

    fn end(role: &str, lower: u64, upper: u64, unique: bool) -> Value {
        json!({"type": ORDER, "role": role, "multiplicity":
            {"lower": lower, "upper": upper, "ordered": false, "unique": unique}})
    }

    fn relationship(identity: &str, source: Value, target: Value) -> Value {
        json!({"identity": identity, "category": "structural", "composite": false,
            "direction": "bidirectional", "sourceEnd": source, "targetEnd": target,
            "origin": {"source": {"sourceIdentity": ORDER, "path": "models/Order.md",
                "startLine": 1, "startColumn": 1}}})
    }

    /// The selected domain package: `Order` with a relationship of unequal
    /// ends (`links` -> `peer`) and one whose ends share the role `same`,
    /// which no application can name and be admitted.
    fn document() -> Value {
        json!({
            "contractVersion": "2.0.0",
            "package": {"identity": IDENTITY, "version": "1.0.0"},
            "constructs": [{
                "kind": {"module": "acme/orders", "name": "entity"},
                "construct": {"meaning": "quire.meaning.model.object-type/v1"},
            }],
            "types": [{
                "identity": ORDER, "displayName": "Order",
                "kind": {"module": "acme/orders", "name": "entity"},
                "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
                "supertypes": [], "fields": [], "operations": [],
                "relationships": [
                    relationship(LINKS, end("links", 1, 1, false), end("peer", 2, 3, true)),
                    relationship(SAME, end("same", 1, 1, true), end("same", 1, 1, true)),
                ],
            }],
        })
    }

    fn replace_id(value: &mut Value, old: &Value, new: &Value) {
        if value == old {
            *value = new.clone();
            return;
        }
        match value {
            Value::Array(members) => {
                for member in members {
                    replace_id(member, old, new);
                }
            }
            Value::Object(members) => {
                for member in members.values_mut() {
                    replace_id(member, old, new);
                }
            }
            _ => {}
        }
    }

    /// A package admitted by the reader, with a model declaration node for
    /// `Order`, relation nodes for both relationships, and the reaches
    /// application `7` over `LINKS` naming role `links`. Node keys are the
    /// digests of their own preimages, authored in dependency order. Returns
    /// the selected model root, the package, the evidence and the
    /// placeholder-to-key map.
    pub(in crate::checked_package::v2) fn admitted(
    ) -> (CheckedPackageV2, BTreeMap<char, CheckedNodeId>) {
        let limits = quire_canonical::Limits::new(1 << 20);
        let digest_of = |value: &Value| {
            quire_canonical::sha256(value, limits)
                .expect("fixture digest")
                .to_string()
        };
        let empty = json!({"term": "aggregate", "members": []});
        let owner = |node: &str| json!({"kind": "model", "identity": IDENTITY, "node": node});
        let node = |digit, tag, form, ty, role, body: Value| {
            json!({"node_id": raw_id(digit), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag, "semantic_form": form, "semantic_type": raw_id(ty),
                "dependencies": [], "occurrences": [{"role": role, "ordinal": 0}], "body": body})
        };
        let parameter = |name: &str| {
            json!({"term": "aggregate", "members": [
                {"term": "binding", "name": "name", "value":
                    {"term": "literal", "type": raw_id('3'), "value_kind": "text", "value": name}},
                {"term": "binding", "name": "level", "value":
                    {"term": "literal", "type": raw_id('2'), "value_kind": "integer", "value": "0"}},
            ]})
        };
        let mut nodes = vec![
            node('1', "scalar_type", "boolean", '1', "type", empty.clone()),
            node('2', "scalar_type", "integer", '2', "type", empty.clone()),
            node('3', "scalar_type", "text", '3', "type", empty.clone()),
            node('a', "model", "object_type", 'a', "type", empty.clone()),
            node(
                'b',
                "composite_type",
                "reference",
                'b',
                "type",
                json!({"term": "aggregate",
                "members": [{"term": "reference", "target": raw_id('a')}]}),
            ),
            node(
                '5',
                "value",
                "parameter",
                'b',
                "expression",
                parameter("left"),
            ),
            node(
                '8',
                "value",
                "parameter",
                'b',
                "expression",
                parameter("right"),
            ),
            node('c', "relation", "relationship", 'c', "type", empty.clone()),
            node('d', "relation", "relationship", 'd', "type", empty),
            node(
                '7',
                "expression",
                "reachability",
                '1',
                "expression",
                json!({
                    "term": "application", "operator": "reaches",
                    "operation": {"identity": "quire.op.model.reaches", "laws": [], "mode": null,
                        "leaves": [], "member": {"kind": "relationship_end",
                            "declaration": raw_id('c'), "name": "links"}},
                    "result_type": raw_id('1'),
                    "arguments": [{"term": "reference", "target": raw_id('5')},
                        {"term": "reference", "target": raw_id('8')}],
                }),
            ),
        ];
        for (digit, node_name) in [('a', ORDER), ('c', LINKS), ('d', SAME)] {
            let at = nodes
                .iter()
                .position(|n| n["node_id"] == raw_id(digit))
                .expect("node");
            nodes[at]["owner"] = owner(node_name);
        }
        let mut ids: BTreeMap<char, CheckedNodeId> = BTreeMap::new();
        for digit in ['1', '2', '3', 'a', 'b', '5', '8', 'c', 'd', '7'] {
            let at = nodes
                .iter()
                .position(|n| n["node_id"] == raw_id(digit))
                .expect("node");
            let current = &nodes[at];
            let semantic_type = if current["node_id"] == current["semantic_type"] {
                Value::Null
            } else {
                current["semantic_type"].clone()
            };
            let mut preimage = json!({
                "version": if digit == '7' { "quire.application-node/v1" } else { "quire.structural-node/v1" },
                "node_tag": current["node_tag"], "semantic_form": current["semantic_form"],
                "semantic_type": semantic_type, "declaration": null, "recursion": null,
                "body": current["body"],
            });
            if !current["owner"].is_null() {
                preimage["owner"] = current["owner"].clone();
            }
            let fresh = json!({"domain": NODE_DOMAIN, "digest": digest_of(&preimage)});
            for other in &mut nodes {
                replace_id(other, &raw_id(digit), &fresh);
            }
            ids.insert(digit, serde_json::from_value(fresh).expect("node id"));
        }
        let by_id = |digit: char| json!(ids[&digit]);
        fn position(nodes: &[Value], id: &Value) -> usize {
            nodes
                .iter()
                .position(|n| n["node_id"] == *id)
                .expect("node")
        }
        let at = position(&nodes, &by_id('b'));
        nodes[at]["dependencies"] = json!([by_id('a')]);
        let mut application_dependencies =
            vec![ids[&'5'].clone(), ids[&'8'].clone(), ids[&'c'].clone()];
        application_dependencies.sort();
        let at = position(&nodes, &by_id('7'));
        nodes[at]["dependencies"] = json!(application_dependencies);

        let document = document();
        let edition =
            json!({"role": "edition", "definition": {"authority": "test", "identity": "edition"}});
        let root_owner = json!({
            "kind": "model_root", "identity": IDENTITY,
        });
        let root_body = json!({"term": "aggregate", "members": []});
        let root_preimage = json!({
            "version": "quire.structural-node/v1", "node_tag": "model",
            "semantic_form": "model_root", "semantic_type": null,
            "declaration": null, "recursion": null,
            "owner": root_owner, "body": root_body,
        });
        let root_id = json!({"domain": NODE_DOMAIN, "digest": digest_of(&root_preimage)});
        nodes.push(json!({
            "node_id": root_id, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "model", "semantic_form": "model_root", "semantic_type": root_id,
            "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
            "owner": root_owner, "body": root_body,
        }));
        let source = json!({"authority": "test", "identity": "relationship-end"});
        let source_map: Vec<Value> = nodes
            .iter()
            .enumerate()
            .map(|(at, n)| {
                json!({"node_id": n["node_id"], "role": n["occurrences"][0]["role"], "ordinal": 0,
                    "regions": [{"source": source, "start": at, "end": at + 1}]})
            })
            .collect();
        let projection: Vec<Value> = nodes
            .iter()
            .cloned()
            .map(|mut n| {
                n.as_object_mut().expect("node").remove("occurrences");
                n
            })
            .collect();
        let preimage = json!({
            "version": "quire.checked-package-id/v2", "edition": edition,
            "profile_selections": [], "definition_selections": [],
            "required_features": [],
            "dependency_selections": [], "identity_projection": projection,
        });
        let mut wire_selection = preimage.clone();
        wire_selection
            .as_object_mut()
            .expect("selection")
            .remove("identity_projection");
        let wire = json!({
            "contract_version": "quire.checked-package/v2",
            "package_id": {"domain": "quire.package.semantic/v2", "algorithm": "sha256",
                "digest": digest_of(&preimage)},
            "identity_preimage": wire_selection,
            "lock": {"sources": [source], "edition": edition, "profile_selections": [],
                "definition_selections": [],
                "required_features": [], "dependency_selections": []},
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
            "source_map": source_map, "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"},
                "entries": []},
        });
        let mut evidence = CheckedPackageEvidence::new();
        evidence
            .insert_domain_package_document(
                IDENTITY,
                serde_json::to_vec(&document).expect("document bytes"),
            )
            .expect("evidence");
        match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("fixture bytes"),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => (*package, ids),
            other => panic!("the fixture must be reader-admitted before mutation: {other:?}"),
        }
    }

    /// The graph position of `id`.
    fn at(package: &CheckedPackageV2, id: &CheckedNodeId) -> usize {
        package.node_index[id]
    }

    fn operation<'a>(
        package: &'a mut CheckedPackageV2,
        ids: &BTreeMap<char, CheckedNodeId>,
    ) -> &'a mut Value {
        let position = at(package, &ids[&'7']);
        let super::super::CheckedTerm::Application { operation, .. } = &mut package.terms[position]
        else {
            panic!("admitted reaches application term");
        };
        operation.record_mut()
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn relationship_end_uses_retained_operation_after_raw_body_changes() {
        let (mut package, ids) = admitted();
        let expected = package.relationship_end(&ids[&'7']).expect("admitted end");
        let position = at(&package, &ids[&'7']);
        package.wire.semantic_graph.nodes[position].body = CheckedTerm::aggregate(Vec::new());
        assert_eq!(package.relationship_end(&ids[&'7']), Ok(expected));
    }

    /// The admitted application resolves to the retained destination end, and
    /// the retained record is outside package equality.
    #[trace("TC-048", "FR-038-AC-266")]
    #[test]
    fn tc_048_admitted_retention_is_read_and_outside_package_equality() {
        let (package, ids) = admitted();
        let forward = package.relationship_end(&ids[&'7']).expect("forward");
        assert_eq!(forward.relationship, ids[&'c']);
        assert_eq!(forward.destination_type, ids[&'a']);
        assert_eq!((forward.lower, forward.upper), (2, Some(3)));
        assert_eq!(forward.role.as_deref(), Some("peer"));
        assert!(forward.forward);

        let mut inverse = package.clone();
        operation(&mut inverse, &ids)["member"]["name"] = json!("peer");
        let inverse = inverse.relationship_end(&ids[&'7']).expect("inverse");
        assert_eq!(
            (inverse.lower, inverse.upper, inverse.forward),
            (1, Some(1), false)
        );
        assert_eq!(inverse.role.as_deref(), Some("links"));
        assert_eq!(inverse.destination_type, ids[&'a']);

        let mut without_record = package.clone();
        without_record.models = RetainedModels::default();
        assert_eq!(package, without_record);
        assert_eq!(
            without_record.relationship_end(&ids[&'7']),
            Err(CheckedRelationshipEndError::UnresolvedEnd)
        );
    }

    /// Each state below mutates a reader-admitted application (or the
    /// relation node it names) after admission, so the lookup from
    /// `RetainedModels::new` through `RetainedModels::relationship` is run on
    /// the record admission built.
    #[trace("TC-048", "FR-038-AC-265")]
    #[test]
    fn tc_048_post_admission_mutations_of_an_admitted_application_refuse_unresolved_end() {
        let mutations: [(&str, Mutation); 11] = [
            (
                "member removed",
                Box::new(|p, ids| {
                    operation(p, ids)
                        .as_object_mut()
                        .expect("operation")
                        .remove("member");
                }),
            ),
            (
                "member null",
                Box::new(|p, ids| operation(p, ids)["member"] = Value::Null),
            ),
            (
                "member kind is not relationship_end",
                Box::new(|p, ids| operation(p, ids)["member"]["kind"] = json!("field")),
            ),
            (
                "declaration is not a node id",
                Box::new(|p, ids| operation(p, ids)["member"]["declaration"] = json!(7)),
            ),
            (
                "declaration names a node outside the declaration index",
                Box::new(|p, ids| operation(p, ids)["member"]["declaration"] = json!(ids[&'1'])),
            ),
            (
                "declaration names an indexed declaration of another form",
                Box::new(|p, ids| operation(p, ids)["member"]["declaration"] = json!(ids[&'a'])),
            ),
            (
                "declaration node no longer has the declaration shape",
                Box::new(|p, ids| {
                    let position = at(p, &ids[&'c']);
                    p.terms[position] = CheckedTerm::Aggregate {
                        members: vec![CheckedTerm::Reference {
                            target: ids[&'1'].clone(),
                        }],
                    };
                }),
            ),
            (
                "name is not text",
                Box::new(|p, ids| operation(p, ids)["member"]["name"] = json!(3)),
            ),
            (
                "name matches neither role",
                Box::new(|p, ids| operation(p, ids)["member"]["name"] = json!("other")),
            ),
            (
                "name matches both roles of the named relationship",
                Box::new(|p, ids| {
                    operation(p, ids)["member"]["declaration"] = json!(ids[&'d']);
                    operation(p, ids)["member"]["name"] = json!("same");
                }),
            ),
            (
                "name matches a role of another relationship",
                Box::new(|p, ids| {
                    operation(p, ids)["member"]["declaration"] = json!(ids[&'d']);
                }),
            ),
        ];
        for (label, mutate) in mutations {
            let (mut package, ids) = admitted();
            mutate(&mut package, &ids);
            assert_eq!(
                package.relationship_end(&ids[&'7']),
                Err(CheckedRelationshipEndError::UnresolvedEnd),
                "{label}"
            );
        }
    }

    /// The retained destination key cannot be absent in a package the reader
    /// admitted (admission refuses an end whose type is not a selected
    /// declaration), so this one state is produced by editing the admitted
    /// retained record itself. It backs no acceptance criterion by itself.
    #[test]
    fn tc_048_an_absent_retained_destination_key_refuses_unresolved_end() {
        let (mut package, ids) = admitted();
        let position = at(&package, &ids[&'c']);
        let retained = package
            .models
            .relationship(
                &package.wire.semantic_graph.nodes[position],
                &package.terms[position],
            )
            .cloned()
            .expect("the admitted record");
        package.models = RetainedModels::with_relationship(
            ids[&'c'].digest.as_ref(),
            0,
            LINKS,
            RetainedRelationship {
                target_key: None,
                ..retained
            },
        );
        assert_eq!(
            package.relationship_end(&ids[&'7']),
            Err(CheckedRelationshipEndError::UnresolvedEnd)
        );
    }

    /// Ids that are not in the graph and nodes that are not reaches
    /// applications refuse by their own variants (the full AC-264 evidence is
    /// the integration test over admitted navigate, add and field-read nodes).
    #[test]
    fn tc_048_unknown_and_non_reaches_nodes_refuse_by_variant() {
        let (package, ids) = admitted();
        assert_eq!(
            package.relationship_end(&node_id('f')),
            Err(CheckedRelationshipEndError::UnknownNode)
        );
        for other in ['1', 'a', 'c', '5'] {
            assert_eq!(
                package.relationship_end(&ids[&other]),
                Err(CheckedRelationshipEndError::NotReachesApplication),
                "{other}"
            );
        }
    }
}
