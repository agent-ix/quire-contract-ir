// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-447's explicit producer lane. This target is outside ordinary IR tests:
//! it compiles authored QSL source using the owning checkout's current binary,
//! strictly admits those bytes, and queries the resulting ordinary FR-330
//! package. No generated QSL artifact is copied into this repository.

use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedNodeId, CheckedNodePathSegment, CheckedNodeTag,
    CheckedOccurrenceRole, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, ContractDependencies,
    ContractDispositionV1, ContractOccurrenceKeyV1, ContractPackageReadResult, ContractPackageV1,
    ContractPackageWireV1, ContractResultContextV1, ContractResultViewV1, ContractRootViewV1,
    ContractSourceKeyV1, LoweringProfileRef,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

const SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    function f using v(a: Int[0, 9]): Int[0, 10] pure { a + 1 }\n";

const SEPARATED_SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    function f0 using v(a: Int[0, 9]): Int[0, 10] pure { a + 1 }\n\
    function f1 using v(a: Int[0, 9]): Int[0, 11] pure { a + 2 }\n\
    function f2 using v(a: Int[0, 9]): Int[0, 12] pure { a + 3 }\n";

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.join("result-context.native"));
        let _ = std::fs::remove_file(self.0.join("compile.json"));
        let _ = std::fs::remove_dir(&self.0);
    }
}

fn request(directory: &std::path::Path, source: &str) -> PathBuf {
    std::fs::write(directory.join("result-context.native"), source)
        .expect("write authored QSL source");
    let request = json!({"format": "native-compile/1", "request": {
        "models": [],
        "program": {"source": {
            "file": "result-context.native", "authority": "agent-ix",
            "identity": "test:ir769-result-context", "revision_namespace": "fixture",
            "revision": "fixture:1",
            "document": "ResultContext", "formal_revision": 1,
        }}
    }});
    let path = directory.join("compile.json");
    std::fs::write(&path, serde_json::to_vec(&request).expect("request JSON"))
        .expect("write own request");
    path
}

fn compile_qsl(source: &str, label: &str) -> CheckedPackageV2 {
    let qsl = std::env::var_os("QUIRE_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("real-QSL conformance requires QUIRE_SPEC_LANGUAGE_DIR");
    let producer = std::env::var_os("QUIRE_SPEC_LANGUAGE_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| qsl.join("target/debug/quire-spec"));
    assert!(
        producer.is_file(),
        "build QSL's current quire-spec binary first"
    );
    let scratch = Scratch(
        std::env::temp_dir().join(format!("ir769-qsl-producer-{}-{label}", std::process::id())),
    );
    std::fs::create_dir(&scratch.0).expect("create own producer scratch");
    let request_file = request(&scratch.0, source);
    let output = Command::new(&producer)
        .arg("compile")
        .arg(&request_file)
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", producer.display()));
    assert!(
        output.status.success(),
        "QSL refused authored source: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    match CheckedPackageV2::read(
        &output.stdout,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("fresh QSL emission must admit: {other:?}"),
    }
}

fn exact_operation<'a>(package: &'a CheckedPackageV2, identity: &str) -> &'a CheckedNodeId {
    let matches = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            serde_json::to_value(&node.body)
                .expect("typed QSL body")
                .pointer("/operation/identity")
                .and_then(Value::as_str)
                == Some(identity)
        })
        .collect::<Vec<_>>();
    let [node] = matches.as_slice() else {
        panic!(
            "QSL must emit exactly one {identity} expression, got {}",
            matches.len()
        );
    };
    &node.node_id
}

fn occurrence(package: &CheckedPackageV2, key: &CheckedNodeId) -> ContractOccurrenceKeyV1 {
    let entries = package
        .source_map()
        .iter()
        .filter(|entry| &entry.node_id == key && entry.role == CheckedOccurrenceRole::Expression)
        .collect::<Vec<_>>();
    let [entry] = entries.as_slice() else {
        panic!("QSL must retain one expression occurrence for the selected key");
    };
    ContractOccurrenceKeyV1 {
        node_id: key.clone(),
        role: entry.role.clone(),
        ordinal: entry.ordinal,
    }
}

fn admitted_occurrence(
    source: &CheckedPackageV2,
    package: &ContractPackageV1,
    key: &ContractSourceKeyV1,
    requested: &ContractOccurrenceKeyV1,
) -> ContractOccurrenceKeyV1 {
    assert_eq!(
        ContractSourceKeyV1::from_v2_request(source, &requested.node_id).as_ref(),
        Some(key),
        "QSL occurrence and requested expression must have the same structural identity"
    );
    let ContractRootViewV1::Lowered(reached) = package
        .reached_closure(key)
        .expect("admitted source-key view")
    else {
        panic!("requested occurrence must be lowered");
    };
    assert_eq!(
        &reached.root_structural_key, key,
        "admitted output root must be the requested source identity"
    );
    let sources = package
        .source_occurrences(&reached.root.node_id)
        .expect("represented source map");
    let matches = sources
        .iter()
        .filter(|entry| entry.role == requested.role && entry.ordinal == requested.ordinal)
        .collect::<Vec<_>>();
    let [entry] = matches.as_slice() else {
        panic!("one admitted V1 source occurrence matches the QSL role and ordinal");
    };
    ContractOccurrenceKeyV1 {
        node_id: entry.node_id.clone(),
        role: entry.role.clone(),
        ordinal: entry.ordinal,
    }
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "result-context".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

fn package(source: &CheckedPackageV2, roots: &[CheckedNodeId]) -> ContractPackageV1 {
    package_with_profile(source, roots, &profile())
}

fn package_with_profile(
    source: &CheckedPackageV2,
    roots: &[CheckedNodeId],
    profile: &CompleteLoweringProfileV2,
) -> ContractPackageV1 {
    let lower = source.lower(roots, profile);
    let bytes = lower
        .package
        .canonical_bytes()
        .expect("lowered FR-330 bytes");
    match ContractPackageWireV1::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("FR-330 package must strictly admit: {other:?}"),
    }
}

fn endpoints(node: &quire_contract_model::CheckedSemanticNodeV2<Value>) -> (&str, &str) {
    assert_eq!(node.node_tag.as_ref(), "bounded_domain");
    assert_eq!(node.semantic_form.as_ref(), "integer_range");
    let members = node.body["members"].as_array().expect("range members");
    let [min, max] = members.as_slice() else {
        panic!("integer range must have exactly min and max");
    };
    assert_eq!(min["name"], "min");
    assert_eq!(max["name"], "max");
    (
        min["value"]["value"].as_str().expect("minimum"),
        max["value"]["value"].as_str().expect("maximum"),
    )
}

fn qsl_pair(source: &CheckedPackageV2, upper: &str) -> (CheckedNodeId, CheckedNodeId) {
    let narrows = source
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            let body = serde_json::to_value(&node.body).expect("typed QSL body");
            if body.pointer("/operation/identity").and_then(Value::as_str)
                != Some("quire.op.numeric.narrow")
            {
                return false;
            }
            source.graph().nodes.iter().any(|bound| {
                bound.node_id == node.semantic_type
                    && bound.node_tag.as_ref() == "bounded_domain"
                    && serde_json::to_value(&bound.body)
                        .ok()
                        .and_then(|body| {
                            body.pointer("/members/1/value/value")
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                        })
                        .as_deref()
                        == Some(upper)
            })
        })
        .collect::<Vec<_>>();
    let [narrow] = narrows.as_slice() else {
        panic!("exactly one QSL narrow has upper bound {upper}");
    };
    let target = serde_json::to_value(&narrow.body)
        .expect("typed narrow")
        .pointer("/arguments/0/target")
        .cloned()
        .expect("direct narrow target");
    let adds = source
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            serde_json::to_value(&node.node_id).expect("typed id") == target
                && serde_json::to_value(&node.body)
                    .expect("typed QSL body")
                    .pointer("/operation/identity")
                    .and_then(Value::as_str)
                    == Some("quire.op.integer.add")
        })
        .collect::<Vec<_>>();
    let [add] = adds.as_slice() else {
        panic!("one add directly feeds narrow with upper bound {upper}");
    };
    (add.node_id.clone(), narrow.node_id.clone())
}

fn raw_indexes(key: &CheckedNodeId) -> Vec<u64> {
    let indexes = key
        .path
        .iter()
        .filter_map(|segment| match segment {
            CheckedNodePathSegment::Index(index) => Some(*index),
            CheckedNodePathSegment::Name(_) | CheckedNodePathSegment::Node(_) => None,
        })
        .collect::<Vec<_>>();
    indexes
}

fn structural_body(value: &Value, output: &BTreeMap<CheckedNodeId, CheckedNodeId>) -> Value {
    if let Ok(raw) = serde_json::from_value::<CheckedNodeId>(value.clone()) {
        if let Some(structural) = output.get(&raw) {
            return serde_json::to_value(structural).expect("structural node identity");
        }
    }
    match value {
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| structural_body(item, output))
                .collect(),
        ),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(name, item)| (name.clone(), structural_body(item, output)))
                .collect(),
        ),
        other => other.clone(),
    }
}

type StructuralOccurrence = (
    CheckedNodeId,
    CheckedOccurrenceRole,
    u64,
    Vec<quire_contract_model::CheckedSourceRegion>,
);
type StructuralMember = (CheckedNodeId, Value, Vec<StructuralOccurrence>);

fn structural_members(
    package: &ContractPackageV1,
    reached: &quire_contract_model::ContractReachedClosureV1<'_>,
) -> Vec<StructuralMember> {
    let output = reached
        .nodes
        .iter()
        .map(|entry| (entry.node.node_id.clone(), entry.structural_key.clone()))
        .collect::<BTreeMap<_, _>>();
    let source = package
        .wire()
        .source_keys
        .iter()
        .enumerate()
        .map(|(index, raw)| {
            (
                raw.clone(),
                package
                    .source_key_at(index)
                    .expect("admitted source-frame key")
                    .structural_key()
                    .clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    reached
        .nodes
        .iter()
        .map(|entry| {
            (
                entry.structural_key.clone(),
                structural_body(&entry.node.body, &output),
                entry
                    .sources
                    .iter()
                    .map(|occurrence| {
                        (
                            source
                                .get(&occurrence.node_id)
                                .expect("admitted occurrence source identity")
                                .clone(),
                            occurrence.role.clone(),
                            occurrence.ordinal,
                            occurrence.regions.clone(),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

#[trace("TC-447", "FR-049-AC-1", "FR-049-AC-5", "FR-049-AC-6")]
#[test]
fn tc_447_fresh_qsl_source_admits_exact_application_and_narrow_context() {
    let checked = compile_qsl(SOURCE, "single");
    let add_key = exact_operation(&checked, "quire.op.integer.add");
    let add_request = occurrence(&checked, add_key);
    let add_key = ContractSourceKeyV1::from_v2_request(&checked, add_key)
        .expect("QSL add request has a source structural key");
    let narrows = checked
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            let body = serde_json::to_value(&node.body).expect("typed QSL body");
            body.pointer("/operation/identity").and_then(Value::as_str)
                == Some("quire.op.numeric.narrow")
                && body.pointer("/arguments/0/target")
                    == Some(&serde_json::to_value(&add_request.node_id).expect("typed key"))
        })
        .collect::<Vec<_>>();
    let [narrow_node] = narrows.as_slice() else {
        panic!("QSL checked result must have exactly one direct narrow");
    };
    let narrow_request = occurrence(&checked, &narrow_node.node_id);
    let narrow_key = ContractSourceKeyV1::from_v2_request(&checked, &narrow_request.node_id)
        .expect("QSL narrow request has a source structural key");
    let roots = [add_request.node_id.clone(), narrow_request.node_id.clone()];
    let contextual = package(&checked, &roots);
    let plain = package(&checked, std::slice::from_ref(&add_request.node_id));
    let add = admitted_occurrence(&checked, &contextual, &add_key, &add_request);
    let narrow = admitted_occurrence(&checked, &contextual, &narrow_key, &narrow_request);
    assert_eq!(contextual.wire().feature_accounting.len(), 2);
    assert!(contextual.wire().semantic_graph.nodes.len() > plain.wire().semantic_graph.nodes.len());
    assert_eq!(plain.wire().feature_accounting.len(), 1);
    let canonical_v1 = |package: &ContractPackageV1| {
        quire_canonical::to_vec(package.wire(), quire_canonical::Limits::new(u64::MAX))
            .expect("admitted V1 package encodes canonically")
    };
    assert_ne!(canonical_v1(&plain), canonical_v1(&contextual));
    let ContractResultViewV1::Lowered(plain_view) = plain
        .result_context(
            &add_key,
            &add,
            Some(&narrow),
            BoundedDomainForm::IntegerRange,
        )
        .expect("plain admitted query")
    else {
        panic!("QSL add lowers alone");
    };
    assert_eq!(
        plain_view.context,
        ContractResultContextV1::IncompleteContext
    );
    assert!(!plain.wire().semantic_graph.nodes.iter().any(|node| {
        node.body
            .pointer("/operation/identity")
            .and_then(Value::as_str)
            == Some("quire.op.numeric.narrow")
            || (node.node_tag.as_ref() == "bounded_domain" && endpoints(node) == ("0", "10"))
    }));
    let before = contextual.wire().clone();
    let ContractResultViewV1::Lowered(view) = contextual
        .result_context(
            &add_key,
            &add,
            Some(&narrow),
            BoundedDomainForm::IntegerRange,
        )
        .expect("contextual admitted query")
    else {
        panic!("QSL add lowers with its narrow");
    };
    let ContractResultContextV1::Narrowed {
        conversion,
        conversion_sources,
        bound,
    } = view.context
    else {
        panic!("QSL's direct narrow must name its result bound");
    };
    let ContractRootViewV1::Lowered(narrow_root) = contextual
        .reached_closure(&narrow_key)
        .expect("selected narrow source key")
    else {
        panic!("selected narrow is lowered");
    };
    assert_eq!(conversion.node_id, narrow_root.root.node_id);
    assert!(conversion_sources.iter().any(|entry| {
        entry.node_id == narrow.node_id
            && entry.role == narrow.role
            && entry.ordinal == narrow.ordinal
    }));
    assert_eq!(endpoints(bound.node), ("0", "10"));
    assert!(
        structural_members(&plain, &plain_view.reached)
            == structural_members(&contextual, &view.reached),
        "the contextual narrow must not rewrite the add's structural closure, bodies or sources"
    );
    assert!(view
        .reached
        .nodes
        .iter()
        .any(|entry| entry.node.node_tag.as_ref() == "bounded_domain"
            && endpoints(entry.node) == ("0", "9")));
    assert!(!view
        .reached
        .nodes
        .iter()
        .any(|entry| entry.node.node_id == narrow_root.root.node_id));
    assert_eq!(contextual.wire(), &before);
}

#[trace("TC-447", "FR-049-AC-7")]
#[test]
fn tc_447_separated_qsl_siblings_keep_source_and_output_frames_distinct() {
    let checked = compile_qsl(SEPARATED_SOURCE, "separated");
    let (f0_add, f0_narrow) = qsl_pair(&checked, "10");
    let (f1_add, _) = qsl_pair(&checked, "11");
    let (f2_add, f2_narrow) = qsl_pair(&checked, "12");
    let excluded = checked
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "function"
                && node
                    .declaration
                    .as_ref()
                    .and_then(|declaration| declaration.qualified_name.last())
                    .is_some_and(|name| name.as_ref() == "f1")
        })
        .expect("QSL f1 function root")
        .node_id
        .clone();
    let mut selected_profile = profile();
    selected_profile
        .supported_tags
        .retain(|tag| *tag != CheckedNodeTag::Function);
    let roots = [
        f0_add.clone(),
        f0_narrow.clone(),
        excluded.clone(),
        f2_add.clone(),
        f2_narrow.clone(),
    ];
    let admitted = package_with_profile(&checked, &roots, &selected_profile);
    assert_eq!(admitted.wire().feature_accounting.len(), roots.len());
    assert_eq!(
        admitted.wire().feature_accounting[2].disposition(),
        ContractDispositionV1::Unsupported
    );
    let excluded_key = ContractSourceKeyV1::from_v2_request(&checked, &excluded)
        .expect("excluded function has source identity");
    assert!(matches!(
        admitted
            .reached_closure(&excluded_key)
            .expect("excluded view"),
        ContractRootViewV1::NonLowered(_)
    ));
    assert!(!admitted
        .wire()
        .semantic_graph
        .nodes
        .iter()
        .any(|node| node.node_tag.as_ref() == "function"));

    let pairs = [(&f0_add, &f0_narrow, "10"), (&f2_add, &f2_narrow, "12")];
    let mut selected = Vec::new();
    for (add, narrow, upper) in pairs {
        let add_key =
            ContractSourceKeyV1::from_v2_request(&checked, add).expect("V2 add source identity");
        let narrow_key = ContractSourceKeyV1::from_v2_request(&checked, narrow)
            .expect("V2 narrow source identity");
        let add_request = occurrence(&checked, add);
        let narrow_request = occurrence(&checked, narrow);
        let application = admitted_occurrence(&checked, &admitted, &add_key, &add_request);
        let conversion = admitted_occurrence(&checked, &admitted, &narrow_key, &narrow_request);
        let ContractResultViewV1::Lowered(view) = admitted
            .result_context(
                &add_key,
                &application,
                Some(&conversion),
                BoundedDomainForm::IntegerRange,
            )
            .expect("admitted result context")
        else {
            panic!("QSL supported add must be lowered");
        };
        let admitted_sources = admitted
            .source_occurrences(&view.reached.root.node_id)
            .expect("output root source correspondence");
        let root_in_closure = view
            .reached
            .nodes
            .iter()
            .find(|entry| entry.node.node_id == view.reached.root.node_id)
            .expect("root in reached closure");
        assert_eq!(root_in_closure.sources, admitted_sources);
        assert!(admitted_sources.iter().any(|entry| {
            entry.node_id == application.node_id
                && entry.role == application.role
                && entry.ordinal == application.ordinal
        }));
        let ContractResultContextV1::Narrowed {
            conversion: converted,
            conversion_sources,
            bound,
        } = view.context
        else {
            panic!("QSL direct narrow must select its own bound");
        };
        assert_eq!(endpoints(bound.node), ("0", upper));
        assert!(conversion_sources.iter().any(|entry| {
            entry.node_id == conversion.node_id
                && entry.role == conversion.role
                && entry.ordinal == conversion.ordinal
        }));
        let ContractRootViewV1::Lowered(narrow_root) = admitted
            .reached_closure(&narrow_key)
            .expect("typed narrow view")
        else {
            panic!("QSL narrow must be lowered");
        };
        assert_eq!(converted.node_id, narrow_root.root.node_id);
        selected.push((
            add_key,
            application,
            conversion,
            view.reached.root.node_id.clone(),
        ));
    }
    assert_ne!(selected[0].3, selected[1].3);
    assert_ne!(f0_add, f1_add);
    let witness = selected
        .iter()
        .zip([&f0_add, &f2_add])
        .find(|((_, application, _, output), request)| {
            let indices = [
                raw_indexes(request),
                raw_indexes(&application.node_id),
                raw_indexes(output),
            ];
            indices.iter().all(|indexes| !indexes.is_empty())
                && indices[0] != indices[1]
                && indices[0] != indices[2]
                && indices[1] != indices[2]
        })
        .expect("one sibling must cross three distinct graph-local indexes");

    // A different admitted source identity, or a wrong occurrence coordinate,
    // cannot be accepted as the first sibling's application.
    let (key, application, conversion, output) = witness.0;
    let other_application = if *key == selected[0].0 {
        &selected[1].1
    } else {
        &selected[0].1
    };
    let raw_collision = admitted
        .wire()
        .source_map
        .nodes
        .iter()
        .filter(|mapping| mapping.node_id != *output)
        .flat_map(|mapping| &mapping.occurrences)
        .find(|entry| {
            entry.role == CheckedOccurrenceRole::Expression
                && raw_indexes(output)
                    .iter()
                    .any(|index| raw_indexes(&entry.node_id).contains(index))
        })
        .map(|entry| ContractOccurrenceKeyV1 {
            node_id: entry.node_id.clone(),
            role: entry.role.clone(),
            ordinal: entry.ordinal,
        })
        .expect("another source identity must collide with the output's raw graph index");
    assert_ne!(raw_collision.node_id, application.node_id);
    for bad in [
        (*other_application).clone(),
        ContractOccurrenceKeyV1 {
            role: CheckedOccurrenceRole::Type,
            ..application.clone()
        },
        ContractOccurrenceKeyV1 {
            ordinal: application.ordinal + 1,
            ..application.clone()
        },
    ] {
        let ContractResultViewV1::Lowered(view) = admitted
            .result_context(key, &bad, Some(conversion), BoundedDomainForm::IntegerRange)
            .expect("adverse admitted query")
        else {
            panic!("first sibling remains lowered");
        };
        assert_eq!(view.context, ContractResultContextV1::IncompleteContext);
    }
    // An admitted occurrence on a different node with a colliding raw index
    // cannot authenticate the represented output root's source identity.
    let ContractResultViewV1::Lowered(view) = admitted
        .result_context(
            key,
            &raw_collision,
            Some(conversion),
            BoundedDomainForm::IntegerRange,
        )
        .expect("raw collision query")
    else {
        panic!("witness sibling remains lowered");
    };
    assert_eq!(view.context, ContractResultContextV1::IncompleteContext);

    let baseline_roots = [
        f0_add.clone(),
        f0_narrow.clone(),
        f2_add.clone(),
        f2_narrow.clone(),
    ];
    let baseline = package_with_profile(&checked, &baseline_roots, &selected_profile);
    assert_eq!(baseline.wire().feature_accounting.len(), 4);
    let reordered_roots = [
        f1_add.clone(),
        f2_add.clone(),
        f2_narrow.clone(),
        f0_add.clone(),
        f0_narrow.clone(),
        excluded,
    ];
    let reordered = package_with_profile(&checked, &reordered_roots, &selected_profile);
    assert_eq!(
        reordered.wire().feature_accounting[5].disposition(),
        ContractDispositionV1::Unsupported
    );
    assert_eq!(
        reordered.wire().feature_accounting[0].disposition(),
        ContractDispositionV1::Lowered
    );
    assert_ne!(
        admitted.wire().semantic_graph.nodes,
        reordered.wire().semantic_graph.nodes,
        "the independent f1 add must perturb the output graph"
    );
    assert!(
        [&f0_add, &f2_add].iter().any(|add| {
            let key =
                ContractSourceKeyV1::from_v2_request(&checked, add).expect("supported V2 add key");
            let ContractRootViewV1::Lowered(before) = admitted
                .reached_closure(&key)
                .expect("original admitted view")
            else {
                panic!("original add is lowered");
            };
            let ContractRootViewV1::Lowered(after) = reordered
                .reached_closure(&key)
                .expect("reordered admitted view")
            else {
                panic!("reordered add is lowered");
            };
            before.root.node_id != after.root.node_id
                && before.root_structural_key == after.root_structural_key
        }),
        "unrelated output changes must move a supported raw output id without changing identity"
    );
    for candidate in [&baseline, &reordered] {
        for (add, narrow, upper) in [(&f0_add, &f0_narrow, "10"), (&f2_add, &f2_narrow, "12")] {
            let key = ContractSourceKeyV1::from_v2_request(&checked, add).expect("V2 add key");
            let narrow_key =
                ContractSourceKeyV1::from_v2_request(&checked, narrow).expect("V2 narrow key");
            let application =
                admitted_occurrence(&checked, candidate, &key, &occurrence(&checked, add));
            let conversion = admitted_occurrence(
                &checked,
                candidate,
                &narrow_key,
                &occurrence(&checked, narrow),
            );
            let ContractResultViewV1::Lowered(view) = candidate
                .result_context(
                    &key,
                    &application,
                    Some(&conversion),
                    BoundedDomainForm::IntegerRange,
                )
                .expect("perturbed admitted query")
            else {
                panic!("supported add remains lowered");
            };
            let ContractResultContextV1::Narrowed { bound, .. } = view.context else {
                panic!("direct narrow remains selected");
            };
            assert_eq!(endpoints(bound.node), ("0", upper));
        }
    }
}
