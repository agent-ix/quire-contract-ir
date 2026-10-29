// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-450's authoritative QSL producer route: compile FR-062 RR-13 and query
//! the current QSL `RequirementItem::path_condition()` API before mapping into IR.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2ReadResult, CompleteLoweringProfileV2, ContractDependencies,
    ContractGuardContextViewV1, ContractOccurrenceKeyV1, ContractPackageReadResult,
    ContractPackageV1, ContractPackageWireV1, ContractPathGuardV1, LoweringProfileRef,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SOURCE: &str = concat!(
    "language \"ix:native\" edition \"1-draft\";\n",
    "profile v = \"quire.value.complete/v1\";\n",
    "function rr13 using v(x: Int[0, 9], y: Int[-9, 9]): Rational[-9, 9; 1, 9] pure { if y != 0 then x / y else rational(0, 1) }\n",
    "function rr13_else using v(x: Int[0, 9], y: Int[-9, 9]): Rational[-9, 9; 1, 9] pure { if y != 0 then x / y else x / 1 }\n",
);

const AUTHORITY: &str = "agent-ix";
const IDENTITY: &str = "test:ir779-rr13";
const REVISION_NAMESPACE: &str = "fixture";
const REVISION: &str = "fixture:1";

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned QSL producer scratch");
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn qsl_items(qsl: &Path, scratch: &Path) -> Value {
    let helper_dir = scratch.join("qsl-helper");
    fs::create_dir_all(helper_dir.join("src")).expect("create helper project");
    let manifest = format!(
        "[package]\nname = \"ir779-qsl-items\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nqsl-cst = {{ path = \"{}\" }}\nqsl-forms = {{ path = \"{}\" }}\nqsl-foundation = {{ path = \"{}\" }}\nqsl-semantics = {{ path = \"{}\" }}\nqsl-route = {{ path = \"{}\" }}\nquire-semantic-value = {{ git = \"https://github.com/agent-ix/quire-semantic-value\", branch = \"main\" }}\nserde_json = \"1\"\n",
        qsl.join("qsl-cst").display(),
        qsl.join("qsl-forms").display(),
        qsl.join("qsl-foundation").display(),
        qsl.join("qsl-semantics").display(),
        qsl.join("qsl-route").display(),
    );
    fs::write(helper_dir.join("Cargo.toml"), manifest).expect("write local helper manifest");
    fs::copy(qsl.join("Cargo.lock"), helper_dir.join("Cargo.lock"))
        .expect("reuse the current QSL workspace dependency resolution");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/conformance_qsl_ir779/qsl_items.rs"),
        helper_dir.join("src/main.rs"),
    )
    .expect("copy tracked QSL API harness into owned scratch");
    let source_path = scratch.join("rr13.native");
    fs::write(&source_path, SOURCE).expect("write FR-062 RR-13 source");
    let target = env::var_os("QUIRE_SPEC_LANGUAGE_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| qsl.join("target/ir779-qsl-helper"));
    let output = Command::new(env!("CARGO"))
        .args(["run", "--offline", "--quiet", "--manifest-path"])
        .arg(helper_dir.join("Cargo.toml"))
        .args(["--", source_path.to_str().expect("UTF-8 scratch path")])
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("run QSL checked-item API helper");
    assert!(
        output.status.success(),
        "QSL item helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("QSL helper emits item JSON")
}

fn compile_request(scratch: &Path) -> PathBuf {
    let source_path = scratch.join("rr13.native");
    let request = json!({"format": "native-compile/1", "request": {
        "models": [],
        "program": {"source": {
            "file": "rr13.native",
            "authority": AUTHORITY,
            "identity": IDENTITY,
            "revision_namespace": REVISION_NAMESPACE,
            "revision": REVISION,
            "digest": format!("sha256:{}", hex(&Sha256::digest(SOURCE.as_bytes()))),
            "document": "IR779RR13",
            "formal_revision": 1,
        }}
    }});
    let path = scratch.join("compile.json");
    fs::write(&path, serde_json::to_vec(&request).expect("request JSON"))
        .expect("write QSL compile request");
    assert!(source_path.is_file());
    path
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "ir779-qsl-rr13".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: quire_contract_model::CheckedNodeTag::ALL
            .iter()
            .copied()
            .collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

fn lower(source: &CheckedPackageV2, roots: &[CheckedNodeId]) -> ContractPackageV1 {
    let result = source.lower(roots, &profile());
    match ContractPackageWireV1::read(
        result.package.canonical_bytes().expect("FR-330 bytes"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("one QSL-rooted FR-330 package admits: {other:?}"),
    }
}

fn item_occurrence(item: &Value, field: &str) -> ContractOccurrenceKeyV1 {
    let occurrence = &item[field];
    assert_eq!(occurrence["role"], "expression");
    ContractOccurrenceKeyV1 {
        node_id: CheckedNodeId {
            domain: "quire.checked-semantic-node/v1".into(),
            digest: occurrence["node"]
                .as_str()
                .expect("node digest")
                .to_owned()
                .into(),
        },
        role: CheckedOccurrenceRole::Expression,
        ordinal: occurrence["ordinal"].as_u64().expect("ordinal"),
    }
}

#[trace("TC-450", "FR-050-AC-1")]
#[test]
fn tc_450_qsl_rr13_items_map_their_checked_path_conditions_into_admitted_ir() {
    let qsl = env::var_os("QUIRE_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("TC-450 QSL producer target requires QUIRE_SPEC_LANGUAGE_DIR");
    let producer = env::var_os("QUIRE_SPEC_LANGUAGE_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| qsl.join("target/debug/quire-spec"));
    assert!(producer.is_file(), "build the current QSL compiler first");
    let scratch = Scratch(env::temp_dir().join(format!("ir779-qsl-{}", std::process::id())));
    fs::create_dir(&scratch.0).expect("create producer scratch");
    let qsl_items = qsl_items(&qsl, &scratch.0);
    let request = compile_request(&scratch.0);
    let emitted = Command::new(&producer)
        .arg("compile")
        .arg(request)
        .output()
        .expect("run QSL FR-322 producer");
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let checked = match CheckedPackageV2::read(
        &emitted.stdout,
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("fresh QSL FR-322 output admits: {other:?}"),
    };

    let guarded_items = qsl_items
        .as_array()
        .expect("QSL RequirementItems")
        .iter()
        .filter(|item| {
            if item["guards"].as_array().is_none_or(Vec::is_empty) {
                return false;
            }
            let application = item_occurrence(item, "application");
            checked
                .graph()
                .nodes
                .iter()
                .any(|node| node.node_id == application.node_id)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        guarded_items.len(),
        3,
        "RR-13 plus both checked branch items"
    );
    let mut holds_seen = Vec::new();
    let mut guard_polarities = Vec::new();
    for item in guarded_items {
        let application = item_occurrence(item, "application");
        let qsl_guards = item["guards"].as_array().expect("path condition");
        assert_eq!(qsl_guards.len(), 1);
        let guard_occurrence = item_occurrence(&qsl_guards[0], "occurrence");
        let holds = qsl_guards[0]["holds"].as_bool().expect("guard polarity");
        holds_seen.push(holds);
        guard_polarities.push((guard_occurrence.clone(), holds));
        let application_node = checked
            .graph()
            .nodes
            .iter()
            .find(|node| node.node_id == application.node_id)
            .expect("QSL item application is represented");
        assert_eq!(
            application_node
                .body
                .pointer("/operation/identity")
                .and_then(Value::as_str),
            Some("quire.op.rational.div")
        );
        let roots = [
            application.node_id.clone(),
            guard_occurrence.node_id.clone(),
        ];
        let package = lower(&checked, &roots);
        let guards = [ContractPathGuardV1 {
            occurrence: guard_occurrence.clone(),
            holds,
        }];
        let ContractGuardContextViewV1::Lowered(view) = package
            .guard_context(&application.node_id, &application, &guards)
            .expect("QSL-checked guard admits")
        else {
            panic!("RR-13 application lowers");
        };
        assert_eq!(view.application, application);
        assert_eq!(view.guards.len(), 1);
        assert_eq!(view.guards[0].occurrence, guard_occurrence);
        assert_eq!(view.guards[0].holds, holds);
        assert_eq!(
            view.guards[0].expression.node_id,
            view.guards[0].occurrence.node_id
        );
        assert_eq!(
            view.guards[0].source.role,
            CheckedOccurrenceRole::Expression
        );
        assert_eq!(
            view.guards[0].source.ordinal,
            view.guards[0].occurrence.ordinal
        );
    }
    holds_seen.sort_unstable();
    assert_eq!(holds_seen, [false, true, true]);
    assert!(
        guard_polarities.iter().any(|(occurrence, holds)| {
            *holds
                && guard_polarities
                    .iter()
                    .any(|(other_occurrence, other_holds)| {
                        !other_holds && occurrence == other_occurrence
                    })
        }),
        "QSL retains opposite polarity for the same checked guard occurrence"
    );
}
