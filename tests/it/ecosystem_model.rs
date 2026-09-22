use crate::support::{result_fixture, temporal_fixture};
use ix_trace_rs::trace;
use jsonschema::{Draft, JSONSchema};
use quire_contract_ir::{
    bridge::{BridgeDigest, BridgeLimits, ContractSelection},
    ecosystem_model::{
        self, manifest, EcosystemLimits, ExpectedCampaign, ExpectedRepository, ImprovementProposal,
        ModelCauseCode, MANIFEST_SCHEMA_BYTES, MANIFEST_SCHEMA_SHA256, MODEL_PROFILE,
        MODEL_SCHEMA_BYTES, MODEL_SCHEMA_SHA256,
    },
    predicate,
    temporal::{self, ObservationViews, PositionValuations},
};
use quire_observation::authority::OpenClosed;
use quire_protocol::result::{contract_ir as protocol_map, Limits as ResultLimits, Truth};
use quire_spec_language::protocol_artifact::native_temporal::result::{
    self as native_result, Relation as NativeRelation,
};
use serde_json::{json, Value};
use tl_mltl::{mapping as tl_mapping, wire, wire::OwnerLimits};

const QSPEC: &str = "983b0b28c479241fb066cbe4db3fc0980362de36";
const TL_SYNTAX: &str = "842d82553f045eb69a7f38745756d968254fc25e";
const TL_PARSE: &str = "2bc030dae8fdb30c9ddc967434c6c9902a3905dc";
const TL_MLTL: &str = "22862189ac4eb515ab84928faec25b2eac47d835";
const TL_REWRITE: &str = "c416951281c34e2b9d30187d401605f30f34a18b";
const QSL: &str = "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07";
const QOBS: &str = "9ac80e93f4b68a2c7d5a337f9a448ad10de798fc";
const QPROTOCOL: &str = "34d1752e6c5f789a52ccf115b0694eedd96cdd46";
const QCI: &str = "0c450731626f40fd90c99e787cc0f7f5e053904c";
const CAMPAIGN: &str = "agent-ix/tl-syntax#52/PLAN-010/Task-011";

fn repository_revisions() -> [(&'static str, &'static str); 9] {
    [
        ("agent-ix/quire-contract-ir", QCI),
        ("agent-ix/quire-observation", QOBS),
        ("agent-ix/quire-protocol", QPROTOCOL),
        ("agent-ix/quire-spec-language", QSL),
        ("agent-ix/quire-specification", QSPEC),
        ("agent-ix/tl-mltl", TL_MLTL),
        ("agent-ix/tl-parse", TL_PARSE),
        ("agent-ix/tl-rewrite", TL_REWRITE),
        ("agent-ix/tl-syntax", TL_SYNTAX),
    ]
}

fn expected(bytes: &[u8]) -> ExpectedCampaign {
    let repositories = repository_revisions()
        .map(|(identity, revision)| ExpectedRepository::new(identity, revision));
    ExpectedCampaign::new(CAMPAIGN, repositories, BridgeDigest::raw(bytes))
}

fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("canonical JSON fixture")
}

fn push_edge(edges: &mut Vec<Value>, kind: &str, source: &str, target: &str) {
    edges.push(json!({"kind": kind, "source": source, "target": target}));
}

fn manifest_value() -> Value {
    let mut nodes: Vec<Value> = repository_revisions()
        .into_iter()
        .map(|(identity, revision)| {
            json!({"kind": "repository", "identity": identity, "revision": revision})
        })
        .collect();
    let components = [
        (
            "component:quire-contract-ir-bridge",
            "agent-ix/quire-contract-ir",
        ),
        (
            "component:quire-contract-model",
            "agent-ix/quire-contract-ir",
        ),
        ("component:quire-observation", "agent-ix/quire-observation"),
        ("component:quire-protocol", "agent-ix/quire-protocol"),
        (
            "component:quire-spec-language",
            "agent-ix/quire-spec-language",
        ),
        (
            "component:quire-specification",
            "agent-ix/quire-specification",
        ),
        ("component:tl-mltl", "agent-ix/tl-mltl"),
        ("component:tl-parse", "agent-ix/tl-parse"),
        ("component:tl-rewrite", "agent-ix/tl-rewrite"),
        ("component:tl-syntax", "agent-ix/tl-syntax"),
    ];
    nodes.extend(
        components
            .iter()
            .map(|(identity, _)| json!({"kind": "component", "identity": identity})),
    );
    nodes.extend([
        json!({"kind": "object", "identity": "ix://agent-ix/quire-specification/VO-008", "revision": QSPEC}),
        json!({"kind": "interface", "identity": "ix://agent-ix/tl-syntax/IF-006", "revision": TL_SYNTAX}),
        contract_node(
            "contract:temporal-ecosystem-manifest-v1",
            "quire.contract.temporal-ecosystem-manifest/v1",
            MANIFEST_SCHEMA_SHA256,
        ),
        contract_node(
            "contract:temporal-ecosystem-model-v1",
            "quire.contract.temporal-ecosystem-model/v1",
            MODEL_SCHEMA_SHA256,
        ),
        json!({"kind": "requirement", "identity": "ix://agent-ix/quire-contract-ir/FR-025", "revision": QCI}),
        json!({"kind": "requirement", "identity": "ix://agent-ix/quire-contract-ir/FR-026", "revision": QCI}),
        json!({"kind": "requirement", "identity": "ix://agent-ix/quire-contract-ir/FR-027", "revision": QCI}),
        json!({"kind": "requirement", "identity": "ix://agent-ix/quire-protocol/FR-006", "revision": QPROTOCOL}),
        json!({"kind": "test", "identity": "ix://agent-ix/quire-contract-ir/TC-038", "revision": QCI}),
        json!({"kind": "test", "identity": "ix://agent-ix/quire-contract-ir/TC-039", "revision": QCI}),
        json!({"kind": "test", "identity": "ix://agent-ix/quire-contract-ir/TC-040", "revision": QCI}),
        json!({"kind": "review", "identity": "ix://agent-ix/quire-contract-ir/SR-536", "revision": QCI}),
        json!({"kind": "review", "identity": "ix://agent-ix/quire-contract-ir/SR-537", "revision": QCI}),
    ]);
    for selection in owner_selections() {
        let identity = format!("contract:{}", selection.contract());
        if let Some(existing) = nodes.iter().find(|node| node["identity"] == identity) {
            assert_eq!(
                existing["selection"],
                serde_json::to_value(&selection).expect("selection")
            );
        } else {
            nodes.push(json!({
                "kind": "contract",
                "identity": identity,
                "selection": selection,
            }));
        }
    }
    nodes.sort_by(|left, right| node_key(left).cmp(&node_key(right)));

    let mut edges = Vec::new();
    for (component, repository) in components {
        push_edge(&mut edges, "owns", repository, component);
    }
    let repository_owners = [
        (
            "ix://agent-ix/quire-specification/VO-008",
            "agent-ix/quire-specification",
        ),
        ("ix://agent-ix/tl-syntax/IF-006", "agent-ix/tl-syntax"),
        (
            "ix://agent-ix/quire-contract-ir/FR-025",
            "agent-ix/quire-contract-ir",
        ),
        (
            "ix://agent-ix/quire-contract-ir/FR-026",
            "agent-ix/quire-contract-ir",
        ),
        (
            "ix://agent-ix/quire-contract-ir/FR-027",
            "agent-ix/quire-contract-ir",
        ),
        (
            "ix://agent-ix/quire-protocol/FR-006",
            "agent-ix/quire-protocol",
        ),
        (
            "ix://agent-ix/quire-contract-ir/TC-038",
            "agent-ix/quire-contract-ir",
        ),
        (
            "ix://agent-ix/quire-contract-ir/TC-039",
            "agent-ix/quire-contract-ir",
        ),
        (
            "ix://agent-ix/quire-contract-ir/TC-040",
            "agent-ix/quire-contract-ir",
        ),
        (
            "ix://agent-ix/quire-contract-ir/SR-536",
            "agent-ix/quire-contract-ir",
        ),
        (
            "ix://agent-ix/quire-contract-ir/SR-537",
            "agent-ix/quire-contract-ir",
        ),
    ];
    for (node, repository) in repository_owners {
        push_edge(&mut edges, "owns", repository, node);
    }
    for node in &nodes {
        if node["kind"] != "contract" {
            continue;
        }
        let contract = node["identity"].as_str().expect("contract identity");
        let repository = node["selection"]["repository"]
            .as_str()
            .expect("contract repository");
        let component = owner_component(repository);
        push_edge(&mut edges, "owns", repository, contract);
        push_edge(&mut edges, "owns", component, contract);
        push_edge(
            &mut edges,
            "consumes",
            "component:quire-contract-ir-bridge",
            contract,
        );
    }
    for target in [
        "component:quire-contract-model",
        "component:quire-observation",
        "component:quire-protocol",
        "component:quire-spec-language",
        "component:tl-mltl",
        "component:tl-syntax",
    ] {
        push_edge(
            &mut edges,
            "runtime-dependency",
            "component:quire-contract-ir-bridge",
            target,
        );
    }
    let runtime = [
        (
            "component:quire-spec-language",
            "component:quire-contract-model",
        ),
        ("component:quire-protocol", "component:quire-spec-language"),
        ("component:quire-protocol", "component:quire-observation"),
        ("component:tl-parse", "component:tl-syntax"),
        ("component:tl-mltl", "component:tl-syntax"),
        ("component:tl-rewrite", "component:tl-syntax"),
        ("component:tl-rewrite", "component:tl-mltl"),
    ];
    for (source, target) in runtime {
        push_edge(&mut edges, "runtime-dependency", source, target);
    }
    push_edge(
        &mut edges,
        "normative-reference",
        "ix://agent-ix/quire-contract-ir/FR-027",
        "ix://agent-ix/quire-specification/VO-008",
    );
    push_edge(
        &mut edges,
        "normative-reference",
        "ix://agent-ix/quire-contract-ir/FR-027",
        "ix://agent-ix/tl-syntax/IF-006",
    );
    for (test, requirement) in [
        (
            "ix://agent-ix/quire-contract-ir/TC-038",
            "ix://agent-ix/quire-contract-ir/FR-025",
        ),
        (
            "ix://agent-ix/quire-contract-ir/TC-039",
            "ix://agent-ix/quire-contract-ir/FR-026",
        ),
        (
            "ix://agent-ix/quire-contract-ir/TC-040",
            "ix://agent-ix/quire-contract-ir/FR-027",
        ),
        (
            "ix://agent-ix/quire-contract-ir/SR-536",
            "ix://agent-ix/quire-contract-ir/FR-026",
        ),
        (
            "ix://agent-ix/quire-contract-ir/SR-537",
            "ix://agent-ix/quire-contract-ir/FR-026",
        ),
    ] {
        push_edge(&mut edges, "verifies", test, requirement);
    }
    edges.sort_by(|left, right| edge_key(left).cmp(&edge_key(right)));
    json!({
        "profile": ecosystem_model::MANIFEST_PROFILE,
        "schema_digest": MANIFEST_SCHEMA_SHA256,
        "campaign": CAMPAIGN,
        "nodes": nodes,
        "edges": edges,
        "gaps": [{
            "identity": "gap:quire-protocol-8",
            "requirement": "ix://agent-ix/quire-protocol/FR-006",
            "description": "Protocol issue 8 remains open beyond the exact result-mapping surface consumed by FR-025 and FR-026"
        }]
    })
}

fn contract_node(identity: &str, contract: &str, schema_digest: &str) -> Value {
    json!({
        "kind": "contract",
        "identity": identity,
        "selection": {
            "contract": contract,
            "package_version": "0.1.0",
            "repository": "agent-ix/quire-contract-ir",
            "revision": QCI,
            "schema_digest": schema_digest
        }
    })
}

fn owner_component(repository: &str) -> &'static str {
    match repository {
        "agent-ix/quire-contract-ir" => "component:quire-contract-ir-bridge",
        "agent-ix/quire-observation" => "component:quire-observation",
        "agent-ix/quire-protocol" => "component:quire-protocol",
        "agent-ix/quire-spec-language" => "component:quire-spec-language",
        "agent-ix/quire-specification" => "component:quire-specification",
        "agent-ix/tl-mltl" => "component:tl-mltl",
        "agent-ix/tl-parse" => "component:tl-parse",
        "agent-ix/tl-rewrite" => "component:tl-rewrite",
        "agent-ix/tl-syntax" => "component:tl-syntax",
        other => panic!("unknown repository {other}"),
    }
}

fn owner_selections() -> Vec<ContractSelection> {
    let temporal = temporal::TargetSelection::current();
    let predicate = predicate::TargetSelection::current();
    [
        predicate.native(),
        predicate.signal_catalog(),
        predicate.proposition_map(),
        temporal.native(),
        temporal.predicate_projection(),
        temporal.proposition_map(),
        temporal.formula_v1(),
        temporal.formula_v2(),
        temporal.past_operators(),
        temporal.position_ledger(),
        temporal.clock(),
        temporal.capture(),
        temporal.progress(),
        temporal.closure(),
        temporal.completeness(),
        temporal.availability(),
        temporal.trace(),
        temporal.history(),
        temporal.history_requirement(),
        temporal.request(),
        temporal.evaluator_report(),
        temporal.native_request(),
        temporal.native_result(),
        temporal.protocol_result(),
        temporal.protocol_mapping(),
        temporal.tl_mapping(),
    ]
    .into_iter()
    .cloned()
    .collect()
}

fn node_key(value: &Value) -> (&str, &str) {
    (
        value["kind"].as_str().expect("node kind"),
        value["identity"].as_str().expect("node identity"),
    )
}

fn edge_key(value: &Value) -> (&str, &str, &str) {
    (
        value["kind"].as_str().expect("edge kind"),
        value["source"].as_str().expect("edge source"),
        value["target"].as_str().expect("edge target"),
    )
}

fn sort_manifest(value: &mut Value) {
    value["nodes"]
        .as_array_mut()
        .expect("nodes")
        .sort_by(|left, right| node_key(left).cmp(&node_key(right)));
    value["edges"]
        .as_array_mut()
        .expect("edges")
        .sort_by(|left, right| edge_key(left).cmp(&edge_key(right)));
    value["gaps"]
        .as_array_mut()
        .expect("gaps")
        .sort_by(|left, right| {
            left["identity"]
                .as_str()
                .expect("gap identity")
                .cmp(right["identity"].as_str().expect("gap identity"))
        });
}

fn read_value(
    value: &Value,
    limits: EcosystemLimits,
) -> Result<manifest::CheckedManifestSet, ecosystem_model::ModelDecision> {
    let bytes = canonical(value);
    manifest::read(&bytes, &expected(&bytes), limits)
}

#[trace("TC-040", "FR-027-AC-1", "FR-027-AC-3", "FR-027-AC-5", "FR-027-AC-6")]
#[test]
fn tc_040_exact_manifest_exports_and_rereads_one_non_authoritative_model() {
    let mut cause_labels = std::collections::BTreeSet::new();
    let registry = include_str!("../../spec/contract/STD-001-diagnostic-registry.md");
    for code in ModelCauseCode::all() {
        assert!(cause_labels.insert(code.as_str()));
        assert!(registry.contains(&format!("| `{}` |", code.as_str())));
    }
    let value = manifest_value();
    let bytes = canonical(&value);
    let checked = manifest::read(&bytes, &expected(&bytes), EcosystemLimits::default())
        .expect("exact campaign manifest");
    assert_eq!(
        checked.nodes().len(),
        value["nodes"].as_array().expect("nodes").len()
    );
    assert_eq!(checked.byte_digest(), BridgeDigest::raw(&bytes));
    let model = ecosystem_model::export(&checked, EcosystemLimits::default())
        .expect("bounded model export");
    assert_eq!(model.manifest_identity(), checked.identity());
    assert_eq!(model.counts().repositories, 9);
    assert_eq!(model.counts().gaps, 1);
    assert_eq!(model.adjacency().len(), model.topological_order().len());
    let validated = ecosystem_model::read(model.bytes(), &checked, EcosystemLimits::default())
        .expect("independent model re-export");
    assert_eq!(validated.identity(), model.identity());
    assert_eq!(validated.byte_digest(), model.byte_digest());

    let proposal = ImprovementProposal::new(
        &validated,
        "close the remaining protocol result contract through owner review",
        EcosystemLimits::default(),
    )
    .expect("descriptive proposal");
    assert_eq!(proposal.source_model_identity(), validated.identity());
    assert_ne!(proposal.identity(), validated.identity());
    assert!(!proposal.description().is_empty());

    let mut permuted = value.clone();
    permuted["nodes"].as_array_mut().expect("nodes").reverse();
    permuted["edges"].as_array_mut().expect("edges").reverse();
    sort_manifest(&mut permuted);
    assert_eq!(canonical(&permuted), bytes);

    let semantic_mutations = [
        mutate(&value, |changed| {
            changed["nodes"]
                .as_array_mut()
                .expect("nodes")
                .iter_mut()
                .find(|node| node["identity"] == "ix://agent-ix/quire-contract-ir/SR-537")
                .expect("review")["revision"] = json!("69ec82bf4da1bdbee710544a4570c2042dc781a0");
        }),
        mutate(&value, |changed| {
            changed["nodes"]
                .as_array_mut()
                .expect("nodes")
                .iter_mut()
                .find(|node| node["identity"] == "contract:temporal-ecosystem-model-v1")
                .expect("contract")["selection"]["schema_digest"] = json!("00".repeat(32));
        }),
        mutate(&value, |changed| {
            changed["edges"].as_array_mut().expect("edges").push(json!({
                "kind": "normative-reference",
                "source": "ix://agent-ix/quire-contract-ir/FR-026",
                "target": "ix://agent-ix/quire-specification/VO-008"
            }));
        }),
        mutate(&value, |changed| {
            changed["gaps"][0]["description"] =
                json!("owner issue remains open after the exact mapping merge");
        }),
        mutate(&value, |changed| {
            for node in changed["nodes"].as_array_mut().expect("nodes") {
                if node["identity"] == "ix://agent-ix/quire-specification/VO-008" {
                    node["identity"] = json!("ix://agent-ix/quire-specification/VO-008-v2");
                }
            }
            for edge in changed["edges"].as_array_mut().expect("edges") {
                if edge["source"] == "ix://agent-ix/quire-specification/VO-008" {
                    edge["source"] = json!("ix://agent-ix/quire-specification/VO-008-v2");
                }
                if edge["target"] == "ix://agent-ix/quire-specification/VO-008" {
                    edge["target"] = json!("ix://agent-ix/quire-specification/VO-008-v2");
                }
            }
        }),
    ];
    for mut changed in semantic_mutations {
        sort_manifest(&mut changed);
        let changed_bytes = canonical(&changed);
        let changed_manifest = manifest::read(
            &changed_bytes,
            &expected(&changed_bytes),
            EcosystemLimits::default(),
        )
        .expect("well-formed semantic mutation");
        let changed_model = ecosystem_model::export(&changed_manifest, EcosystemLimits::default())
            .expect("mutated model");
        assert_ne!(changed_model.identity(), model.identity());
        assert_ne!(changed_manifest.byte_digest(), checked.byte_digest());
    }

    let manifest_schema: Value =
        serde_json::from_slice(MANIFEST_SCHEMA_BYTES).expect("manifest schema");
    let model_schema: Value = serde_json::from_slice(MODEL_SCHEMA_BYTES).expect("model schema");
    let manifest_validator = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&manifest_schema)
        .expect("compiled manifest schema");
    assert!(manifest_validator.is_valid(&value));
    let model_validator = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .with_document(
            manifest_schema["$id"]
                .as_str()
                .expect("manifest schema id")
                .to_owned(),
            manifest_schema,
        )
        .compile(&model_schema)
        .expect("compiled model schema");
    let model_value: Value = serde_json::from_slice(model.bytes()).expect("model JSON");
    assert!(model_validator.is_valid(&model_value));
    assert_eq!(
        BridgeDigest::raw(MANIFEST_SCHEMA_BYTES).to_string(),
        MANIFEST_SCHEMA_SHA256
    );
    assert_eq!(
        BridgeDigest::raw(MODEL_SCHEMA_BYTES).to_string(),
        MODEL_SCHEMA_SHA256
    );
}

#[trace("TC-040", "FR-027-AC-1", "FR-027-AC-5", "FR-027-AC-6")]
#[test]
fn tc_040_manifest_selection_executes_the_real_owner_bridge_path_end_to_end() {
    let manifest_value = manifest_value();
    let manifest_bytes = canonical(&manifest_value);
    let checked = manifest::read(
        &manifest_bytes,
        &expected(&manifest_bytes),
        EcosystemLimits::default(),
    )
    .expect("exact campaign manifest");
    let model =
        ecosystem_model::export(&checked, EcosystemLimits::default()).expect("campaign model");
    let validated_model =
        ecosystem_model::read(model.bytes(), &checked, EcosystemLimits::default())
            .expect("strict campaign model");

    let fixture = result_fixture::fixture();
    let predicates = predicate::project(
        fixture.predicates(),
        predicate::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let decision =
        temporal_fixture::event_position(&fixture, "ecosystem-decision", OpenClosed::Closed);
    let surrounding =
        temporal_fixture::event_position(&fixture, "ecosystem-surrounding", OpenClosed::Closed);
    let protocol_result = result_fixture::validated_result_for_temporal(
        &fixture,
        &decision.progress,
        &decision.closure,
        &surrounding.progress,
        &surrounding.closure,
        &decision.completeness,
        &decision.observation_identity,
        Truth::Violated,
    );
    let mapped = protocol_map::map(
        &protocol_result,
        protocol_map::MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("Protocol owner mapping");
    let availability = decision.availability(protocol_result.result_id());
    let predicate_ref = predicates.decision().correspondences()[0].predicate_ref();
    let valuation = predicate::value(
        predicates.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits::default(),
    );
    assert_eq!(valuation.value(), Some(false));
    let cells = [valuation];
    let positions = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &cells,
    }];
    let observations = ObservationViews {
        position_ledger: &decision.position,
        clock: &decision.clock,
        capture: &decision.capture,
        trigger_scope_closure: &decision.closure,
        decision_scope_progress: &decision.progress,
        decision_scope_closure: &decision.closure,
        surrounding_execution_progress: &surrounding.progress,
        surrounding_execution_closure: &surrounding.closure,
        completeness: &decision.completeness,
        availability: &availability,
        activation_guard: None,
        positions: &positions,
        anchor: 0,
    };
    let target = temporal::TargetSelection::current();
    for selection in [
        target.native(),
        target.predicate_projection(),
        target.proposition_map(),
        target.formula_v1(),
        target.formula_v2(),
        target.position_ledger(),
        target.clock(),
        target.capture(),
        target.progress(),
        target.closure(),
        target.completeness(),
        target.availability(),
        target.trace(),
        target.history(),
        target.history_requirement(),
        target.request(),
        target.evaluator_report(),
        target.native_request(),
        target.native_result(),
        target.protocol_result(),
        target.protocol_mapping(),
        target.tl_mapping(),
    ] {
        assert!(manifest_selects_contract(checked.nodes(), selection));
    }
    let projection = temporal::project(
        fixture.temporal(),
        predicates.validated(),
        observations,
        target,
        BridgeLimits::default(),
    )
    .expect("temporal projection");
    let native_document = native_result::evaluate(
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("native owner result");
    let native = native_result::read(
        native_document.bytes(),
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("native owner reader");
    let tl_document = wire::report::evaluate(
        projection.validated().request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("TL owner result");
    let tl = wire::report::read(
        tl_document.bytes(),
        projection.validated().request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("TL owner reader");
    let tl_selection = tl_mapping::contract_ir::MappingSelection::for_result(&tl);
    let tl_map_document =
        tl_mapping::contract_ir::map(&tl, &tl_selection, OwnerLimits::owner_max())
            .expect("TL owner map");
    let tl_mapped = tl_mapping::contract_ir::read(
        tl_map_document.bytes(),
        &tl,
        &tl_selection,
        OwnerLimits::owner_max(),
    )
    .expect("TL map reader");
    let joined = temporal::join(
        projection.validated(),
        Some(&native),
        Some(&tl_mapped),
        None,
        BridgeLimits::default(),
    );
    assert_eq!(joined.kind(), temporal::TemporalJoinKind::Agreement);
    assert_eq!(joined.value(), Some(false));
    let unavailable = temporal::join(
        projection.validated(),
        None,
        Some(&tl_mapped),
        None,
        BridgeLimits::default(),
    );
    assert_eq!(unavailable.kind(), temporal::TemporalJoinKind::Unavailable);
    assert_eq!(unavailable.value(), None);
    assert_eq!(validated_model.identity(), model.identity());
}

fn manifest_selects_contract(
    nodes: &[ecosystem_model::ManifestNode],
    expected: &ContractSelection,
) -> bool {
    nodes.iter().any(|node| {
        matches!(
            node,
            ecosystem_model::ManifestNode::Contract { selection, .. }
                if selection == expected
        )
    })
}

#[trace("TC-040", "FR-027-AC-1", "FR-027-AC-2")]
#[test]
fn tc_040_every_graph_identity_ownership_and_topology_violation_refuses() {
    let base = manifest_value();
    let cases: Vec<(ModelCauseCode, Value)> = vec![
        (
            ModelCauseCode::RepositorySetMismatch,
            mutate(&base, |value| {
                value["nodes"]
                    .as_array_mut()
                    .expect("nodes")
                    .retain(|node| node["identity"] != "agent-ix/tl-parse");
            }),
        ),
        (
            ModelCauseCode::DuplicateNode,
            mutate(&base, |value| {
                let duplicate = value["nodes"].as_array().expect("nodes")[0].clone();
                value["nodes"]
                    .as_array_mut()
                    .expect("nodes")
                    .push(duplicate);
            }),
        ),
        (
            ModelCauseCode::DanglingEdge,
            mutate(&base, |value| {
                value["edges"].as_array_mut().expect("edges").push(json!({"kind":"consumes","source":"component:tl-parse","target":"contract:absent"}));
            }),
        ),
        (
            ModelCauseCode::DuplicateGap,
            mutate(&base, |value| {
                let mut duplicate = value["gaps"][0].clone();
                duplicate["description"] = json!("same gap identity with different content");
                value["gaps"].as_array_mut().expect("gaps").push(duplicate);
            }),
        ),
        (
            ModelCauseCode::MultipleOwners,
            mutate(&base, |value| {
                value["edges"].as_array_mut().expect("edges").push(json!({"kind":"owns","source":"agent-ix/tl-syntax","target":"component:tl-parse"}));
            }),
        ),
        (
            ModelCauseCode::IllTypedEdge,
            mutate(&base, |value| {
                value["edges"].as_array_mut().expect("edges").push(json!({"kind":"runtime-dependency","source":"agent-ix/tl-parse","target":"agent-ix/tl-syntax"}));
            }),
        ),
        (
            ModelCauseCode::SelfEdge,
            mutate(&base, |value| {
                value["edges"].as_array_mut().expect("edges").push(json!({"kind":"runtime-dependency","source":"component:tl-parse","target":"component:tl-parse"}));
            }),
        ),
        (
            ModelCauseCode::DependencyCycle,
            mutate(&base, |value| {
                value["edges"].as_array_mut().expect("edges").push(json!({"kind":"runtime-dependency","source":"component:tl-syntax","target":"component:tl-parse"}));
            }),
        ),
        (
            ModelCauseCode::MovingRevision,
            mutate(&base, |value| {
                value["nodes"]
                    .as_array_mut()
                    .expect("nodes")
                    .iter_mut()
                    .find(|node| node["identity"] == "agent-ix/tl-parse")
                    .expect("tl-parse")["revision"] = Value::String("main".to_owned());
            }),
        ),
    ];
    for (expected_code, mut value) in cases {
        sort_manifest(&mut value);
        let error =
            read_value(&value, EcosystemLimits::default()).expect_err("invalid graph must refuse");
        assert_eq!(error.code(), expected_code, "{error}");
    }

    let mut out_of_order = base;
    out_of_order["nodes"]
        .as_array_mut()
        .expect("nodes")
        .swap(0, 1);
    let error =
        read_value(&out_of_order, EcosystemLimits::default()).expect_err("out-of-order nodes");
    assert_eq!(error.code(), ModelCauseCode::PopulationOutOfOrder);
}

#[trace("TC-040", "FR-027-AC-3")]
#[test]
fn tc_040_manifest_and_model_readers_reject_hostile_or_replayed_bytes() {
    let value = manifest_value();
    let bytes = canonical(&value);
    let checked =
        manifest::read(&bytes, &expected(&bytes), EcosystemLimits::default()).expect("manifest");
    let model = ecosystem_model::export(&checked, EcosystemLimits::default()).expect("model");

    let mut trailing = bytes.clone();
    trailing.push(b'\n');
    assert_eq!(
        manifest::read(&trailing, &expected(&trailing), EcosystemLimits::default())
            .expect_err("trailing manifest data")
            .code(),
        ModelCauseCode::InvalidDocument
    );
    let mut unknown = value.clone();
    unknown
        .as_object_mut()
        .expect("manifest object")
        .insert("unknown".to_owned(), json!(true));
    assert_eq!(
        read_value(&unknown, EcosystemLimits::default())
            .expect_err("unknown field")
            .code(),
        ModelCauseCode::InvalidDocument
    );
    let duplicate = format!(
        "{{\"campaign\":\"{CAMPAIGN}\",{}",
        &String::from_utf8(bytes.clone()).expect("UTF-8")[1..]
    );
    assert_eq!(
        manifest::read(
            duplicate.as_bytes(),
            &expected(duplicate.as_bytes()),
            EcosystemLimits::default()
        )
        .expect_err("duplicate field")
        .code(),
        ModelCauseCode::InvalidDocument
    );

    let mut wrong_campaign = value.clone();
    wrong_campaign["campaign"] = json!("agent-ix/tl-syntax#foreign");
    let foreign_bytes = canonical(&wrong_campaign);
    assert_eq!(
        manifest::read(
            &foreign_bytes,
            &expected(&bytes),
            EcosystemLimits::default()
        )
        .expect_err("foreign campaign")
        .code(),
        ModelCauseCode::CampaignMismatch
    );

    for field in [
        "counts",
        "adjacency",
        "topological_order",
        "nodes",
        "manifest_identity",
    ] {
        let mut replay: Value = serde_json::from_slice(model.bytes()).expect("model JSON");
        mutate_model_field(&mut replay, field);
        recompute_model_identity(&mut replay);
        let replay_bytes = canonical(&replay);
        assert_eq!(
            ecosystem_model::read(&replay_bytes, &checked, EcosystemLimits::default())
                .expect_err("replayed model field")
                .code(),
            ModelCauseCode::GraphMismatch,
            "field {field}"
        );
    }
    let mut trailing_model = model.bytes().to_vec();
    trailing_model.push(b'\n');
    assert_eq!(
        ecosystem_model::read(&trailing_model, &checked, EcosystemLimits::default())
            .expect_err("trailing model data")
            .code(),
        ModelCauseCode::InvalidDocument
    );
}

#[trace("TC-040", "FR-027-AC-4")]
#[test]
fn tc_040_every_manifest_and_model_resource_dimension_has_an_exact_cliff() {
    let value = manifest_value();
    let bytes = canonical(&value);
    let counts = kind_counts(&value);
    let exact = EcosystemLimits {
        manifest_bytes: u64::try_from(bytes.len()).expect("manifest length"),
        json_depth: json_depth(&value),
        string_bytes: longest_string(&value),
        repositories: counts[0],
        components: counts[1],
        objects: counts[2],
        interfaces: counts[3],
        contracts: counts[4],
        requirements: counts[5],
        tests: counts[6],
        reviews: counts[7],
        gaps: u64::try_from(value["gaps"].as_array().expect("gaps").len()).expect("gaps"),
        edges: u64::try_from(value["edges"].as_array().expect("edges").len()).expect("edges"),
        visited_work: u64::try_from(bytes.len()).expect("manifest work"),
        allocation_bytes: u64::try_from(bytes.len()).expect("manifest allocation"),
        ..EcosystemLimits::default()
    };
    manifest::read(&bytes, &expected(&bytes), exact).expect("every exact manifest limit");

    let cliffs = vec![
        EcosystemLimits {
            manifest_bytes: exact.manifest_bytes - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            json_depth: exact.json_depth - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            string_bytes: exact.string_bytes - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            repositories: exact.repositories - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            components: exact.components - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            objects: exact.objects - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            interfaces: exact.interfaces - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            contracts: exact.contracts - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            requirements: exact.requirements - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            tests: exact.tests - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            reviews: exact.reviews - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            gaps: exact.gaps - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            edges: exact.edges - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            visited_work: exact.visited_work - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            allocation_bytes: exact.allocation_bytes - 1,
            ..EcosystemLimits::default()
        },
    ];
    for limits in cliffs {
        assert_eq!(
            manifest::read(&bytes, &expected(&bytes), limits)
                .expect_err("one-under ceiling")
                .code(),
            ModelCauseCode::ResourceExhausted
        );
    }

    let checked =
        manifest::read(&bytes, &expected(&bytes), EcosystemLimits::default()).expect("manifest");
    let baseline = ecosystem_model::export(&checked, EcosystemLimits::default()).expect("model");
    let model_value: Value = serde_json::from_slice(baseline.bytes()).expect("model JSON");
    let model_counts = baseline.counts();
    let exact_model_populations = EcosystemLimits {
        repositories: model_counts.repositories,
        components: model_counts.components,
        objects: model_counts.objects,
        interfaces: model_counts.interfaces,
        contracts: model_counts.contracts,
        requirements: model_counts.requirements,
        tests: model_counts.tests,
        reviews: model_counts.reviews,
        gaps: model_counts.gaps,
        edges: model_counts.edges,
        json_depth: json_depth(&model_value),
        string_bytes: longest_string(&model_value),
        ..EcosystemLimits::default()
    };
    ecosystem_model::export(&checked, exact_model_populations)
        .expect("every exact model population, depth, and string limit");

    let mut model_cliffs = vec![
        EcosystemLimits {
            json_depth: exact_model_populations.json_depth - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            string_bytes: exact_model_populations.string_bytes - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            repositories: model_counts.repositories - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            components: model_counts.components - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            objects: model_counts.objects - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            interfaces: model_counts.interfaces - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            contracts: model_counts.contracts - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            requirements: model_counts.requirements - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            tests: model_counts.tests - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            reviews: model_counts.reviews - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            gaps: model_counts.gaps - 1,
            ..EcosystemLimits::default()
        },
        EcosystemLimits {
            edges: model_counts.edges - 1,
            ..EcosystemLimits::default()
        },
    ];
    for setter in [
        set_model_bytes as fn(&mut EcosystemLimits, u64),
        set_visited_work,
        set_allocation_bytes,
    ] {
        let exact = exact_export_limit(&checked, setter);
        let mut exact_limits = EcosystemLimits::default();
        setter(&mut exact_limits, exact);
        ecosystem_model::export(&checked, exact_limits).expect("exact model byte/work limit");
        setter(&mut exact_limits, exact - 1);
        model_cliffs.push(exact_limits);
    }
    for limits in model_cliffs {
        assert_eq!(
            ecosystem_model::export(&checked, limits)
                .expect_err("one-under model ceiling")
                .code(),
            ModelCauseCode::ResourceExhausted
        );
    }
}

fn mutate(base: &Value, action: impl FnOnce(&mut Value)) -> Value {
    let mut value = base.clone();
    action(&mut value);
    value
}

fn mutate_model_field(model: &mut Value, field: &str) {
    match field {
        "counts" => model["counts"]["edges"] = json!(0),
        "adjacency" => {
            model["adjacency"].as_array_mut().expect("adjacency").pop();
        }
        "topological_order" => model["topological_order"]
            .as_array_mut()
            .expect("topology")
            .reverse(),
        "nodes" => {
            model["nodes"].as_array_mut().expect("nodes").pop();
        }
        "manifest_identity" => model["manifest_identity"] = json!("00".repeat(32)),
        _ => panic!("unknown model mutation"),
    };
}

fn recompute_model_identity(model: &mut Value) {
    let mut body = model.clone();
    body.as_object_mut()
        .expect("model object")
        .remove("identity");
    let body_bytes = canonical(&body);
    let mut preimage = MODEL_PROFILE.as_bytes().to_vec();
    preimage.push(0);
    preimage.extend_from_slice(&body_bytes);
    model["identity"] = json!(BridgeDigest::raw(&preimage).to_string());
}

fn kind_counts(value: &Value) -> [u64; 8] {
    let mut counts = [0; 8];
    for node in value["nodes"].as_array().expect("nodes") {
        let index = match node["kind"].as_str().expect("kind") {
            "repository" => 0,
            "component" => 1,
            "object" => 2,
            "interface" => 3,
            "contract" => 4,
            "requirement" => 5,
            "test" => 6,
            "review" => 7,
            other => panic!("unknown node kind {other}"),
        };
        counts[index] += 1;
    }
    counts
}

fn json_depth(value: &Value) -> u64 {
    match value {
        Value::Array(values) => 1 + values.iter().map(json_depth).max().unwrap_or(0),
        Value::Object(values) => 1 + values.values().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn longest_string(value: &Value) -> u64 {
    match value {
        Value::String(text) => u64::try_from(text.len()).expect("string length"),
        Value::Array(values) => values.iter().map(longest_string).max().unwrap_or(0),
        Value::Object(values) => values
            .iter()
            .map(|(key, value)| {
                u64::try_from(key.len())
                    .expect("key length")
                    .max(longest_string(value))
            })
            .max()
            .unwrap_or(0),
        _ => 0,
    }
}

fn exact_export_limit(
    checked: &manifest::CheckedManifestSet,
    setter: fn(&mut EcosystemLimits, u64),
) -> u64 {
    let mut selected = 64 * 1024 * 1024;
    for _ in 0..16 {
        let mut limits = EcosystemLimits::default();
        setter(&mut limits, selected);
        match ecosystem_model::export(checked, limits) {
            Ok(model) => {
                let actual = u64::try_from(model.bytes().len()).expect("model byte length");
                if actual == selected {
                    return selected;
                }
                selected = actual;
            }
            Err(error) if error.code() == ModelCauseCode::ResourceExhausted => {
                selected = selected.saturating_add(64);
            }
            Err(error) => panic!("unexpected exact-limit refusal: {error}"),
        }
    }
    panic!("model byte limit did not reach a fixed point")
}

fn set_model_bytes(limits: &mut EcosystemLimits, value: u64) {
    limits.model_bytes = value;
}

fn set_visited_work(limits: &mut EcosystemLimits, value: u64) {
    limits.visited_work = value;
}

fn set_allocation_bytes(limits: &mut EcosystemLimits, value: u64) {
    limits.allocation_bytes = value;
}
