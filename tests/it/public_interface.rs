use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use ix_trace_rs::trace;
use quire_contract_model::{
    derive_structural_candidates, CheckedSemanticNodeV2, StructuralCandidateError,
};
use serde_json::{json, Value};

const MODEL_ROOT: &str = include_str!("../../crates/quire-contract-model/src/lib.rs");
const BRIDGE_ROOT: &str = include_str!("../../src/lib.rs");
const FR_019: &str = include_str!("../../spec/model/functional/FR-019-rust-library-interface.md");

const FORBIDDEN_PUBLIC_NAME_PARTS: [&str; 17] = [
    "rust",
    "gumbo",
    "aadl",
    "hamr",
    "solver",
    "runtime",
    "usize",
    "isize",
    "pathbuf",
    "filesystem",
    "file_path",
    "os_path",
    "pointerwidth",
    "pointer_width",
    "linux",
    "windows",
    "macos",
];

fn portable_name_violations<I, S>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    names
        .into_iter()
        .flat_map(|name| {
            let name = name.as_ref().to_ascii_lowercase();
            FORBIDDEN_PUBLIC_NAME_PARTS
                .into_iter()
                .filter(|part| name.contains(part))
                .map(|part| format!("{name}: {part}"))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn collect_schema_names(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for field in ["properties", "$defs", "definitions"] {
                if let Some(entries) = object.get(field).and_then(Value::as_object) {
                    names.extend(entries.keys().cloned());
                }
            }
            if let Some(title) = object.get("title").and_then(Value::as_str) {
                names.push(title.to_owned());
            }
            if let Some(variants) = object.get("enum").and_then(Value::as_array) {
                names.extend(variants.iter().filter_map(Value::as_str).map(str::to_owned));
            }
            for child in object.values() {
                collect_schema_names(child, names);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_schema_names(item, names);
            }
        }
        _ => {}
    }
}

fn public_schema_names() -> Vec<Vec<String>> {
    let schemas = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let mut paths = fs::read_dir(schemas)
        .expect("schema directory must be readable")
        .map(|entry| entry.expect("schema entry must be readable").path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".schema.json"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let mut names = vec![path
                .file_name()
                .expect("schema path has a filename")
                .to_string_lossy()
                .into_owned()];
            let schema: Value =
                serde_json::from_slice(&fs::read(path).expect("schema must be readable"))
                    .expect("public schema must be JSON");
            collect_schema_names(&schema, &mut names);
            names
        })
        .collect()
}

fn quoted_names(text: &str) -> impl Iterator<Item = &str> {
    text.split('`').skip(1).step_by(2)
}

fn expected_items() -> BTreeMap<String, BTreeSet<String>> {
    let (_, after_heading) = FR_019
        .split_once("### Public items")
        .expect("FR-019 public items table must exist");
    let (table, _) = after_heading
        .split_once("## Acceptance Criteria")
        .expect("FR-019 acceptance criteria must follow the table");
    let mut items = BTreeMap::<String, BTreeSet<String>>::new();
    for row in table.lines().filter(|line| line.starts_with("| `")) {
        let cells: Vec<_> = row.split('|').collect();
        let module = quoted_names(cells[1])
            .next()
            .expect("public item row must name a module")
            .split_whitespace()
            .next()
            .expect("module name must be present");
        let names = quoted_names(cells[2]).map(|name| {
            if module == "crate_root" {
                name.strip_prefix("module ").unwrap_or(name).to_owned()
            } else {
                name.to_owned()
            }
        });
        items.entry(module.into()).or_default().extend(names);
    }
    items
}

fn exported_items() -> BTreeMap<String, BTreeSet<String>> {
    let mut items = BTreeMap::<String, BTreeSet<String>>::new();
    let mut exports = MODEL_ROOT.split("pub use ").skip(1);
    for export in exports.by_ref() {
        let statement = export
            .split(';')
            .next()
            .expect("export must end with semicolon");
        assert!(
            !statement.contains('*'),
            "model root must not glob-export: {statement}"
        );
        if statement.starts_with("output_mapping::MappingAllocationPoint") {
            continue;
        }
        let (module, names) = statement
            .split_once("::{")
            .expect("default-feature model exports must name items in braces");
        let names = names.strip_suffix('}').expect("item list must close");
        for name in names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(
                items.entry(module.into()).or_default().insert(name.into()),
                "duplicate export: {module}::{name}"
            );
        }
    }
    let mut sources =
        vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("crates/quire-contract-model/src")];
    let mut exported_macros = BTreeSet::new();
    while let Some(path) = sources.pop() {
        for entry in fs::read_dir(path).expect("model source directory must be readable") {
            let path = entry
                .expect("source directory entry must be readable")
                .path();
            if path.is_dir() {
                sources.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).expect("model source file must be readable");
                for macro_export in source.split("#[macro_export]").skip(1) {
                    assert_eq!(
                        path.file_name().and_then(|name| name.to_str()),
                        Some("code.rs"),
                        "FR-019 lists exported macros under code"
                    );
                    let declaration = macro_export
                        .trim_start()
                        .strip_prefix("macro_rules! ")
                        .expect("exported macro must have an adjacent macro_rules declaration");
                    let name = declaration
                        .split(|character: char| {
                            !character.is_ascii_alphanumeric() && character != '_'
                        })
                        .next()
                        .expect("exported macro must have a name");
                    assert!(
                        exported_macros.insert(name.to_owned()),
                        "duplicate exported macro: {name}"
                    );
                }
            }
        }
    }
    items
        .entry("code".into())
        .or_default()
        .extend(exported_macros);
    if MODEL_ROOT
        .lines()
        .any(|line| line.trim() == "pub mod source_obligation;")
    {
        items
            .entry("crate_root".into())
            .or_default()
            .insert("source_obligation".into());
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("crates/quire-contract-model/src/source_obligation.rs"),
        )
        .expect("source-obligation module must be readable");
        let module_items = source.lines().filter_map(|line| {
            let declaration = line.strip_prefix("pub ")?;
            let name = declaration
                .strip_prefix("struct ")
                .or_else(|| declaration.strip_prefix("enum "))
                .or_else(|| declaration.strip_prefix("trait "))
                .or_else(|| declaration.strip_prefix("fn "))
                .or_else(|| declaration.strip_prefix("type "))
                .or_else(|| declaration.strip_prefix("const "))
                .or_else(|| declaration.strip_prefix("static "))
                .unwrap_or_else(|| {
                    panic!("unrecognized public source-obligation declaration: {declaration}")
                });
            Some(
                name.split(|character: char| {
                    !character.is_ascii_alphanumeric() && character != '_'
                })
                .next()?
                .to_owned(),
            )
        });
        items
            .entry("source_obligation".into())
            .or_default()
            .extend(module_items);
    }
    if MODEL_ROOT
        .lines()
        .any(|line| line.trim() == "pub mod selected_population;")
    {
        items
            .entry("crate_root".into())
            .or_default()
            .insert("selected_population".into());
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("crates/quire-contract-model/src/selected_population.rs"),
        )
        .expect("selected-population facade must be readable");
        for declaration in source.lines().filter(|line| line.starts_with("pub ")) {
            assert!(
                declaration.starts_with("pub use "),
                "unrecognized public selected-population declaration: {declaration}"
            );
        }
        let declaration = source
            .split_once("pub use crate::checked_package::{")
            .expect("selected-population facade must name its exports")
            .1
            .split_once("};")
            .expect("selected-population export list must close")
            .0;
        items
            .entry("selected_population".into())
            .or_default()
            .extend(
                declaration
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned),
            );
    }
    items
}

#[trace("TC-058", "FR-019-AC-5", "FR-052-AC-1")]
#[test]
fn tc_058_model_root_exports_exactly_the_documented_public_items() {
    assert_eq!(exported_items(), expected_items());
    assert_eq!(
        MODEL_ROOT
            .lines()
            .filter(|line| line.starts_with("pub ")
                && !line.starts_with("pub use ")
                && !line.starts_with("pub mod source_obligation;")
                && !line.starts_with("pub mod selected_population;"))
            .count(),
        0,
        "model root must expose items only through named exports"
    );
    assert!(MODEL_ROOT
        .lines()
        .any(|line| line == "pub mod source_obligation;"));
    assert!(MODEL_ROOT
        .lines()
        .any(|line| line == "pub mod selected_population;"));
}

#[trace("TC-456", "NFR-002-AC-1")]
#[test]
fn tc_456_public_schema_and_model_names_are_portable() {
    let forbidden_schema = json!({
        "title": "RuntimeHandle",
        "properties": { "pointer_width": { "enum": ["GumboNode"] } }
    });
    let mut forbidden_names = Vec::new();
    collect_schema_names(&forbidden_schema, &mut forbidden_names);
    assert_eq!(portable_name_violations(forbidden_names).len(), 3);
    assert!(portable_name_violations(["jsonpointer"]).is_empty());

    let model_items = exported_items();
    let public_modules = MODEL_ROOT
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod ")?.strip_suffix(';'))
        .collect::<BTreeSet<_>>();
    let inventoried_modules = model_items
        .get("crate_root")
        .expect("public modules must be inventoried")
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(public_modules, inventoried_modules);
    for module in public_modules {
        assert!(
            model_items
                .get(module)
                .is_some_and(|items| !items.is_empty()),
            "public module {module} must have an item inventory"
        );
    }
    for declaration in MODEL_ROOT.lines().filter(|line| line.starts_with("pub ")) {
        assert!(
            declaration.starts_with("pub mod ") || declaration.starts_with("pub use "),
            "unrecognized public model-root declaration: {declaration}"
        );
    }
    let model_names = model_items
        .into_iter()
        .flat_map(|(module, items)| std::iter::once(module).chain(items));
    let schema_names = public_schema_names();
    assert!(!schema_names.is_empty(), "public schemas must be present");
    let violations =
        portable_name_violations(model_names.chain(schema_names.into_iter().flatten()));
    assert!(
        violations.is_empty(),
        "forbidden public names: {violations:?}"
    );
}

#[trace("TC-058", "FR-019-AC-5")]
#[test]
fn tc_058_fault_injection_adds_only_its_named_export() {
    let gate =
        "#[cfg(feature = \"fault-injection\")]\npub use output_mapping::MappingAllocationPoint;";
    assert!(MODEL_ROOT.contains(gate));
    assert_eq!(
        MODEL_ROOT
            .lines()
            .filter(|line| *line == "pub use output_mapping::MappingAllocationPoint;")
            .count(),
        1
    );
}

#[trace("TC-058", "FR-019-AC-5")]
#[test]
fn tc_058_root_crate_exposes_only_its_kani_module() {
    let public: Vec<_> = BRIDGE_ROOT
        .lines()
        .filter(|line| line.starts_with("pub "))
        .collect();
    assert_eq!(public, ["pub mod kani;"]);
}

/// The external consumer can call the borrowed graph API and distinguish
/// every refusal by its variant, without reading error messages.
#[trace("TC-058", "FR-048-AC-1")]
#[test]
fn tc_058_structural_candidate_api_is_typed_and_closed() {
    let draft: CheckedSemanticNodeV2 = serde_json::from_value(json!({
        "node_id": {"domain": "quire.checked-semantic-node/v1", "digest": "draft"},
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "boolean",
        "semantic_type": {"domain": "quire.checked-semantic-node/v1", "digest": "draft"},
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "aggregate", "members": []},
    }))
    .expect("typed draft");
    let candidates =
        derive_structural_candidates(std::slice::from_ref(&draft), 4096, 10).expect("candidate");
    assert_eq!(candidates.nodes().len(), 1);
    assert_eq!(candidates.nodes()[0].position(), 0);
    assert_eq!(
        candidates.nodes()[0].candidate_node_id().domain.as_ref(),
        "quire.checked-semantic-node/v1"
    );
    assert!(candidates.groups().is_empty());
    let repeated = [draft.clone(), draft];
    let error = derive_structural_candidates(&repeated, 4096, 10).expect_err("repeat");
    let kind = match error {
        StructuralCandidateError::RepeatedKey {
            first_position,
            repeated_position,
            ..
        } => {
            assert_eq!((first_position, repeated_position), (0, 1));
            "repeated"
        }
        StructuralCandidateError::InvalidTagForm { .. } => "tag-form",
        StructuralCandidateError::InvalidClosedShape { .. } => "shape",
        StructuralCandidateError::UnsupportedGroupMember { .. } => "group-member",
        StructuralCandidateError::CounterOverflow { .. } => "overflow",
        StructuralCandidateError::WorkExhausted { .. } => "work",
        StructuralCandidateError::CanonicalEncoding { .. } => "canonical",
        StructuralCandidateError::ApplicationLocal { .. } => "application-local",
    };
    assert_eq!(kind, "repeated");
}
