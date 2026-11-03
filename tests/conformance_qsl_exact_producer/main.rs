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
        // Keep the invoking toolchain across path dependencies with their own
        // rustup directory overrides; this selects no new toolchain version.
        .env(
            "RUSTC",
            Path::new(env!("CARGO")).with_file_name(format!("rustc{}", env::consts::EXE_SUFFIX)),
        )
        .env(
            "RUSTDOC",
            Path::new(env!("CARGO")).with_file_name(format!("rustdoc{}", env::consts::EXE_SUFFIX)),
        )
        .output()
        .expect("run isolated typed producer consumer")
}

fn prepare_consumer(
    label: &str,
    source: &str,
    additional_packages: &[&str],
) -> (Scratch, PathBuf, PathBuf) {
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
    let scratch_path = env::temp_dir().join(format!("ir1003-{label}-{}", std::process::id()));
    fs::create_dir(&scratch_path).expect("create owned scratch");
    let scratch = Scratch(scratch_path);
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
    for name in additional_packages {
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
    fs::copy(root.join(source), helper.join("src/bin/producer.rs"))
        .expect("copy tracked consumer program, never producer rows or model fixtures");
    // The scratch package changes the workspace root and patches one dependency;
    // resolve its own lock before all build/run/check commands use --locked.
    let resolved = cargo(
        &helper,
        &target,
        &["metadata", "--offline", "--format-version=1"],
    );
    assert!(
        resolved.status.success(),
        "scratch dependency resolution: {}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    let metadata: Value =
        serde_json::from_slice(&resolved.stdout).expect("resolved Cargo metadata");
    let models = metadata["packages"]
        .as_array()
        .expect("resolved packages")
        .iter()
        .filter(|package| package["name"] == "quire-contract-model")
        .collect::<Vec<_>>();
    assert_eq!(
        models.len(),
        1,
        "the typed producer and consumer must share one model crate"
    );
    assert_eq!(
        Path::new(models[0]["manifest_path"].as_str().expect("model manifest")),
        root.join("crates/quire-contract-model/Cargo.toml")
    );
    (scratch, helper, target)
}

#[trace("FR-352-AC-9")]
#[test]
fn genuine_qsl_model_identities_join_admitted_record_and_reference_projection() {
    let (_scratch, helper, target) = prepare_consumer(
        "exact-producer",
        "tests/conformance_qsl_exact_producer/producer.rs",
        &[],
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

    print!("{}", String::from_utf8_lossy(&emitted.stdout));

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
        println!("{name}: E0308 rejects a bare input at the joined-witness boundary");
    }
}

#[trace("TC-462", "FR-058-AC-1", "FR-058-AC-6")]
#[test]
fn fresh_qsl_four_points_survive_source_drop_at_the_public_accessor() {
    assert_accessor_reads_only_admitted_terms();
    let (_scratch, helper, target) = prepare_consumer(
        "execution-points",
        "tests/conformance_qsl_exact_producer/execution_points.rs",
        &["qsl-replay"],
    );
    let produced = cargo(
        &helper,
        &target,
        &["run", "--offline", "--locked", "-j2", "--bin", "producer"],
    );
    assert!(
        produced.status.success(),
        "four-point producer and public consumer: {}\n{}",
        String::from_utf8_lossy(&produced.stdout),
        String::from_utf8_lossy(&produced.stderr)
    );
    let wire: Value = serde_json::from_slice(&produced.stdout).expect("fresh producer envelope");
    let qspec = env::var_os("QUIRE_SPECIFICATION_DIR")
        .map(PathBuf::from)
        .expect("explicit producer lane requires QUIRE_SPECIFICATION_DIR");
    let schema: Value = serde_json::from_slice(
        &fs::read(qspec.join("proposals/checked-package-v2/schema.json"))
            .expect("authoritative QSpec carrier schema by reference"),
    )
    .expect("QSpec schema JSON");
    let body_schema =
        serde_json::json!({"$ref":"#/$defs/StateClauseBody", "$defs":schema["$defs"]});
    let validator = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .compile(&body_schema)
        .expect("published state-clause carrier schema");
    let bodies = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .filter(|node| node["semantic_form"] == "state_clause")
        .map(|node| &node["body"])
        .collect::<Vec<_>>();
    assert_eq!(
        bodies.len(),
        4,
        "one fresh package contains all four clauses"
    );
    let mut points = Vec::new();
    for body in bodies {
        if let Err(errors) = validator.validate(body) {
            panic!(
                "fresh state clause violates QSpec: {:?}",
                errors.map(|error| error.to_string()).collect::<Vec<_>>()
            );
        }
        points.push(body["operation"]["member"]["execution_point"].to_string());
    }
    points.sort();
    let mut expected = [
        serde_json::json!({"kind":"initialization", "name":"startup"}).to_string(),
        serde_json::json!({"kind":"handler", "name":"tick"}).to_string(),
        serde_json::json!({"kind":"pre", "operation":"adjust"}).to_string(),
        serde_json::json!({"kind":"post", "operation":"adjust"}).to_string(),
    ];
    expected.sort();
    assert_eq!(
        points, expected,
        "exact points in the authoritative carrier"
    );

    fs::write(helper.join("src/bin/caller_point.rs"),
        "fn consume(p: &quire_contract_model::CheckedPackageV2, n: &quire_contract_model::CheckedNodeId, c: &quire_contract_model::CheckedOccurrence, supplied: quire_contract_model::ExecutionPoint) { let _ = p.clause_execution_point(n, (n,c), supplied); } fn main() {}\n"
    ).expect("external caller-point rejection control");
    let refused = cargo(
        &helper,
        &target,
        &[
            "check",
            "--offline",
            "--locked",
            "-j2",
            "--bin",
            "caller_point",
            "--message-format=json",
        ],
    );
    assert_eq!(
        refused.status.code(),
        Some(101),
        "caller point must not enter accessor"
    );
    let messages = String::from_utf8(refused.stdout).expect("compiler JSON");
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
        "only the intended signature refusal: {errors:?}\n{}",
        String::from_utf8_lossy(&refused.stderr)
    );
    assert_eq!(errors[0]["target"]["name"], "caller_point");
    assert_eq!(errors[0]["message"]["code"]["code"], "E0061");
}

fn assert_accessor_reads_only_admitted_terms() {
    use std::collections::BTreeSet;
    use syn::visit::Visit;

    #[derive(Default)]
    struct Calls {
        methods: BTreeSet<String>,
        functions: BTreeSet<String>,
    }
    impl<'ast> Visit<'ast> for Calls {
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            self.methods.insert(call.method.to_string());
            syn::visit::visit_expr_method_call(self, call);
        }
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            let syn::Expr::Path(path) = call.func.as_ref() else {
                panic!("accessor cannot dispatch through a supplied callback");
            };
            self.functions.insert(
                path.path
                    .segments
                    .last()
                    .expect("call path")
                    .ident
                    .to_string(),
            );
            syn::visit::visit_expr_call(self, call);
        }
    }
    let source = include_str!(
        "../../crates/quire-contract-model/src/checked_package/v2/execution_point_accessor.rs"
    );
    let file = syn::parse_file(source).expect("accessor Rust syntax");
    let functions = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(item) => Some(item),
            _ => None,
        })
        .flat_map(|item| &item.items)
        .filter_map(|item| match item {
            syn::ImplItem::Fn(function) if function.sig.ident == "clause_execution_point" => {
                Some(function)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        functions.len(),
        1,
        "one admitted accessor, no overload path"
    );
    assert_eq!(
        functions[0].sig.inputs.len(),
        3,
        "self, clause, owner-bound claim only"
    );
    let mut calls = Calls::default();
    calls.visit_block(&functions[0].block);
    assert_eq!(
        calls.functions,
        ["Some", "error"].map(str::to_owned).into_iter().collect()
    );
    assert_eq!(calls.methods, ["clone", "cloned", "contains", "execution_point", "node", "node_kind", "ok_or_else", "term"].map(str::to_owned).into_iter().collect(), "the accessor obtains only admitted typed nodes/terms, never source bindings or a fallback producer path");
}
