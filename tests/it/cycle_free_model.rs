// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    process::Command,
};

use ix_trace_rs::trace;
#[cfg(feature = "conformance-cli")]
use quire_contract_model::{run_corpus, FixtureStatus, RunnerErrorCode};
use serde_json::{json, Value};
#[cfg(feature = "conformance-cli")]
use std::path::Path;
#[cfg(feature = "conformance-cli")]
use std::process::Output;
use syn::visit::{self, Visit};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn metadata(features: &[String]) -> Value {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1"])
        .args(features)
        .arg("--manifest-path")
        .arg(repository().join("Cargo.toml"))
        .output()
        .expect("cargo metadata must start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata must emit JSON")
}

fn package<'a>(metadata: &'a Value, name: &str) -> &'a Value {
    metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"] == name)
        .unwrap_or_else(|| panic!("metadata must contain {name}"))
}

/// Trace: FR-353-AC-5.
/// Provenance: IR-1209 scoped model dependency and public boundary.
#[test]
fn equality_admission_api_and_resolved_dependency_direction() {
    use syn::{Fields, FnArg, Item, Meta, ReturnType, Type, Visibility};

    let graph = metadata(&[]);
    let model = package(&graph, "quire-contract-model");
    let root = package(&graph, "quire-contract-ir");
    let qsv = package(&graph, "quire-semantic-value");
    let exact = package(&graph, "quire-exact");
    let resolved = |id: &Value| {
        graph["resolve"]["nodes"]
            .as_array()
            .expect("resolved nodes")
            .iter()
            .find(|node| &node["id"] == id)
            .expect("resolved package node")
    };
    let edge = |from: &Value, to: &Value| {
        resolved(&from["id"])["deps"]
            .as_array()
            .expect("resolved dependencies")
            .iter()
            .any(|dependency| dependency["pkg"] == to["id"])
    };
    assert!(edge(root, model));
    assert!(edge(model, qsv));
    assert!(edge(qsv, exact));
    assert!(!edge(root, qsv));
    assert!(!edge(qsv, model));
    for ir in [root, model] {
        assert!(resolved(&ir["id"])["deps"]
            .as_array()
            .expect("dependencies")
            .iter()
            .all(
                |dependency| !dependency["name"].as_str().unwrap_or("").starts_with("qsl")
                    && dependency["name"] != "quire_spec_language"
            ));
    }

    let raw_carrier = syn::parse_file(include_str!(
        "../../crates/quire-contract-model/src/checked_package/v2/checked_term_data.rs"
    ))
    .expect("IR raw sidecar syntax");
    let raw = raw_carrier
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(value) if value.ident == "CheckedEqualityDecision" => Some(value),
            _ => None,
        })
        .expect("IR raw equality carrier");
    assert!(matches!(raw.vis, Visibility::Public(_)));
    let Fields::Named(raw_fields) = &raw.fields else {
        panic!("raw carrier has named fields")
    };
    assert!(raw_fields
        .named
        .iter()
        .all(|field| matches!(field.vis, Visibility::Public(_))));
    assert!(raw
        .attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("derive")
            && matches!(&attribute.meta, Meta::List(meta) if meta.tokens.to_string().split(',')
            .any(|trait_name| trait_name.trim() == "Deserialize"))));

    let qsv_root = PathBuf::from(qsv["manifest_path"].as_str().expect("QSV manifest"))
        .parent()
        .expect("QSV directory")
        .to_path_buf();
    let admitted = syn::parse_file(
        &fs::read_to_string(qsv_root.join("src/declaration/admitted.rs"))
            .expect("QSV admitted API source"),
    )
    .expect("QSV admitted API syntax");
    let value = admitted
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(value) if value.ident == "AdmittedEqualityDecision" => Some(value),
            _ => None,
        })
        .expect("QSV admitted type");
    assert!(matches!(value.vis, Visibility::Public(_)));
    let Fields::Named(fields) = &value.fields else {
        panic!("admitted value has named private fields")
    };
    assert!(fields
        .named
        .iter()
        .all(|field| matches!(field.vis, Visibility::Inherited)));
    for item in &admitted.items {
        if let Item::Impl(implementation) = item {
            let Type::Path(path) = implementation.self_ty.as_ref() else {
                continue;
            };
            if !path.path.is_ident("AdmittedEqualityDecision") {
                continue;
            }
            for member in &implementation.items {
                if let syn::ImplItem::Fn(method) = member {
                    if matches!(method.vis, Visibility::Public(_)) {
                        assert!(
                            !matches!(&method.sig.output, ReturnType::Type(_, ty)
                            if matches!(ty.as_ref(), Type::Path(returned) if returned.path.is_ident("Self")
                                || returned.path.is_ident("AdmittedEqualityDecision"))),
                            "public raw-field constructor must not appear"
                        );
                    }
                }
            }
        }
    }
    let reader = syn::parse_file(
        &fs::read_to_string(qsv_root.join("src/reader_admission.rs"))
            .expect("QSV reader API source"),
    )
    .expect("QSV reader API syntax");
    let mint = reader
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(function) if function.sig.ident == "mint_admitted_equality_decision" => {
                Some(function)
            }
            _ => None,
        })
        .expect("hidden mint");
    assert!(matches!(mint.vis, Visibility::Public(_)));
    assert!(mint.attrs.iter().any(|attribute| matches!(&attribute.meta,
        Meta::List(meta) if meta.path.is_ident("doc") && meta.tokens.to_string() == "hidden")));
    assert!(
        matches!(mint.sig.inputs.first(), Some(FnArg::Typed(argument))
        if matches!(argument.ty.as_ref(), Type::Path(path) if path.path.is_ident("ReaderSeal")))
    );

    fn exports(tree: &syn::UseTree, name: &str) -> bool {
        match tree {
            syn::UseTree::Path(path) => exports(&path.tree, name),
            syn::UseTree::Name(item) => item.ident == name,
            syn::UseTree::Rename(item) => item.rename == name,
            syn::UseTree::Group(group) => group.items.iter().any(|item| exports(item, name)),
            syn::UseTree::Glob(_) => panic!("public API export must be explicit"),
        }
    }
    let ir_api = syn::parse_file(include_str!("../../crates/quire-contract-model/src/lib.rs"))
        .expect("IR public API syntax");
    for item in ir_api.items {
        if let Item::Use(export) = item {
            if matches!(export.vis, Visibility::Public(_)) {
                assert!(
                    !exports(&export.tree, "ReaderSeal")
                        && !exports(&export.tree, "mint_admitted_equality_decision")
                );
            }
        }
    }
}
const FORBIDDEN_DOWNSTREAM_CRATES: [&str; 2] = ["quire-contract-codegen", "quire-contract-runtime"];

fn forbidden(package: &Value) -> bool {
    let name = package["name"].as_str().unwrap();
    [
        "quire-contract-ir",
        "quire-spec-language",
        "quire-observation",
        "quire-protocol",
        "quire-checked",
    ]
    .contains(&name)
        || FORBIDDEN_DOWNSTREAM_CRATES.contains(&name)
        || name.starts_with("qsl-")
        || name.starts_with("tl-")
        || name == "quire-mltl"
        || [
            "quire-spec-language",
            "quire-observation",
            "quire-protocol",
            "quire-checked",
            "tl-syntax",
            "tl-parse",
            "tl-mltl",
            "tl-rewrite",
        ]
        .iter()
        .any(|owner| {
            package["source"]
                .as_str()
                .unwrap_or("")
                .contains(&format!("github.com/agent-ix/{owner}"))
        })
}

// Cargo's resolved IDs retain renamed and transitive dependencies; manifest
// dependency names alone cannot establish production reachability.
fn production_graph(
    metadata: &Value,
    root: &str,
    exclude_owners: bool,
) -> Result<BTreeSet<String>, String> {
    let packages: BTreeMap<_, _> = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|package| (package["id"].as_str().unwrap(), package))
        .collect();
    let nodes: BTreeMap<_, _> = metadata["resolve"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| (node["id"].as_str().unwrap(), node))
        .collect();
    let mut active = BTreeSet::new();
    let mut complete = BTreeSet::new();
    let mut pending = vec![(root, false)];
    while let Some((id, leaving)) = pending.pop() {
        if leaving {
            active.remove(id);
            complete.insert(id.to_owned());
            continue;
        }
        if complete.contains(id) {
            continue;
        }
        if !active.insert(id) {
            return Err(format!("production dependency cycle at {id}"));
        }
        if exclude_owners && forbidden(packages[id]) {
            return Err(format!("forbidden production dependency {id}"));
        }
        pending.push((id, true));
        for edge in nodes[id]["deps"].as_array().unwrap() {
            let production = edge["dep_kinds"].as_array().unwrap().iter().any(|kind| {
                match kind["kind"].as_str() {
                    None | Some("build") => true,
                    Some("dev") => false,
                    other => panic!("unknown Cargo dependency kind {other:?}"),
                }
            });
            if production {
                pending.push((edge["pkg"].as_str().unwrap(), false));
            }
        }
    }
    Ok(complete)
}

#[trace("TC-041", "FR-028-AC-1", "FR-349-AC-9")]
#[test]
fn resolved_production_graph_is_acyclic_and_owner_free_in_each_feature_configuration() {
    let default = metadata(&[]);
    let model = package(&default, "quire-contract-model");
    let features: Vec<_> = model["features"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|name| name.as_str() != "default")
        .cloned()
        .collect();
    let mut combinations = vec![Vec::<String>::new()];
    for feature in features {
        let additions: Vec<_> = combinations
            .iter()
            .map(|selected| {
                let mut selected = selected.clone();
                selected.push(format!("quire-contract-model/{feature}"));
                selected
            })
            .collect();
        combinations.extend(additions);
    }
    // Workspace metadata may unify the root's dev-dependency features into the
    // model even for --no-default-features. These graphs are conservative
    // supersets, not claims about which features a production build disables.
    // The all-feature graph must activate every advertised model feature; its
    // owner-free acyclic production edges cover every feature subset as well.
    let all_features: BTreeSet<_> = model["features"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    let mut configurations = vec![Vec::new(), vec!["--all-features".to_owned()]];
    configurations.extend(combinations.into_iter().map(|features| {
        let mut args = vec!["--no-default-features".to_owned()];
        if !features.is_empty() {
            args.extend(["--features".to_owned(), features.join(",")]);
        }
        args
    }));
    for configuration in configurations {
        let resolved = metadata(&configuration);
        let model_id = package(&resolved, "quire-contract-model")["id"]
            .as_str()
            .unwrap();
        if configuration.iter().any(|arg| arg == "--all-features") {
            let node = resolved["resolve"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|node| node["id"] == model_id)
                .unwrap();
            let activated: BTreeSet<_> = node["features"]
                .as_array()
                .unwrap()
                .iter()
                .map(|feature| feature.as_str().unwrap().to_owned())
                .collect();
            assert!(
                all_features.is_subset(&activated),
                "all-feature graph must cover the model's complete declared feature set"
            );
        }
        production_graph(&resolved, model_id, true)
            .unwrap_or_else(|error| panic!("{configuration:?}: {error}"));
        for member in resolved["workspace_members"].as_array().unwrap() {
            production_graph(&resolved, member.as_str().unwrap(), false).unwrap();
        }
    }
}

#[trace("TC-041", "FR-028-AC-1")]
#[test]
fn graph_control_detects_indirect_owners_build_edges_and_cycles_but_excludes_dev_edges() {
    let edge = |id: &str, kind: Value| json!({"pkg": id, "dep_kinds": [{"kind": kind}]});
    let mut graph = json!({"packages": [
        {"id": "model", "name": "quire-contract-model"},
        {"id": "middle", "name": "ordinary-library"},
        {"id": "owner", "name": "tl-syntax"}],
        "resolve": {"nodes": [
            {"id": "model", "deps": [edge("middle", Value::Null)]},
            {"id": "middle", "deps": [edge("owner", Value::Null)]},
            {"id": "owner", "deps": []}]}});
    assert!(production_graph(&graph, "model", true)
        .unwrap_err()
        .contains("owner"));
    graph["resolve"]["nodes"][1]["deps"][0]["dep_kinds"][0]["kind"] = json!("build");
    assert!(production_graph(&graph, "model", true)
        .unwrap_err()
        .contains("owner"));
    graph["resolve"]["nodes"][1]["deps"][0]["dep_kinds"][0]["kind"] = json!("dev");
    assert_eq!(
        production_graph(&graph, "model", true).unwrap(),
        BTreeSet::from(["model".into(), "middle".into()])
    );
    graph["resolve"]["nodes"][1]["deps"] =
        json!([{"pkg": "owner", "dep_kinds": [{"kind": null, "target": "cfg(windows)"}]}]);
    assert!(production_graph(&graph, "model", true)
        .unwrap_err()
        .contains("owner"));
    graph["resolve"]["nodes"][1]["deps"] = json!([edge("model", Value::Null)]);
    assert!(production_graph(&graph, "model", true)
        .unwrap_err()
        .contains("cycle"));
}

#[trace("TC-041", "FR-028-AC-1")]
#[test]
fn model_graph_refuses_downstream_crate_names() {
    for name in FORBIDDEN_DOWNSTREAM_CRATES {
        let graph = json!({"packages": [
            {"id": "model", "name": "quire-contract-model"},
            {"id": "downstream", "name": name}],
            "resolve": {"nodes": [
                {"id": "model", "deps": [{"pkg": "downstream", "dep_kinds": [{"kind": null}]}]},
                {"id": "downstream", "deps": []}]}});
        assert_eq!(
            production_graph(&graph, "model", true),
            Err("forbidden production dependency downstream".to_owned()),
            "AD-005 D-2 must refuse {name}"
        );
    }
}

#[trace("TC-041", "FR-028-AC-3")]
#[test]
fn root_manifest_has_no_qsl_or_downstream_dependency_in_any_kind() {
    let workspace = metadata(&[]);
    let root = package(&workspace, "quire-contract-ir");
    for dependency in root["dependencies"].as_array().unwrap() {
        let name = dependency["name"].as_str().unwrap();
        assert!(
            name != "quire-spec-language" && !name.starts_with("qsl-"),
            "QSL dependency {name}"
        );
        assert!(
            !FORBIDDEN_DOWNSTREAM_CRATES.contains(&name),
            "AD-005 D-2 forbids root dependency {name}"
        );
        assert!(!dependency["source"]
            .as_str()
            .unwrap_or("")
            .contains("github.com/agent-ix/quire-spec-language"));
    }
    assert!(root["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .any(|dependency| dependency["name"] == "quire-contract-model"));
}

#[trace("TC-041", "FR-028-AC-4")]
#[test]
#[ignore = "explicit default/all-feature/MSRV build lane: make model-boundary-builds"]
fn production_default_all_feature_and_msrv_builds() {
    let workspace = metadata(&[]);
    let minimum = package(&workspace, "quire-contract-model")["rust_version"]
        .as_str()
        .unwrap();
    assert_eq!(
        package(&workspace, "quire-contract-ir")["rust_version"],
        minimum
    );
    for (toolchain, features) in [(None, false), (None, true), (Some(minimum), true)] {
        let mut command = if let Some(toolchain) = toolchain {
            let mut command = Command::new("rustup");
            command.args(["run", toolchain, "cargo"]);
            command
        } else {
            Command::new(env!("CARGO"))
        };
        command
            .args([
                "build",
                "--locked",
                "--workspace",
                "--lib",
                "--bins",
                "--jobs",
                "4",
            ])
            .current_dir(repository())
            .env("CARGO_TARGET_DIR", repository().join("target"));
        if features {
            command.arg("--all-features");
        }
        let status = command.status().expect("production build must start");
        assert!(
            status.success(),
            "toolchain={toolchain:?} all_features={features}"
        );
    }
}

#[cfg(feature = "conformance-cli")]
fn runner(schemas: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quire-contract-conformance"))
        .args(["run", "--corpus"])
        .arg(repository().join("corpus/contract-v0.1"))
        .arg("--schemas")
        .arg(schemas)
        .output()
        .unwrap()
}

#[cfg(feature = "conformance-cli")]
fn jsonl<T: serde::Serialize>(rows: &[T]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for row in rows {
        serde_json::to_writer(&mut bytes, row).unwrap();
        bytes.push(b'\n');
    }
    bytes
}

#[trace("TC-041", "FR-028-AC-2")]
#[cfg(feature = "conformance-cli")]
#[test]
fn model_and_root_runner_agree_on_schema_canonical_identity_diagnostic_and_corpus_results() {
    let schemas = repository().join("schemas");
    let results = run_corpus(&repository().join("corpus/contract-v0.1"), &schemas).unwrap();
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .all(|row| row.status() == FixtureStatus::Match),
        "actual results must also match the independent recorded corpus expectations"
    );
    let rows: Vec<_> = results
        .iter()
        .map(|row| serde_json::to_value(row).unwrap())
        .collect();
    assert_eq!(
        rows.iter()
            .map(|row| row["operation"].as_str().unwrap())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["package", "expression", "coverage"])
    );
    let canonical = rows
        .iter()
        .find(|row| {
            row["actual"]["canonical"]
                .as_array()
                .is_some_and(|values| !values.is_empty())
        })
        .unwrap();
    for field in ["bytes", "identity"] {
        assert!(!canonical["actual"]["canonical"][0][field]
            .as_str()
            .unwrap()
            .is_empty());
    }
    let diagnostic = rows
        .iter()
        .find(|row| {
            row["actual"]["diagnostics"]
                .as_array()
                .is_some_and(|values| !values.is_empty())
        })
        .unwrap();
    assert!(diagnostic["actual"]["diagnostics"][0]["code"].is_string());
    let root = runner(&schemas);
    assert!(
        root.status.success(),
        "{}",
        String::from_utf8_lossy(&root.stderr)
    );
    assert!(root.stderr.is_empty());
    // Serialize the actual typed results in the same declaration order as the
    // CLI. Converting structs to Value first changes object-key order.
    let model_bytes = jsonl(&results);
    assert_eq!(root.stdout, model_bytes);
    let root_rows: Vec<Value> = root
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| serde_json::from_slice(row).unwrap())
        .collect();
    assert_eq!(rows, root_rows);
    // Each named result category participates in the byte comparison.
    for pointer in ["/actual/canonical/0/bytes", "/actual/canonical/0/identity"] {
        let mut changed = rows.clone();
        let row = changed
            .iter_mut()
            .find(|row| row.pointer(pointer).is_some())
            .unwrap();
        *row.pointer_mut(pointer).unwrap() = json!("adverse-result");
        assert_ne!(changed, root_rows, "{pointer}");
    }
    let mut changed = rows.clone();
    let row = changed
        .iter_mut()
        .find(|row| row.pointer("/actual/diagnostics/0/code").is_some())
        .unwrap();
    *row.pointer_mut("/actual/diagnostics/0/code").unwrap() = json!("adverse-diagnostic");
    assert_ne!(changed, root_rows);
    changed[0]["fixture_id"] = json!("adverse-corpus-row");
    assert_ne!(changed, root_rows);
}

#[cfg(feature = "conformance-cli")]
struct ScratchSchemas(PathBuf);
#[cfg(feature = "conformance-cli")]
impl Drop for ScratchSchemas {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[trace("TC-041", "FR-028-AC-2")]
#[cfg(feature = "conformance-cli")]
#[test]
fn both_paths_consume_and_refuse_a_changed_package_schema() {
    let scratch = ScratchSchemas(
        std::env::temp_dir().join(format!("ir-tc041-schema-{}", std::process::id())),
    );
    fs::create_dir(&scratch.0).unwrap();
    for name in [
        "contract-package-reference-v1.schema.json",
        "contract-conformance-fixture-v1.schema.json",
    ] {
        let mut schema: Value =
            serde_json::from_slice(&fs::read(repository().join("schemas").join(name)).unwrap())
                .unwrap();
        if name == "contract-package-reference-v1.schema.json" {
            schema["definitions"]["package"]["required"]
                .as_array_mut()
                .unwrap()
                .push(json!("adverse_required_member"));
        }
        fs::write(scratch.0.join(name), serde_json::to_vec(&schema).unwrap()).unwrap();
    }
    let model = run_corpus(&repository().join("corpus/contract-v0.1"), &scratch.0).unwrap_err();
    assert_eq!(model.code(), RunnerErrorCode::InvalidCorpus);
    let root = runner(&scratch.0);
    assert_eq!(root.status.code(), Some(2));
    assert!(root.stdout.is_empty());
    let mut expected = serde_json::to_vec(&model).unwrap();
    expected.push(b'\n');
    assert_eq!(root.stderr, expected);
}

// This check concerns owner syntax/admission surfaces, not the model's own
// semantic types (e.g. ProtocolForm) or its dependency enumeration callback.
fn owner_symbol(name: &str) -> bool {
    [
        "ParsedUnit",
        "LinkedPackage",
        "NativeModel",
        "FormalSource",
        "AdmittedPackage",
        "ValidatedResult",
        "ProtocolAssessment",
        "CompleteGlobalConformance",
        "PastProfileManifest",
        "PastProfileManifestError",
    ]
    .contains(&name)
        || (name.starts_with("Qsl")
            && !["Qsl", "QslCheckedRequirementsAndPendingObligations"].contains(&name))
        || name.starts_with("QSL")
        || name.starts_with("ObservationWire")
        || name.starts_with("ProtocolWire")
}

#[derive(Default)]
struct AdmissionSignature {
    has_callback: bool,
}
impl<'ast> Visit<'ast> for AdmissionSignature {
    fn visit_type_trait_object(&mut self, _: &'ast syn::TypeTraitObject) {
        self.has_callback = true;
    }
    fn visit_type_bare_fn(&mut self, _: &'ast syn::TypeBareFn) {
        self.has_callback = true;
    }
    fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
        if ["Fn", "FnMut", "FnOnce"].contains(&segment.ident.to_string().as_str()) {
            self.has_callback = true;
        }
        visit::visit_path_segment(self, segment);
    }
}

fn admission_callback(signature: &syn::Signature) -> bool {
    // FR-019's DependencySource enumerates already-owned semantic dependencies;
    // it does not outsource validation or admit an owner value.
    if signature.ident == "visit_dependencies" {
        return false;
    }
    let mut surface = AdmissionSignature::default();
    surface.visit_signature(signature);
    surface.has_callback
}

fn checked_node_id(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("CheckedNodeId"))
}

fn optional_checked_node_id(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return false;
    }
    let segment = &path.path.segments[0];
    matches!(&segment.arguments, syn::PathArguments::AngleBracketed(arguments)
        if segment.ident == "Option" && arguments.args.len() == 1
            && matches!(&arguments.args[0], syn::GenericArgument::Type(ty) if checked_node_id(ty)))
}

fn node_id_map_callback(signature: &syn::Signature) -> bool {
    // FR-048 permits only this typed draft visitor, not an admission callback.
    if signature.ident != "map_node_ids"
        || signature.inputs.len() != 2
        || !signature.generics.params.is_empty()
        || signature.constness.is_some()
        || signature.asyncness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !matches!(signature.output, syn::ReturnType::Default)
    {
        return false;
    }
    let mut inputs = signature.inputs.iter();
    if !matches!(inputs.next(), Some(syn::FnArg::Receiver(receiver)) if receiver.reference.is_some() && receiver.mutability.is_some())
    {
        return false;
    }
    let Some(syn::FnArg::Typed(map)) = inputs.next() else {
        return false;
    };
    let syn::Type::ImplTrait(implementation) = map.ty.as_ref() else {
        return false;
    };
    let mut bounds = implementation.bounds.iter();
    let Some(syn::TypeParamBound::Trait(bound)) = bounds.next() else {
        return false;
    };
    if bounds.next().is_some() || bound.path.segments.len() != 1 {
        return false;
    }
    let segment = &bound.path.segments[0];
    let syn::PathArguments::Parenthesized(args) = &segment.arguments else {
        return false;
    };
    if segment.ident != "FnMut" || args.inputs.len() != 1 {
        return false;
    }
    if !matches!(&args.inputs[0], syn::Type::Reference(reference) if checked_node_id(&reference.elem))
    {
        return false;
    }
    matches!(&args.output, syn::ReturnType::Type(_, output) if optional_checked_node_id(output))
}

#[derive(Default)]
struct Boundary {
    violations: BTreeSet<String>,
    impl_owner: Option<String>,
}
impl<'ast> Visit<'ast> for Boundary {
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let previous = self.impl_owner.take();
        self.impl_owner = match item.self_ty.as_ref() {
            syn::Type::Path(path)
                if item.trait_.is_none()
                    && item.generics.params.is_empty()
                    && path.qself.is_none()
                    && path.path.segments.len() == 1 =>
            {
                Some(path.path.segments[0].ident.to_string())
            }
            _ => None,
        };
        visit::visit_item_impl(self, item);
        self.impl_owner = previous;
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        if matches!(item.vis, syn::Visibility::Public(_)) {
            let mut surface = AdmissionSignature::default();
            surface.visit_type(&item.ty);
            if surface.has_callback {
                self.violations.insert(item.ident.to_string());
            }
        }
        visit::visit_item_type(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if matches!(item.vis, syn::Visibility::Public(_)) && admission_callback(&item.sig) {
            self.violations
                .insert(format!("public callback {}", item.sig.ident));
        }
        visit::visit_item_fn(self, item);
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let is_typed_map = matches!(
            self.impl_owner.as_deref(),
            Some("CheckedSemanticNodeV2" | "CheckedTerm")
        ) && node_id_map_callback(&item.sig);
        if matches!(item.vis, syn::Visibility::Public(_))
            && !is_typed_map
            && admission_callback(&item.sig)
        {
            self.violations
                .insert(format!("public callback {}", item.sig.ident));
        }
        visit::visit_impl_item_fn(self, item);
    }
    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        if admission_callback(&item.sig) {
            self.violations
                .insert(format!("trait callback {}", item.sig.ident));
        }
        visit::visit_trait_item_fn(self, item);
    }
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if item.ident.to_string().ends_with("Validator") {
            self.violations.insert(item.ident.to_string());
        }
        visit::visit_item_trait(self, item);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if item.attrs.iter().any(|attr| {
            attr.path().is_ident("cfg")
                && attr
                    .parse_args::<syn::Path>()
                    .is_ok_and(|path| path.is_ident("test"))
        }) {
            return;
        }
        visit::visit_item_mod(self, item);
    }
    fn visit_ident(&mut self, ident: &'ast syn::Ident) {
        let name = ident.to_string();
        if owner_symbol(&name)
            || [
                "parse_qsl",
                "parse_native_source",
                "parse_observation",
                "parse_protocol",
                "parse_tl",
                "validate_past_profile_implementation",
            ]
            .contains(&name.as_str())
        {
            self.violations.insert(name);
        }
    }
    fn visit_field(&mut self, field: &'ast syn::Field) {
        if let Some(name) = &field.ident {
            if ["trusted", "trust", "skip_validation", "already_validated"]
                .contains(&name.to_string().as_str())
            {
                self.violations.insert(name.to_string());
            }
        }
        visit::visit_field(self, field);
    }
    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        let name = signature.ident.to_string();
        if [
            "validate_with",
            "admit_with",
            "from_owner",
            "from_validated",
            "new_unchecked",
        ]
        .contains(&name.as_str())
        {
            self.violations.insert(name);
        }
        visit::visit_signature(self, signature);
    }
}

fn boundary(source: &str) -> BTreeSet<String> {
    let mut check = Boundary::default();
    check.visit_file(&syn::parse_file(source).expect("production Rust must parse"));
    check.violations
}

#[trace("TC-041", "FR-028-AC-5")]
#[test]
fn production_boundary_has_no_owner_mirrors_parsers_or_admission_escape_surface() {
    for directory in ["crates/quire-contract-model/src", "src"] {
        let mut pending = vec![repository().join(directory)];
        while let Some(path) = pending.pop() {
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().is_some_and(|extension| extension == "rs") {
                    assert!(
                        boundary(&fs::read_to_string(&path).unwrap()).is_empty(),
                        "owner boundary violation in {}",
                        path.display()
                    );
                }
            }
        }
    }
    let root = syn::parse_file(include_str!("../../src/lib.rs")).unwrap();
    for item in root.items {
        match item {
            syn::Item::Mod(module) => assert_eq!(module.ident, "kani"),
            _ => panic!("root must expose only its bounded-Kani module"),
        }
    }
}

#[trace("TC-041", "FR-028-AC-5")]
#[test]
fn boundary_control_rejects_copied_owner_types_parsers_callbacks_and_trust_flags() {
    assert!(boundary(
        "pub enum SourceInventoryOwner { Qsl } pub enum SourceInventoryScope { QslCheckedRequirementsAndPendingObligations }"
    )
    .is_empty());
    for (source, violation) in [
        ("pub struct QslRequirementsWire;", "QslRequirementsWire"),
        ("pub struct ParsedUnit { pub text: String }", "ParsedUnit"),
        ("pub fn parse_qsl(_: &str) {}", "parse_qsl"),
        (
            "pub fn validate_with(_: &dyn Fn(&str) -> bool) {}",
            "validate_with",
        ),
        ("pub struct Wire { pub trusted: bool }", "trusted"),
        (
            "pub struct Owner; impl Owner { pub fn new_unchecked() -> Self { Self } }",
            "new_unchecked",
        ),
    ] {
        assert!(
            boundary(source).contains(violation),
            "adverse control {violation}"
        );
    }
    for source in [
        "pub fn admit<F: Fn(&str) -> bool>(validator: F) {}",
        "pub fn admit(validator: &dyn ExternalValidator) {}",
        "pub trait OwnerValidator { fn validate(&self, bytes: &[u8]) -> bool; }",
        "pub fn admit(validator: fn(&str) -> bool) {}",
        "pub type Gate = dyn Fn(&[u8]) -> bool; pub fn admit(gate: &Gate) {}",
        "pub type Gate = fn(&[u8]) -> bool; pub fn admit(gate: Gate) {}",
    ] {
        assert!(!boundary(source).is_empty(), "callback control {source}");
    }
    for owner in ["CheckedSemanticNodeV2", "CheckedTerm"] {
        let source = format!(
            "impl {owner} {{ pub fn map_node_ids(&mut self, map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>) {{}} }}"
        );
        assert!(boundary(&source).is_empty(), "typed draft visitor {owner}");
    }
    for source in [
        "pub fn map_node_ids(map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>) {}",
        "impl Other { pub fn map_node_ids(&mut self, map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>) {} }",
        "impl CheckedTerm { pub fn map_node_ids(&mut self, map: impl FnMut(&str) -> Option<CheckedNodeId>) {} }",
        "impl CheckedSemanticNodeV2 { pub fn map_node_ids(&mut self, map: impl FnMut(&CheckedNodeId) -> bool) {} }",
        "impl CheckedTerm { pub fn admit_with(&mut self, map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>) {} }",
    ] {
        assert!(!boundary(source).is_empty(), "admission control {source}");
    }
    assert!(boundary("pub struct ProtocolForm; pub fn parse(_: &str) {} #[cfg(test)] mod tests { struct ParsedUnit; }").is_empty());
}

#[trace("TC-018", "FR-020-AC-18")]
#[test]
fn shared_cli_kit_is_confined_to_the_opt_in_conformance_binary() {
    let default = metadata(&[]);
    let root = package(&default, "quire-contract-ir");
    let feature = metadata(&["--features".to_owned(), "conformance-cli".to_owned()]);
    let model = package(&feature, "quire-contract-model");
    let kit = package(&feature, "ix-cli-kit");
    let kit_id = kit["id"].as_str().unwrap();
    let source = kit["source"].as_str().unwrap();
    assert!(source.starts_with("git+https://github.com/agent-ix/ix-cli-kit?branch=main#"));
    assert!(root["targets"].as_array().unwrap().iter().any(|target| {
        target["name"] == "quire-contract-conformance"
            && target["required-features"] == json!(["conformance-cli"])
    }));
    assert!(root["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .any(|dependency| {
            dependency["name"] == "ix-cli-kit"
                && dependency["kind"].is_null()
                && dependency["optional"] == true
        }));
    assert!(
        !production_graph(&default, root["id"].as_str().unwrap(), false)
            .unwrap()
            .contains(kit_id)
    );
    assert!(production_graph(
        &feature,
        package(&feature, "quire-contract-ir")["id"]
            .as_str()
            .unwrap(),
        false,
    )
    .unwrap()
    .contains(kit_id));
    assert!(
        !production_graph(&feature, model["id"].as_str().unwrap(), false)
            .unwrap()
            .contains(kit_id)
    );
    assert_eq!(
        feature["packages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|package| package["name"] == "ix-cli-kit")
            .count(),
        1
    );
}
