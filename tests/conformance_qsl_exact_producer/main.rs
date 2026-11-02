// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Explicit live QSL producer lane. Compile this target first, then execute its
//! native test binary; only its isolated helper owns Cargo during execution.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use ix_trace_rs::trace;
use serde_json::Value;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned producer helper scratch");
    }
}

fn cargo(helper: &Path, target: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO"))
        .current_dir(helper)
        .args(arguments)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("run isolated typed producer consumer")
}

#[trace("FR-352-AC-9")]
#[test]
fn genuine_qsl_model_identities_join_admitted_record_and_reference_projection() {
    let qsl = env::var_os("QUIRE_SPEC_LANGUAGE_DIR")
        .map(PathBuf::from)
        .expect("explicit producer lane requires QUIRE_SPEC_LANGUAGE_DIR");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    assert!(
        target.is_absolute(),
        "producer target must be an absolute owned path"
    );
    let scratch =
        Scratch(env::temp_dir().join(format!("ir1003-exact-producer-{}", std::process::id())));
    fs::create_dir(&scratch.0).expect("create owned scratch");
    let helper = scratch.0.join("helper");
    fs::create_dir_all(helper.join("src/bin")).expect("create helper bins");
    let mut manifest = String::from(
        "[package]\nname = \"ir1003-exact-producer\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    );
    for name in [
        "qsl-cst",
        "qsl-forms",
        "qsl-foundation",
        "qsl-semantics",
        "qsl-package",
    ] {
        manifest.push_str(&format!("{name} = {{ path = {:?} }}\n", qsl.join(name)));
    }
    manifest.push_str(&format!(
        "quire-contract-model = {{ path = {:?} }}\n\
         quire-exact = {{ git = \"https://github.com/agent-ix/quire-exact\", branch = \"main\" }}\n\
         quire-semantic-value = {{ git = \"https://github.com/agent-ix/quire-semantic-value\", branch = \"main\" }}\n\
         serde_json = \"1\"\n\n\
         [patch.\"https://github.com/agent-ix/quire-contract-ir\"]\n\
         quire-contract-model = {{ path = {:?} }}\n",
        root.join("crates/quire-contract-model"), root.join("crates/quire-contract-model"),
    ));
    fs::write(helper.join("Cargo.toml"), manifest).expect("write isolated consumer manifest");
    fs::copy(qsl.join("Cargo.lock"), helper.join("Cargo.lock"))
        .expect("seed scratch resolution from the actual owner workspace");
    fs::copy(
        root.join("tests/conformance_qsl_exact_producer/producer.rs"),
        helper.join("src/bin/producer.rs"),
    )
    .expect("copy tracked consumer program, never producer rows or model fixtures");
    // The scratch package changes the workspace root and patches one dependency;
    // resolve its own lock before all build/run/check commands use --locked.
    let resolved = cargo(&helper, &target, &["generate-lockfile", "--offline"]);
    assert!(
        resolved.status.success(),
        "scratch dependency resolution: {}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    let emitted = cargo(
        &helper,
        &target,
        &["run", "--offline", "--locked", "-j2", "--bin", "producer"],
    );
    assert!(
        emitted.status.success(),
        "typed producer consumer failed:\n{}\n{}",
        String::from_utf8_lossy(&emitted.stdout),
        String::from_utf8_lossy(&emitted.stderr)
    );
    assert!(String::from_utf8_lossy(&emitted.stdout).contains(
        "FR-352-AC-9 producer, record/reference projections and typed adverse joins passed"
    ));

    // The public projection signature requires the joined table. These probes
    // establish that typed boundary, not unforgeability of public stage rows.
    for (name, supplied) in [
        ("bare_node_key", "quire_exact::NodeKey"),
        ("bare_effective_id", "quire_exact::EffectiveId"),
        ("bare_member_id", "quire_exact::MemberId"),
        ("bare_name_pair", "(&str, &str)"),
    ] {
        let source = format!(
            "fn consume(package: &quire_contract_model::CheckedPackageV2, node: &quire_contract_model::CheckedNodeId, supplied: &{supplied}) {{\n\
             let _ = package.project_exact_type(node, supplied, 256);\n}}\nfn main() {{}}\n"
        );
        fs::write(helper.join(format!("src/bin/{name}.rs")), source)
            .expect("write compile-negative consumer");
        let result = cargo(
            &helper,
            &target,
            &[
                "check",
                "--offline",
                "--locked",
                "-j2",
                "--bin",
                name,
                "--message-format=json",
            ],
        );
        assert!(
            !result.status.success(),
            "{name} entered projection without a joined witness"
        );
        let messages = String::from_utf8(result.stdout).expect("Cargo diagnostic JSON");
        let errors = messages
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|message| {
                message["reason"] == "compiler-message" && message["message"]["level"] == "error"
            })
            .collect::<Vec<_>>();
        assert_eq!(
            errors.len(),
            1,
            "{name}: wrong failure: {errors:?}\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(errors[0]["message"]["code"]["code"], "E0308", "{name}");
        assert!(
            errors[0]["message"]["rendered"]
                .as_str()
                .expect("rendered diagnostic")
                .contains("SelectedExactIdentities"),
            "{name}: {errors:?}"
        );
    }
}
