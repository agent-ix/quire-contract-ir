// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-447's explicit producer lane. This target is outside ordinary IR tests:
//! it compiles authored QSL source using the owning checkout's current binary,
//! strictly admits those bytes, and queries the resulting ordinary FR-330
//! package. No generated QSL artifact is copied into this repository.

use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeTag, CheckedOccurrenceRole,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CompleteLoweringProfileV2, ContractDependencies, ContractOccurrenceKeyV1,
    ContractPackageReadResult, ContractPackageV1, ContractPackageWireV1, ContractResultContextV1,
    ContractResultViewV1, LoweringProfileRef,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;

const SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    function f using v(a: Int[0, 9]): Int[0, 10] pure { a + 1 }\n";

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove own producer scratch");
    }
}

fn request(directory: &std::path::Path) -> PathBuf {
    std::fs::write(directory.join("result-context.native"), SOURCE)
        .expect("write authored QSL source");
    let request = json!({"format": "native-compile/1", "request": {
        "models": [],
        "program": {"source": {
            "file": "result-context.native", "authority": "agent-ix",
            "identity": "test:ir769-result-context", "revision_namespace": "fixture",
            "revision": "fixture:1",
            "digest": format!("sha256:{:x}", Sha256::digest(SOURCE.as_bytes())),
            "document": "ResultContext", "formal_revision": 1,
        }}
    }});
    let path = directory.join("compile.json");
    std::fs::write(&path, serde_json::to_vec(&request).expect("request JSON"))
        .expect("write own request");
    path
}

fn exact_operation<'a>(package: &'a CheckedPackageV2, identity: &str) -> &'a CheckedNodeId {
    let matches = package
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.body
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
    let lower = source.lower(roots, &profile());
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

fn endpoints(node: &quire_contract_model::CheckedSemanticNodeV2) -> (&str, &str) {
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

#[trace("TC-447", "FR-049-AC-1", "FR-049-AC-5", "FR-049-AC-6")]
#[test]
fn tc_447_fresh_qsl_source_admits_exact_application_and_narrow_context() {
    let qsl = std::env::var_os("QUIRE_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("TC-447 requires QUIRE_SPEC_LANGUAGE_DIR");
    let producer = std::env::var_os("QUIRE_SPEC_LANGUAGE_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| qsl.join("target/debug/quire-spec"));
    assert!(
        producer.is_file(),
        "build QSL's current quire-spec binary first"
    );
    let scratch =
        Scratch(std::env::temp_dir().join(format!("ir769-qsl-producer-{}", std::process::id())));
    std::fs::create_dir(&scratch.0).expect("create own producer scratch");
    let request_file = request(&scratch.0);
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
    let checked = match CheckedPackageV2::read(
        &output.stdout,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("fresh QSL emission must admit: {other:?}"),
    };
    let add_key = exact_operation(&checked, "quire.op.integer.add");
    let add = occurrence(&checked, add_key);
    let narrows = checked
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.body
                .pointer("/operation/identity")
                .and_then(Value::as_str)
                == Some("quire.op.numeric.narrow")
                && node.body.pointer("/arguments/0/target")
                    == Some(&serde_json::to_value(add_key).expect("typed key"))
        })
        .collect::<Vec<_>>();
    let [narrow_node] = narrows.as_slice() else {
        panic!("QSL checked result must have exactly one direct narrow");
    };
    let narrow = occurrence(&checked, &narrow_node.node_id);
    let roots = [add.node_id.clone(), narrow.node_id.clone()];
    let contextual = package(&checked, &roots);
    let plain = package(&checked, std::slice::from_ref(&add.node_id));
    assert_eq!(contextual.wire().feature_accounting.len(), 2);
    assert_eq!(contextual.wire().semantic_graph.nodes.len(), 8);
    assert_eq!(plain.wire().feature_accounting.len(), 1);
    assert_eq!(plain.wire().semantic_graph.nodes.len(), 6);
    let canonical_v1 = |package: &ContractPackageV1| {
        quire_canonical::to_vec(package.wire(), quire_canonical::Limits::new(u64::MAX))
            .expect("admitted V1 package encodes canonically")
    };
    assert_ne!(canonical_v1(&plain), canonical_v1(&contextual));
    let ContractResultViewV1::Lowered(plain_view) = plain
        .result_context(
            &add.node_id,
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
    let before = contextual.wire().clone();
    let ContractResultViewV1::Lowered(view) = contextual
        .result_context(
            &add.node_id,
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
    assert_eq!(conversion.node_id, narrow.node_id);
    assert!(conversion_sources.iter().any(|entry| {
        entry.node_id == narrow.node_id
            && entry.role == narrow.role
            && entry.ordinal == narrow.ordinal
    }));
    assert_eq!(endpoints(bound.node), ("0", "10"));
    assert_eq!(view.reached.nodes.len(), 6);
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
        .any(|entry| entry.node.node_id == narrow.node_id));
    assert_eq!(contextual.wire(), &before);
}
